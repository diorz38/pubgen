# Publisher Studio (pubgen) — System Architecture & Technical Plan

## Positioning

Publisher Studio is a **standalone Rust desktop app** that lives entirely in `pubgen/` (its own cargo workspace). The DesignCraft repo is **reference only and is never modified**: no fork, no upstream changes required, no workspace coupling.

The boundary between the two apps is DesignCraft's **documented external interfaces**:

* **JSON control protocol** over TCP `127.0.0.1:7979` (`docs/control-protocol.md` in the DesignCraft repo) — live/interactive mode. Key methods used: `engine.execute` (slot fills, `paste.svg`), `document.inspect` (pages, items, oversets), `ui.render` (thumbnails), `ui.paste` (SVG markup → editable vector paths).
* **`designcraft-cli` headless** — batch mode (`designcraft-cli run --cmd '...' --export ...`).
* **`designcraft-cli mcp`** — optional alternative channel, same command surface.

**DesignCraft is the layout interpreter.** All composition (Knuth–Plass), hyphenation, thread reflow, overflow/overset detection, rendering and PDF export happen inside a DesignCraft instance. Publisher Studio never parses, patches or re-serializes `.designcraft` file internals — the earlier "Direct File Synthesis" mode (unzip → patch JSON → rezip) is **dropped**: it bypasses the layout engine, silently produces stale oversets, and couples to an internal schema that drifts every release.

Licensing: DesignCraft is MIT/Apache-2.0, so mimicking its shell patterns (thin eframe app, command pattern, logging) is fine; keep its NOTICE attribution if any code is copied verbatim.

Publisher Studio saves and opens its own project format, **`.pubgen`** (e.g. `DDA2026.pubgen`) — the user's work sources (metadata, content, assets, translations) in one file. See Module 2.

```
┌────────────────────────────────────────────────────────────────────────┐
│                     PUBLISHER STUDIO (pubgen/, Rust)                   │
│                                                                        │
│   ┌───────────────────┐    ┌─────────────────┐    ┌────────────────┐  │
│   │ Content Ingestion │───▶│ AI Translation  │───▶│ Template &     │  │
│   │ (DOCX, XLSX, Img) │    │ & Copyfitting   │    │ Slot Binding   │  │
│   │ → md / csv / json │    └─────────────────┘    └───────┬────────┘  │
│   └───────────────────┘                                   │           │
│   metadata via in-app form → metadata.json                │ Output    │
└───────────────────────────────────────────────────────────┼───────────┘
                                                            │
                 ┌──────────────────────────────────────────┴───────────┐
                 │  Batch: spawn designcraft-cli headless (per run)     │
                 │  Live:  TCP client → 127.0.0.1:7979 (or MCP)         │
                 └───────────────────────────┬──────────────────────────┘
                                             ▼
┌────────────────────────────────────────────────────────────────────────┐
│                     DESIGNCRAFT INSTANCE (unmodified)                  │
│                                                                        │
│   * Knuth–Plass paragraph composer & optical margins                   │
│   * Dictionary hyphenation & text frame threads                        │
│   * Overflow / overset status reporting                                │
│   * Export targets: PDF/X-4 (Press), PDF/A-2b, PNG (optional)          │
└────────────────────────────────────────────────────────────────────────┘
```

## Document Model — Page Kinds

A pubgen publication is an **ordered list of page kinds**. Each kind = pubgen generator + DesignCraft template + slot binding; a new feature is a new kind or a new slot filler — never an architecture change:

| Kind | Input | Output path |
|------|-------|-------------|
| cover | metadata, cover template, optional AI art/copy | slot fills |
| content | MD sections | `slot:body_flow` |
| table | CSV | styled table frame |
| chart | CSV | pubgen-rendered SVG → `paste.svg` |
| map | SVG map + XLSX data | colorized SVG → `paste.svg` |
| toc | content tree + page mapping (scoped) | `slot:toc_entries` |
| divider | section metadata | slot fills |

## App skeleton

```
pubgen/
  Cargo.toml            # own workspace; current egui/eframe versions, not DesignCraft's pins
  app/
    src/
      main.rs           # eframe entry (thin shell, mimics apps/designcraft/src/main.rs shape)
      control_client.rs # TCP client for the control protocol (inverse of DesignCraft's control_server.rs)
      process.rs        # spawn/manage designcraft --control and designcraft-cli headless runs
      project.rs        # open/save .pubgen project containers (zip)
      vector.rs         # SVG generation: charts (from CSV), choropleth maps (SVG + XLSX)
      glossary.rs       # term base: prompt injection + post-translation enforcement
      covers.rs         # cover template grid + AI art/copy candidates
      toc.rs            # TOC entries, scope filters, two-pass page numbers
      ingest.rs         # background converters: DOCX/RTF → MD, XLSX → CSV
      metadata_form.rs  # interactive metadata form → metadata.json
      slots.rs          # template slot discovery, char budgets
      translate.rs      # LLM call + copyfit retry loop
      state.rs          # app state
      panel_left.rs / panel_center.rs / panel_right.rs
```

pubgen actions follow DesignCraft's **command pattern** (id, label, params, run): batch generation is a scriptable list of commands, and the JSON protocol maps naturally onto it.

## Core Operational Modules

### 1. Content Ingestion & Normalization

Accepted inputs are normalized in the background to a small internal format set; everything downstream (translation, slot binding, prompts) only ever sees normalized files:

* **Metadata:** an in-app **interactive form** (title, issue, date, authors, pricing …). Publisher Studio writes the resulting `metadata.json` — no hand-edited metadata files.
* **Paragraph content:** **DOCX / RTF** files are converted to **Markdown** in a background job (headings, paragraphs, lists, bold/italic preserved). Exotic source formatting is dropped by design — visual style comes from the DesignCraft template, not the source document.
* **Tables:** **XLSX** sheets are converted to **CSV** in a background job (one sheet → one CSV, typed cells preserved).
* **Images:** passed through as-is.

Rust library candidates: `docx` (DOCX read), `rtf-parser`, `calamine` (XLSX read), `csv` (write). Conversions are cached next to the source and re-run when the source changes. The content tree and translation diff operate on the normalized MD/CSV, never on the originals.

### 2. Project File — `.pubgen`

A saved work is a **plain ZIP container with the extension changed to `.pubgen`** (e.g. `DDA2026.pubgen`). It holds the user's work sources in one portable file. It is pure data owned entirely by Publisher Studio — no layout semantics inside — so unlike `.designcraft` files it is read and written directly by the app:

```
DDA2026.pubgen (zip)
├── pubgen.json        # manifest: format_version, project name, template reference, slot mapping, app settings
├── metadata.json      # from the metadata form
├── content/           # normalized sources
│   ├── *.md           # paragraphs (converted from DOCX/RTF)
│   └── *.csv          # tables (converted from XLSX)
├── assets/            # images
└── translations.json  # per-slot translated text + fit status (optional)
```

Rules:

* `pubgen.json` carries a `format_version`; readers accept older versions and migrate, writers never emit unversioned files.
* The referenced DesignCraft template (`.designcraft`) is embedded in the container so the project is portable; layout generation always runs inside DesignCraft.
* Save is atomic: write to a temp file, then rename over the target. Never leave a half-written `.pubgen`.
* Unknown extra entries are preserved on re-save (forward compatibility).
* Rust crate: `zip`.

### 3. Template & Slot Convention

Templates are native `.designcraft` project files created once in DesignCraft by a designer. Elements are given semantic slot identifiers in their frame metadata or names:

* `slot:cover_image`: Target image frame (pre-configured with bleed and focal point fitting).
* `slot:title`: Main headline frame (inherits `Headline_1` paragraph style, locked baseline).
* `slot:subtitle`: Deck/lead text frame.
* `slot:issue_meta`: Barcode, price, volume number.
* `slot:body_flow`: Threaded story frames spanning multi-column spreads.
* `slot:table_data`: Pre-styled table containers.

Publisher Studio discovers slots **through DesignCraft** (load template via CLI/control and enumerate frame names/metadata), never by unpacking the file itself.

### 4. AI Translation & Copyfit Engine

Target languages can expand text volume by 15–35%, so translation runs under strict layout constraints:

* **Structured Payload:** Only text elements are extracted into a key-value JSON schema.
* **Character Budget Constraint:** Each slot carries a bounding-box character limit (`max_chars`). If a translation exceeds the budget, the LLM is re-prompted to rewrite concisely without losing core meaning.
* **Table Cell Preservation:** Tables are 2D arrays, translated cell-by-cell, preserving numeric metrics, symbols and formatting tokens.
* **Ground truth = engine, not estimates:** `max_chars` budgets gate the LLM prompt, but final fit verification is DesignCraft's own overflow/overset status after placement — Publisher Studio never computes line-breaking itself.

### 5. DesignCraft Connector (single mode — DesignCraft as interpreter)

* **Batch:** spawn `designcraft-cli` headless per run: open template, apply slot fills via `--cmd`, export PDF. Offline-friendly, engine-correct, no GUI needed.
* **Live/interactive:** launch `designcraft --control 7979` in the background and drive it with the JSON control protocol (frame updates, overflow status, exports).
* Process lifecycle (spawn, health check, restart, teardown) is owned by `process.rs`; the rest of the app only sees an abstract "DesignCraft is OK / busy / down" status.

### 6. Verification Output — PDF

The center panel is a **viewer, not an editor**. The verification artifact is the exported PDF (PDF/X-4 for press, plain PDF for quick checks):

* Publisher Studio never renders pages itself.
* The generated PDF is opened in the system PDF viewer; the in-app panel shows run status, per-slot fit results (from engine overflow status) and the log.
* Optional later: PNG thumbnails via `designcraft-cli --export out.png` if a visual strip is ever wanted — one CLI flag away, still engine-rendered.

## Desktop GUI Layout

3-column studio layout: batch preparation, translation inspection, run/export. No live canvas.

```
+----------------------------------------------------------------------------------------------------+
|  [Logo] Publication Generator Studio   | Project: Winter_2026_Catalog.pubgen   | (● DesignCraft: OK)|
+----------------------------------------------------------------------------------------------------+
| LEFT: Source & Content    | CENTER: Run & Verification              | RIGHT: Slot Inspector & Run  |
|                           |                                         |                              |
| [ + Import Content ]      | ┌── Run Log ──────────────────────────┐ | ┌── Template Settings ──────┐|
| [ Metadata Form ]         | | Generated spread 1-2 in 140ms.      | | │ Template: Modern_Cover_A  │|
| - article_tech.docx → md  | | DesignCraft: 0 oversets.            | | │ Base Language: EN         │|
| - specs_table.xlsx → csv  | ┌── Slot Fit ─────────────────────────┐ | │ Target Lang:   ID (Indo)  │|
|                           | | slot:title      42/50 chars   OK    | | └─────────────────────────┘|
| ┌── Content Tree ────────┐| | slot:subtitle    88/90 chars  OK    | | ┌── Detected Slots (5) ───┐|
| │ ▼ Cover Section        │| | slot:body_flow   OVERSET  → retry   | | │ ☑ slot:cover_image       │|
| │   • Title              │| └─────────────────────────────────────┘ | │ ☑ slot:title             │|
| │   • Subtitle           │|                                         │ │   Length: 42/50 chars    │|
| │   • Hero Image         │|         [ Export PDF ]  [ Open PDF ]    | │   Status: OK (No Wrap)   │|
| │ ▼ Feature Story        │|         (opens system PDF viewer)       | │ ☑ ...                    │|
| │   • Body (3 cols)      │|                                         | │ ┌── AI Copyfit Rules ─────┐|
| │   • Data Table (4x6)   │|                                         | │ │ ☑ Auto-shorten overflow  │|
| └────────────────────────┘|                                         | | │ ☑ Retain tone (Editorial)│|
| ┌── Translation Diff ────┐|                                         | │ └─────────────────────────┘|
| │ EN: "Quantum Leap"     │| Log: Batch finished in 1.2s. PDF ready. | [ Generate All Spreads ]     |
| │ ID: "Lompatan Kuantum" │|                                         | [ Export PDF/X-4 ] [Open GUI]|
| └────────────────────────┘|                                         |                              |
+----------------------------------------------------------------------------------------------------+
| Status: All slots filled. Ready to export.                         | Workers: Idle                 |
+----------------------------------------------------------------------------------------------------+
```

### Detailed Panel Functions

#### 1. Left Panel — Source Content & Assets

* **Input Files:** Drag-and-drop bucket for DOCX/RTF documents, XLSX sheets and image folders. Background converters produce the normalized MD/CSV files; the tree shows the normalized results.
* **Metadata Form:** interactive form (title, issue, date, authors, pricing …). Publisher Studio generates `metadata.json` from it — no hand-edited metadata files.
* **Document Tree:** Structured view of sections, chapters or cover elements extracted from the normalized MD/CSV.
* **Translation Editor / Diff:** Compare source phrase vs AI translation before placement. Editing here is text-only — it never requires a re-render to stay responsive.

#### 2. Center Panel — Run & Verification

* **Run Log:** DesignCraft process status, layout timings, composition warnings.
* **Slot Fit:** Per-slot character budget vs engine-reported fit status (OK / near-limit / overset → retry or manual fix).
* **Export / Open PDF:** Trigger PDF export through the connector; open the result in the system viewer.

#### 3. Right Panel — Template Slot Mapping & Export Controls

* **Template Selector:** Pick from pre-made DesignCraft template files.
* **Slot Inspector:** All `slot:*` identifiers discovered through DesignCraft, with character-count gauges vs budgets.
* **AI Controls:** Translation model, target language, overflow rules (re-prompt LLM within budget, tighten wording, or flag for manual edit).
* **Primary Actions:**
  * **Generate Publication:** Compile all content into a `.designcraft` document via DesignCraft.
  * **Export PDF/X-4:** Engine-side export of the commercial-ready CMYK PDF.
  * **Open in DesignCraft:** Launch the DesignCraft GUI with the generated document for final manual touch-ups.

## Phased Implementation Roadmap

### Phase 1a — App Shell + Live Pipe (Week 1)

1. Init the `pubgen/` cargo workspace; minimal eframe shell (window, status bar, log panel).
2. `process.rs`: launch `designcraft --control 7979` in the background, health-check it.
3. `control_client.rs`: prove the pipe with an `engine.execute` round-trip.

**Acceptance:**

- [ ] `cargo run -p pubgen` opens the shell window with status bar and log panel.
- [ ] The app launches `designcraft --control 7979` itself and reports OK / down in the status bar.
- [ ] An `engine.execute` round-trip from the app returns a result shown in the log panel.
- [ ] `git status` in the DesignCraft checkout stays clean — reference repo untouched.

### Phase 1b — Ingestion + Project Container (Weeks 1–2)

1. `ingest.rs`: background DOCX/RTF → MD and XLSX → CSV converters.
2. `metadata_form.rs`: interactive metadata form writing `metadata.json`.
3. `project.rs`: save/open `.pubgen` (zip + manifest with `format_version`), storing normalized content, metadata and assets.

**Acceptance:**

- [ ] DOCX and RTF inputs produce MD with headings, paragraphs, lists, bold/italic preserved.
- [ ] XLSX sheets produce CSV with typed cells preserved (numbers stay numbers).
- [ ] The metadata form writes `metadata.json` containing all fields.
- [ ] Save → reopen `.pubgen` reproduces identical normalized content, assets and metadata.
- [ ] Interrupted save never leaves a corrupt `.pubgen` (temp + atomic rename verified).

### Phase 1c — Slot Fill Batch (Week 2)

1. `slots.rs`: slot discovery through DesignCraft, `max_chars` budgets.
2. Batch fill via headless `designcraft-cli` → PDF export.

**Acceptance:**

- [ ] Slot discovery lists all `slot:*` frames of a template with budgets — via DesignCraft, not by unpacking the file.
- [ ] Batch fill produces a `.designcraft` document and an exported PDF.
- [ ] PDF shows correct styles, hierarchy and geometry; oversets are reported by the engine, none unlisted.
- [ ] No code path reads or writes `.designcraft` internals (no zip/JSON patching).

### Phase 2: Translation & Copyfitting Pipeline (Weeks 3–4)

1. Translation utility over an LLM API.
2. Prompts accept per-slot character limits and return strict JSON mappings.
3. Copyfit loop: budget exceeded → automated LLM rephrase; placed anyway → engine overset status drives one more retry, then flag for human edit.
4. Table handling: cell-by-cell translation preserving numerics, symbols, formatting tokens.
5. Glossary of terminology (F4): editor + CSV/XLSX import-export; terms injected into prompts as mandatory translations, then enforced post-translation (auto-fix, else flag) on paragraphs and table cells. Enforcement runs before the budget check — substitutions change length.

**Acceptance:**

- [ ] Translation returns strict JSON keyed by slot; malformed LLM output is retried, never accepted.
- [ ] Over-budget translations trigger automatic rephrase; final text fits or is flagged.
- [ ] Engine overset status after placement drives one more retry, then flags for human edit.
- [ ] Glossary terms appear verbatim as specified; `do-not-translate` terms untouched; violations auto-fixed or flagged.
- [ ] Table cells translated cell-by-cell; numerics, symbols and formatting tokens unchanged.
- [ ] Loop order holds: glossary enforce → budget check → rephrase.

### Phase 3: Desktop Interface Polish (Weeks 5–6)

1. Full 3-column GUI on top of the working pipe (content tree, translation diff, slot inspector, fit panel).
2. `process.rs` lifecycle: background DesignCraft launch, health check, restart, teardown.
3. Export actions wired end-to-end: Generate → Export PDF/X-4 → Open PDF; Open in DesignCraft handoff.

**Acceptance:**

- [ ] All panels live: content tree, translation diff, slot inspector with character gauges, fit panel, run log.
- [ ] Process lifecycle works: launch, health check, restart after kill, clean teardown (no orphan DesignCraft processes).
- [ ] Full flow — import sources → metadata form → translate → generate → export PDF/X-4 → open PDF — runs from the GUI with no CLI and no manual steps.
- [ ] Open in DesignCraft hands off the generated document correctly.

### Phase 4+: Feature Backlog (post-v1)

Ordered: F1+F2 vector pages first (shared SVG → `paste.svg` pipe), then F3 cover options (template grid first, AI after), then F5 multi-TOC. F4 glossary ships earlier, in Phase 2. Each feature becomes its own phase with an acceptance list when work starts.

## Feature Backlog

Capabilities verified in the DesignCraft reference repo (read-only): `ui.paste` accepts SVG markup and lands it as editable vector paths (engine command `paste.svg`, `crates/engine/src/cmd/paste_svg.rs`); `document.inspect` reports pages, items and oversets. Nothing below requires DesignCraft changes.

### F1 — Vector Chart Pages (from CSV)

* Adding a table page offers **chart types** alongside the plain table: bar, line, pie, donut, scatter.
* Pipeline: CSV → chart spec (type, columns, palette, labels) → **SVG generated by pubgen** (`vector.rs`, hand-rolled for full style control, palette from template tokens) → `paste.svg` into a chart frame.
* Same CSV source serves both the table page and the chart page — one source, two renderings.
* In-app preview: egui renders the same SVG via `egui_extras`.

### F2 — Choropleth Map Pages (SVG + Excel)

* Input: a pre-projected **SVG map** whose regions carry IDs (e.g. `id="ID-JK"`) + **Excel** data keyed by the same IDs.
* Pipeline: join data ↔ region IDs → color ramp per value → inject `fill` → legend drawn into the same SVG → `paste.svg`.
* No projection math — maps ship pre-projected. Missing data gets a distinct grey/hatch.
* UI: data column mapping, ramp selection (sequential/diverging), min/max, legend toggle. Shares the vector pipe with F1.

### F3 — Cover Design Options

* **Template covers:** grid of `.designcraft` cover templates, thumbnails via `ui.render`; picking one fills slots as usual.
* **AI art:** image model generates background candidates → placed into `slot:cover_image`; text stays typographic in the engine. Prompt seeded from publication theme; palette locked to manifest colors so art matches the interior.
* **AI copy:** LLM proposes title/subtitle variants within `max_chars` budgets.
* Compose N candidates → thumbnail grid → pick → fill slots. AI never lays out pages.

### F4 — Glossary of Terminology (ships in Phase 2)

* Term base: source term → target term per language pair, plus `do-not-translate` (brand names) and case-preserve flags. Stored in `.pubgen`; CSV/XLSX import-export.
* Applied twice: (a) prompt injection — listed terms must translate as specified; (b) post-translation enforcement — scan output, auto-fix term violations, flag what cannot be auto-fixed. Covers paragraphs **and** table cells.
* Loop order: translate → glossary enforce → budget check → rephrase. Enforcement changes length, so it precedes copyfit.

### F5 — TOC Layouts (multi-TOC)

* A TOC page = entries + layout template (`.designcraft` with a `slot:toc_entries` flow frame) + **scope filter** (section/chapter range) → several TOC pages in one publication.
* Page numbers via two-pass generation: pass 1 with placeholder TOC → `document.inspect` maps each heading to its page → pass 2 fills real numbers → repeat until stable (typically 2 passes).
* Layout options: template choice, entry depth (H1 / H1–H2), tab leaders, leader dot style.

## Decisions Log

* 2026-11-02 — DesignCraft repo is reference-only; pubgen is a standalone Rust app in `pubgen/`, boundary = control protocol + CLI. No fork.
* 2026-11-02 — "Direct File Synthesis" (unzip/patch/rezip) dropped; DesignCraft is the sole layout interpreter, batch goes through headless `designcraft-cli`.
* 2026-11-02 — Preview is view-only; PDF is the verification artifact. No live canvas, no PNG screenshots in v1 (PNG thumbnails optional later via CLI).
* 2026-11-02 — Content inputs fixed: metadata via in-app interactive form (generates `metadata.json`); DOCX/RTF paragraph files and XLSX tables are converted in the background to MD and CSV; downstream pipeline consumes only normalized MD/CSV/JSON/images.
* 2026-11-02 — Project format: plain zip with `.pubgen` extension (e.g. `DDA2026.pubgen`), owned solely by pubgen — direct read/write allowed (unlike `.designcraft`). Versioned manifest, embedded template, atomic save.
* 2026-11-02 — Publication modeled as ordered page kinds (cover/content/table/chart/map/toc/divider); features are new kinds or slot fillers.
* 2026-11-02 — Verified in DesignCraft: `ui.paste` accepts SVG as editable vector paths (`paste.svg` engine command) and `document.inspect` reports pages/items — vector pages (F1/F2) and TOC page mapping (F5) need no DesignCraft changes.
