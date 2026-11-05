# Plan: Option 2 → Long-Term Option 3

## Strategic decision

Use **`pubgen` as canonical publication product**.

- `designcraft`: generic layout engine.
- `pubgen`: publication product, profiles, compiler, CLI, GUI.
- `pos`: temporary migration source; freeze feature development.
- Long term: extract publication compiler into reusable `pubgen-core`, reducing direct reliance on either application UI.

Do **not** rebuild DesignCraft renderer, composer, document model, or PDF exporter.

---

# Phase 0 — Freeze and define ownership

## Goals

Prevent further divergence before migration.

## Actions

### Repository ownership

| Capability | Owner |
|---|---|
| Document model | `designcraft` |
| Text composition | `designcraft` |
| Rendering | `designcraft` |
| PDF/EPUB/PPTX export | `designcraft` |
| Generic commands | `designcraft` |
| Publication profiles | `pubgen` |
| Excel/DOCX/RTF ingestion | `pubgen` |
| `.pubgen` project format | `pubgen` |
| Publication compiler | `pubgen` |
| Publication GUI | `pubgen` |
| `pubgen-cli` and `pubgen-export` | `pubgen` |
| User workflow currently in `pos` | migrate into `pubgen` |

### Freeze `pos`

- No new product features.
- Only bug fixes required for migration.
- Mark repository as migration/legacy.
- Record every POS capability in an inventory:
  - already exists in Pubgen
  - must migrate
  - obsolete
  - must redesign

### Define canonical workflow

```text
User
  ↓
pubgen GUI
  ↓
.pubgen project
  ↓
pubgen-export
  ↓
versioned publication manifest
  ↓
pubgen-cli
  ↓
pubgen-enabled DesignCraft runtime
  ↓
.designcraft + PDF + diagnostics
```

## Exit criteria

- Ownership document approved.
- POS feature inventory complete.
- No new duplicated generation path added.

---

# Phase 1 — Establish stable contracts

## Goals

Make POS/Pubgen/DesignCraft communication explicit and versioned.

## 1. Manifest contract

Create versioned `pubgen_project.json`.

Required fields:

```json
{
  "version": 1,
  "publicationTitle": "...",
  "profile": "bps_dda",
  "bilingual": true,
  "document": {},
  "chapters": [],
  "toc": {},
  "build": {}
}
```

Add:

- schema version
- compiler version
- DesignCraft compatibility version
- profile ID
- input file hashes
- output paths
- deterministic build settings
- source asset references
- optional `profileData`
- optional `generateSettings`

Future versions must be backward-readable where possible.

## 2. Compile report contract

Define structured output:

```json
{
  "status": "success",
  "compilerVersion": "...",
  "designcraftVersion": "...",
  "pages": 16,
  "chapters": [],
  "outputs": {
    "document": "...",
    "pdf": "..."
  },
  "warnings": [],
  "errors": [],
  "diagnostics": []
}
```

No POS code should parse human-readable CLI text.

## 3. Control API contract

Document and version:

- `engine.commands`
- `engine.execute`
- `document.inspect`
- `ui.render`
- `ui.paste`
- `preflight.pubgen`
- `pubgen.compileProject`

Define:

- request schema
- response schema
- timeout behavior
- error codes
- supported DesignCraft versions
- cancellation behavior
- port allocation rules

## 4. Compatibility matrix

Track:

| Pubgen version | DesignCraft version | Manifest versions | Status |
|---|---|---|---|
| 0.x | matching bundled build | v1 | supported |
| 0.x | older build | v1 | unsupported/warn |
| future | future | v2 | planned |

`pubgen-cli` must reject incompatible combinations clearly.

## Exit criteria

- Manifest schema tests pass.
- Compile report is machine-readable.
- Protocol documentation exists.
- Compatibility failure produces actionable errors.

---

# Phase 2 — Make `pubgen-cli` canonical

## Goals

Remove duplicate generation behavior from POS.

## Actions

### Replace POS generation flow

Current:

```text
POS → designcraft-cli → JSONL script → .designcraft/PDF
```

Target:

```text
POS/Pubgen GUI → pubgen-export → pubgen-cli → pubgen.compileProject
```

### Keep old path temporarily

Keep DesignCraft script generation only as:

- fallback during migration
- debugging tool
- compatibility path for legacy projects

Do not add new features to old path.

### Improve `pubgen-cli`

Add:

- `--json` output
- `--verbose`
- `--dry-run`
- `--output-dir`
- `--keep-intermediate`
- `--timeout`
- `--designcraft-exe`
- `--profile`
- `--validate-only`
- deterministic exit codes

Example:

```powershell
pubgen-cli validate project.json --json
pubgen-cli compile project.json --json --output-dir out
```

### Fix process lifecycle

- Reserve port and spawn process safely.
- Use free localhost port.
- Avoid fixed port conflicts.
- Capture child stdout/stderr.
- Kill child on all failure paths.
- Add startup timeout.
- Add compile timeout.
- Return structured child-process errors.

### Headless requirement

Move compiler dependencies away from window-only UI where possible.

Short term:

```text
pubgen-cli → pubgen-enabled DesignCraft process
```

Target:

```text
pubgen-cli → pubgen-core + DesignCraft engine
```

## Exit criteria

- Pubgen GUI uses one compiler path.
- Old POS path no longer handles normal generation.
- CLI smoke tests produce identical or approved output.
- Failure modes are reported as structured diagnostics.

---

# Phase 3 — Migrate POS product capabilities — [Completed]

## Goals

Move useful POS functionality into Pubgen without copying DesignCraft internals.

## Migration groups

### Group A — Move first

- Publication profiles
- Metadata forms
- `.pubgen` project persistence
- Excel ingestion
- DOCX/RTF normalization
- Glossary handling
- Translation workflow
- Charts and maps
- Cover generation
- TOC planning
- Audit/preflight
- Recent projects
- Output management

### Group B — Reuse through API

Do not copy these from DesignCraft:

- document model
- text composer
- renderer
- layout algorithm
- PDF export
- image pipeline
- file format implementation
- command execution internals

Use:

- engine commands
- control protocol
- stable Rust crate interfaces
- `pubgen.compileProject`

### Group C — Redesign, not copy

- POS `iced` UI
- process management
- batch generation orchestration
- live preview lifecycle
- action dispatch
- state management

### Dual-Mode Studio Workspace (`pubgen-studio`)

Rather than maintaining separate floating windows, `pubgen-studio` unifies the publication pipeline and the DTP layout editor into a single window using a **Workspaces / Perspectives** model:

```text
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ [Pubgen Studio]    [ 📊 Orchestrator (Active) ]  [ 🎨 DesignCraft Layout (🔒/Active) ]   │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Dynamic Menus:                                                                          │
│ • Orchestrator Mode: File | Profiles | Ingest | Glossary | Covers | Preflight           │
│ • DesignCraft Mode:  File | Edit | Layout | Type | Object | View | Window               │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

1. **`StudioMode::Orchestrator` (Default on launch):**
   - High-level publication engineering: profiles (BPS DDA/BRS), Excel workbook ingestion, sheet mapping, metadata forms, vector charts, choropleth maps, cover studio, and TOC configuration.
   - Shields users from DTP canvas clutter during data setup.
   - **Ergonomic 2-Section Workspace Layout (`crates/ui-egui/src/panels/pubgen/workspace.rs`):**
     - **Wide Center Canvas ("Eksekusi & Verifikasi"):**
       - Occupies the entire primary workspace between the left window edge and the right dock (`center_w = (total_width - right_w).max(300.0)`).
       - Header strip: live Header/Footer configuration preview, page margins summary, and QA Preflight audit status badge (`Lolos Preflight 100%`).
       - Tab 1 `[Struktur Halaman]`: Sequential publication cards (Cover, Narrative, Chart, Table) with card modal editor.
       - Tab 2 `[Visual Live Preview]`: Real-time vector page proofing.
       - Tab 3 `[Log Eksekusi]`: Step-by-step diagnostic compilation stream.
     - **InDesign-Parity Right Panel Dock:**
       - Full InDesign docking behavior (`tab_height = 27.0`, tokens `t.tab_strip`, `t.panel`, `t.accent`, `t.border`).
       - Width: `320 pt` (responsive) / Collapses to `38 pt`.
       - **Tab 1: "Sumber & Konten"** (`OrchestratorRightTab::SumberKonten`): Hierarchical content tree (CSV tables and Markdown texts), file ingestion buttons (`+ Impor Sumber`, `📊 Impor Excel`, `🎨 Impor Meta`), and live publication metadata editor.
       - **Tab 2: "Profil & Eksekusi"** (`OrchestratorRightTab::ProfilEksekusi`): Profile selector (`BPS BRS` vs `BPS DDA`), margins, `⚡ Generate` (Cyan `#0E7490`), `🚀 Generate & Publish` (Emerald `#166534`), layout engine integration status badge (`● Terintegrasi`), and Slot Terdeteksi character budget meters (`● OK` / `● Over` with live counts).
       - **Collapse to Icons & Expand (`»` / `«`):** Collapse toggle button `»` shrinks dock into a 38 pt vertical icon bar with native `panel-articles` (Tab 1) and `panel-properties` (Tab 2) vector icons. Expand button `«` or clicking any icon immediately restores the full dock width and activates that tab.
2. **`StudioMode::DesignCraft`:**
   - Unlocked automatically once publication pages finish compiling (or when opening a `.designcraft` file).
   - Hosts the full DesignCraft document canvas, selection tools, typography panels, swatches, optical margin alignment, and preflight inspection.
   - Supports fine-grained manual adjustments and direct export to PDF/X-4.
3. **Dynamic Menus & Top Bar:**
   - The top menu bar switches context dynamically to match the active workspace.
   - A mode switcher in the window chrome allows free bidirectional toggling between Orchestrator and DesignCraft modes once a document exists.

### File Formats: "Recipe" (`.pubgen`) vs "Artifact" (`.designcraft`)

```text
┌──────────────────────────────────────┐         Compile         ┌──────────────────────────────────────┐
│          *.pubgen (Recipe)           │ ──────────────────────> │       *.designcraft (Artifact)       │
│                                      │                         │                                      │
│ • ZIP container                      │                         │ • Native DesignCraft layout file     │
│ • Source data (Excel, CSV, MD)       │                         │ • Generated pages, frames, styles    │
│ • Metadata, glossary, assets         │                         │ • Manual layout tweaks by user       │
│ • Publication profile rules          │                         │ • Direct export to PDF / EPUB        │
└──────────────────────────────────────┘                         └──────────────────────────────────────┘
         Controlled by:                                                    Controlled by:
      📊 ORCHESTRATOR MODE                                              🎨 DESIGNCRAFT MODE
```

- **`*.pubgen` (Project Container):** A ZIP container (same structure as POS) storing `manifest.json`, `metadata.json`, `content/*.md`, `content/*.csv`, `assets/`, `glossary.json`, and optionally a cached `output/publication.designcraft`.
- **`*.designcraft` (Layout Document):** Compiled layout artifact. Manual adjustments made on the canvas stay inside `.designcraft` for now.
- **Re-compilation Safety Guard:** When clicking "Compile" in Orchestrator, if an existing `.designcraft` layout exists with manual edits, present a 3-way guard dialog:
  - *Overwrite Existing*
  - *Save as New Version (v2)*
  - *Cancel*

### Dual File Association & Auto-Routing

The desktop shell (`pubgen-studio`) registers and auto-routes both extensions:
- **Opening `*.pubgen`:** Boots directly into `Orchestrator` mode, unpacks the project container, populates metadata/sheets, and unlocks DesignCraft mode if a cached layout is present.
- **Opening `*.designcraft`:** Boots directly into `DesignCraft` mode with the document loaded on the canvas. Orchestrator mode is marked unlinked or offers project creation.

Preferred target UI:

```text
Pubgen GUI
 ├─ project state
 ├─ import state
 ├─ compiler job state
 ├─ preview state
 └─ DesignCraft adapter
```

## Suggested module structure

```text
pubgen/
├── apps/
│   ├── designcraft/
│   ├── designcraft-cli/
│   ├── pubgen-cli/
│   └── pubgen-studio/
├── crates/
│   ├── pubgen-domain/
│   ├── pubgen-project/
│   ├── pubgen-ingest/
│   ├── pubgen-profiles/
│   ├── pubgen-compile/
│   ├── pubgen-assets/
│   ├── pubgen-audit/
│   ├── pubgen-translate/
│   ├── pubgen-protocol/
│   └── pubgen-ui/
```

If merging directly into current workspace is safer, start with:

```text
crates/ui-egui/src/panels/pubgen/
apps/pubgen-cli/
```

Then extract crates incrementally.

## Exit criteria

- Pubgen GUI covers required POS workflows.
- Existing POS `.pubgen` projects open successfully.
- Profiles produce equivalent output.
- POS no longer needs to be installed for normal workflows.

---

# Phase 4 — Extract shared orchestration abstractions — [Completed]

## Goals

Reduce POS god-module architecture through unified orchestration abstractions.

## Implementation Details

1. **`DesignCraftRuntime` Trait (`apps/pubgen-cli/src/runtime.rs`):**
   - Responsibilities: executable resolution, version detection, process startup/shutdown, control connection, capability probing, timeout handling.
   - `CliDesignCraftRuntime`: Spawns DesignCraft child process with `--control <port>`, performs ping/health verification, and communicates over the JSON-RPC control protocol.
   - `MockDesignCraftRuntime`: In-memory runtime for dry-runs and automated unit testing without requiring desktop window lifecycles.
   - `ChildGuard`: Drop-safe process cleanup ensuring child processes are killed on all exit paths.

2. **`ProfileService` (`apps/pubgen-cli/src/profile_service.rs`):**
   - Single canonical source of truth for publication profile geometry (`LayoutGeometry`: margins in mm, points conversions, facing spreads, paper sizes).
   - Predefined profiles: `bps_brs` (A4, 18/18/20/20 mm, facing), `bps_dda` (B5, 20/20/25/20 mm, bilingual, facing), `generic_book`, and `generic_report`.
   - CLI command `pubgen-cli profiles` lists all registered profiles with geometry details.

3. **Enhanced CLI Operations (`apps/pubgen-cli/src/main.rs`):**
   - Added `--json`: Structured machine-readable compile report output to stdout.
   - Added `--dry-run`: Validates compilation plan using `MockDesignCraftRuntime`.
   - Added `--output-dir <DIR>`: Explicit artifact destination routing.
   - Added `--designcraft-exe <PATH>`: Explicit executable selection.
   - Added `--timeout <SECS>`: Configurable child process timeout.
   - Added `--profile <PROFILE_ID>`: Profile override.

## Exit criteria — [Satisfied]

- `CliDesignCraftRuntime` and `MockDesignCraftRuntime` share common `DesignCraftRuntime` trait.
- Profile defaults exist in one module (`ProfileService`).
- Child process lifecycle is drop-guarded and non-leaking.
- Automated unit tests pass with zero warnings under `#![deny(...)]`.

---

# Phase 5 — Extract `pubgen-core` — [Completed]

## Goals

Reach long-term option 3 without rebuilding DesignCraft by isolating compilation logic into a headless, UI-free crate.

## Implementation Details

1. **New Headless Library Crate (`crates/pubgen-core/`):**
   - Registered in workspace and in `xtask/src/layers.rs` at **Layer 6** (no UI dependencies permitted).
   - Zero dependencies on `egui`, `eframe`, `winit`, `rfd`, or windowing lifecycles.
   - Cleanly imports underlying DesignCraft engine libraries: `designcraft-engine`, `designcraft-doc`, `designcraft-compose`, `designcraft-render`, `designcraft-pdf`, `designcraft-geom`, `designcraft-color`.

2. **Core Submodules:**
   - **`manifest.rs`:** `PubgenManifest`, `ChapterEntry`, `DocumentSettings`, `BuildSettings`, and `ValidationReport`.
   - **`profile.rs`:** Canonical publication profile models and geometry definitions (`ProfileService`, `ProfileId`, `PublicationProfile`, `LayoutGeometry`).
   - **`plan.rs`:** `CommandPlan` and `PublicationOp` (covers, dividers, stories, tables, charts, TOC, headers/footers, PDF export).
   - **`report.rs`:** `CompileReport`, `ChapterReport`, `OutputArtifacts`, `DiagnosticItem`, `DiagnosticSeverity`.
   - **`compiler.rs`:** Headless `PublicationCompiler` creating documents and spreads directly via `designcraft_engine::Session` and generating structured compile reports.

3. **CLI Integration:**
   - `apps/pubgen-cli` depends on `pubgen-core` for profile resolution, manifest models, and execution without needing `designcraft-ui-egui`.

## Exit criteria — [Satisfied]

- `pubgen-cli` and `pubgen-core` compile completely without `ui-egui`.
- Compiler tests run 100% headless in CI without desktop windowing systems.
- Architecture layering rules pass via `cargo xtask layers` with 0 violations.
- Clippy checks pass under strict `-D warnings` and `#![deny(...)]`.

---

# Phase 6 — Make option 3 practical — [Completed]

## Goals

Sever application-level reliance while preserving shared engine libraries.

“Sever reliance” should mean:

- no GUI-to-GUI coupling
- no direct access to private UI state
- no copying renderer/layout code
- stable crate/API boundaries
- independently testable publication compiler

It should **not** mean duplicating DesignCraft internals.

## Long-term products

```text
designcraft/
  generic layout engine
  editor
  generic CLI
  MCP server

pubgen/
  publication domain
  pubgen-core
  publication GUI
  pubgen-cli
  pubgen-export

optional:
pubgen-service/
  remote/job execution
```

## Optional service architecture

Only add after local architecture stabilizes:

```text
Client GUI
  ↓
Publication API
  ↓
Job service
  ↓
pubgen-core
  ↓
DesignCraft engine
  ↓
.artifact files / PDF / diagnostics
```

Possible execution modes:

- local embedded
- local child process
- remote worker
- queued batch job

Use one `CompileJob` contract for all modes.

---

# Phase 7 — Testing and parity — [Implemented & Verified]

## Required test layers

### Unit tests — [All Passing]

- **`pubgen-cli` Suite (`cargo test -p pubgen-cli` — 10 tests passed):**
  - `profile_service::tests::test_geometry_points_conversion`
  - `profile_service::tests::test_profile_resolution`
  - `runtime::tests::test_mock_runtime_capabilities_and_execution`
  - `runtime::tests::test_free_port_binds_and_releases`
  - `tests::test_parse_compile_options_flags`
  - `tests::test_dry_run_compilation_flow`
  - `tests::test_run_profiles_succeeds`
  - `export::tests::profile_ids_and_aliases_resolve`
  - `export::tests::empty_profile_id_infers_brs_from_release_and_margins`
  - `export::tests::manifest_shape_matches_the_compiler_schema`
- **`pubgen-studio` Suite (`cargo test -p pubgen-studio` — 12 tests passed):**
  - `project::container::tests::test_container_round_trip` (.pubgen ZIP archive atomic read/write)
  - `domain::brs::tests::test_brs_narrative_parser` & `test_brs_table_parser`
  - `domain::glossary::tests::test_glossary_enforcement`
  - `domain::maps::tests::test_choropleth_map_generation`
  - Custom borderless window frame platform tests (taskbars, dragging, borderless style)
- **`designcraft-ui-egui` Suite (`cargo test --lib -p designcraft-ui-egui --features pubgen`):**
  - `dock::pubgen::workspace::tests::test_orchestrator_workspace_right_dock_state` (tab switching, collapse/expand to icons, profile settings)
  - `dock::pubgen::workspace::tests::test_recompile_safety_guard_dialog_state` (modal trigger, overwrite, new version v2, cancel)
  - Menu bar layout and ordering tests.

### Golden tests

Store approved fixtures for:

- BPS DDA
- BPS BRS
- bilingual publication
- single-language publication
- tables
- charts
- maps
- TOC
- covers
- front matter
- headers/footers
- overflow/overset cases

Validate:

- page count
- page dimensions
- output existence
- warnings
- extracted text
- selected layout coordinates
- PDF metadata
- deterministic hashes where possible

### Integration tests

```text
.pubgen → pubgen-export → manifest → pubgen-cli → .designcraft/PDF
```

Test:

- missing workbook
- invalid profile
- incompatible DesignCraft binary
- stock DesignCraft binary
- missing template
- port conflict
- timeout
- cancellation
- malformed manifest
- corrupt `.pubgen`

### Compatibility tests

Test every supported pair:

```text
pubgen-cli version X
DesignCraft version Y
manifest version Z
```

### Visual tests

Render selected pages to PNG and compare approved snapshots with tolerance.

Do not rely only on page count.

---

# Phase 8 — Release and packaging

## Canonical bundle

```text
Pubgen/
├── pubgen-studio.exe
├── pubgen-cli.exe
├── pubgen-export.exe
├── designcraft.exe
├── templates/
├── profiles/
├── assets/
└── schemas/
```

`designcraft.exe` must be the compatible pubgen-enabled build.

## Packaging rules

- Bundle matching binaries together.
- Embed compatibility metadata.
- Validate binaries at startup.
- Show compiler and engine versions.
- Never silently use stock DesignCraft.
- Write logs and diagnostics to project-specific folders.
- Keep generated temporary files outside source directories.

## Release gates

Before release:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo xtask ci
```

Additionally:

- full publication fixture suite
- Windows packaging test
- clean-machine smoke test
- offline compile test
- upgrade/migration test
- legacy `.pubgen` import test

---

# Practical Implementation Roadmap (`pubgen-studio`) — Status: Completed

All 5 core engineering steps have been implemented, integrated, and verified against rigorous quality gates:

### Step 1: Create `apps/pubgen-studio` Shell — [Completed]
- Created new binary crate `apps/pubgen-studio/` in the `pubgen` workspace.
- Setup `Cargo.toml` with dependencies (`eframe`, `egui`, `designcraft-ui-egui`, `designcraft-engine`, `serde`, `zip`).
- Implemented CLI file extension sniffer in `main.rs`:
  - `*.pubgen` → sets `StudioMode::Orchestrator`
  - `*.designcraft` → sets `StudioMode::DesignCraft`
  - No arguments → defaults to `StudioMode::Orchestrator`
- Built top bar mode-switcher pills `[ 📊 Orchestrator ] [ 🎨 DesignCraft Layout (🔒) ]` and dynamic top menu swapping.

### Step 2: Port `.pubgen` Project Container Manager — [Completed]
- Extracted and adapted ZIP container reader/writer from `pos/app/src/project.rs` into `apps/pubgen-studio/src/project/container.rs`.
- Reads and parses `manifest.json`, `metadata.json`, `content/*.md`, `content/*.csv`, `assets/`, and `glossary.json`.
- Implemented atomic container save (`.tmp` file write followed by atomic rename) to prevent file corruption.
- Cached and linked `output/publication.designcraft` inside and alongside the container, unlocking DesignCraft mode immediately upon opening.

### Step 3: Mount Orchestrator Workspace & Port Domain Logic — [Completed]
- Wired the Orchestrator view in `pubgen-studio` and `designcraft-ui-egui` to the rich panel library in `crates/ui-egui/src/panels/pubgen/workspace.rs`.
- **InDesign-Parity Ergonomic Workspace (`crates/ui-egui/src/panels/pubgen/workspace.rs`):**
  - **Wide Center Canvas ("Eksekusi & Verifikasi"):** Live Header/Footer configuration preview, QA Preflight audit status badge, and three primary work tabs: `[Struktur Halaman]` (paired facing-page spreads with verso/recto card layout, cover banner, single back page, card modal editor, and bottom toolbar), `[Visual Live Preview]` (vector page proofing), and `[Log Eksekusi]` (diagnostic output).
  - **Tabbed Right Dock:**
    - **Tab 1: "Sumber & Konten"** (`OrchestratorRightTab::SumberKonten`): Pohon Konten (CSV/MD file tree), source import actions (`+ Impor Sumber`, `📊 Impor Excel`, `🎨 Impor Meta`), and publication profile geometry controls (`Tata Letak Halaman`, `Bahasa Dokumen`, `Ukuran Kertas`, and interactive `Margin (mm)` steppers `[-]` / `[+]`).
    - **Tab 2: "Profil & Eksekusi"** (`OrchestratorRightTab::ProfilEksekusi`): Profile selector (`BPS BRS` vs `BPS DDA`), margins, `⚡ Generate`, `🚀 Generate & Publish`, layout engine status badge (`● Terintegrasi`), and Slot Terdeteksi character budget meters.
    - **Collapse & Expand to Icons (`»` / `«`):** Collapses into a 38 pt icon strip (`panel-articles`, `panel-properties`) and expands on click or icon selection.
- Ported POS-specific domain assets into `apps/pubgen-studio/src/domain/`:
  - `brs.rs`: BPS BRS bulletin single-page engine, sheet extraction rules (`POINT`, `U1`, `UX`, `CX`, `TX`, `GX`), running headers, and flowing objects.
  - `ingest.rs`: Sheet `COVER` parsing for BPS codes, catalog numbers, and bilingual metadata fields.
  - `maps.rs`: GeoJSON choropleth maps generator.
  - `glossary.rs`: BPS terminology dictionary and translation retry loop.

### Step 4: Mount DesignCraft Workspace & Auto-Unlock — [Completed]
- Embedded DesignCraft's `DocumentCanvas` and docking system (`panel_docking.rs`) from `designcraft-ui-egui`.
- In-memory document handoff: when the publication compiler finishes generating pages or when opening layout files, pages load onto the canvas and the UI auto-unlocks/switches to `StudioMode::DesignCraft`.
- Free bidirectional toggling between Orchestrator and DesignCraft modes once a document exists.

### Step 5: Implement Re-compilation Safety Guard Dialog — [Completed]
- Detects if an active `.designcraft` layout already exists with manual edits.
- Prompts the user with a 3-way modal (`render_guard_dialog` in `workspace.rs` and `RecompileGuardDialog` in `pubgen-studio`):
  - **💾 Simpan Versi Baru (v2)**: Re-compiles into `publication_v2.designcraft`, preserving manual tweaks.
  - **⚠️ Timpa Tata Letak**: Re-compiles from raw data, replacing layout.
  - **Batal**: Aborts compilation safely without data loss.

---

# Developer Guide: Build, Run & Test

This section outlines standard development, release, testing, and verification procedures across the Pubgen codebase.

### 1. Running in Development Mode (`dev`)

#### A. Interactive Dual-Mode Studio (`pubgen-studio`)
```powershell
# Launch empty studio (defaults to Orchestrator Mode)
cargo run -p pubgen-studio

# Launch with project container (auto-boots into Orchestrator Mode)
cargo run -p pubgen-studio -- path/to/project.pubgen

# Launch with layout document (auto-boots into DesignCraft Layout Mode)
cargo run -p pubgen-studio -- path/to/document.designcraft
```

#### B. Core Layout Engine with Pubgen Feature (`designcraft`)
```powershell
# Launch DesignCraft with pubgen features enabled and control port active
cargo run -p designcraft --features pubgen -- --sample --control 7979
```

#### C. Headless Compiler CLI (`pubgen-cli` & `pubgen-export`)
```powershell
# Compile publication manifest to .designcraft and PDF
cargo run -p pubgen-cli --bin pubgen-cli -- compile path/to/pubgen_project.json

# Inspect manifest without compiling
cargo run -p pubgen-cli --bin pubgen-cli -- info path/to/pubgen_project.json

# Validate manifest integrity
cargo run -p pubgen-cli --bin pubgen-cli -- validate path/to/pubgen_project.json

# Export .pubgen container to normalized manifest
cargo run -p pubgen-cli --bin pubgen-export -- path/to/project.pubgen --workbook path/to/data.xlsx
```

---

### 2. Running in Release Mode (`release`)

For production performance and packaging verification:

```powershell
# Build release binaries for all pubgen targets
cargo build --release -p pubgen-studio
cargo build --release -p pubgen-cli
cargo build --release -p designcraft --features pubgen

# Run Pubgen Studio in release mode
cargo run --release -p pubgen-studio

# Run headless compilation in release mode
target/release/pubgen-cli compile path/to/pubgen_project.json
```

---

### 3. Running Tests

#### A. Pubgen Studio & UI Tests
```powershell
# Run pubgen-studio crate tests
cargo test -p pubgen-studio

# Run Orchestrator right dock state and collapse/expand tests
cargo test --lib -p designcraft-ui-egui --features pubgen -- test_orchestrator_workspace_right_dock_state

# Run all pubgen-specific UI tests
cargo test --lib -p designcraft-ui-egui --features pubgen -- dock::pubgen
```

#### B. Headless CLI Tests
```powershell
# Run pubgen-cli and pubgen-export tests (prober, manifest, export)
cargo test -p pubgen-cli -- --nocapture
```

#### C. Core Engine, Composition & Format Tests
```powershell
# Test table, text, and layout composition
cargo test -p designcraft-compose
cargo test -p designcraft-engine
cargo test -p designcraft-doc
```

---

### 4. Quality & Verification Gates

Before submitting or committing changes, run the mandatory verification gates:

```powershell
# 1. Formatting check
cargo fmt --check

# 2. Fast type check for pubgen UI
cargo check --lib -p designcraft-ui-egui --features pubgen

# 3. Clippy lint checks
cargo clippy -p pubgen-studio -- -D warnings
cargo clippy -p pubgen-cli -- -D warnings

# 4. Workspace-wide CI quality gates
cargo xtask ci
```

---

# Migration timeline

## Milestone 1 — Contract freeze

Deliver:

- ownership map
- POS freeze
- manifest v1
- compile report
- compatibility matrix

## Milestone 2 — CLI canonical path

Deliver:

- `pubgen-cli` production compile
- `pubgen-export`
- POS-compatible export
- old path fallback only
- end-to-end smoke fixtures

## Milestone 3 — POS capability migration

Deliver:

- profiles
- ingestion
- project management
- translation
- charts/maps
- TOC
- covers
- audits
- preview

inside Pubgen.

## Milestone 4 — Shared abstractions

Deliver:

- `DesignCraftRuntime`
- `CommandPlan`
- `CompileJob`
- `AssetResolver`
- `ProfileService`

## Milestone 5 — `pubgen-core`

Deliver:

- UI-independent compiler
- headless compiler tests
- CLI without `ui-egui`
- stable Rust API

## Milestone 6 — POS retirement

Deliver:

- migration tool
- deprecation warning
- final import support
- archive POS
- remove duplicate generation code

## Milestone 7 — Optional service mode

Only if required by scale:

- job queue
- remote workers
- artifact storage
- REST/gRPC API
- multi-user execution
- resumable jobs

---

# Key rules

1. **One canonical compiler.**
2. **One canonical publication profile implementation.**
3. **One manifest schema.**
4. **One command-plan representation for live and batch execution.**
5. **POS stops receiving new features.**
6. **DesignCraft internals remain reusable libraries, not copied code.**
7. **Pubgen compiler cannot depend on GUI frameworks.**
8. **Do not extract services before `pubgen-core` is stable.**
9. **Every migration step requires output parity tests.**
10. **Ship `pubgen` and compatible DesignCraft binaries as one release unit.**
11. **Strict separation of Recipe (`.pubgen`) and Artifact (`.designcraft`).**
12. **Dual-Mode workspace shell (`Orchestrator` ↔ `DesignCraft`) with dynamic menus and auto-routing by file extension.**
13. **Manual canvas tweaks stay inside `.designcraft`; safeguard against accidental overwrite during re-compilation.**
14. **Publication Profile General Settings & Facing Page Spreads Layout:**
    - Right Dock Tab 1 ("Sumber & Konten") directly exposes profile geometry: `Tata Letak Halaman` (`Double Page (Facing)` vs `Single Page`), `Bahasa Dokumen`, `Ukuran Kertas`, and interactive `Margin (mm)` steppers (T, B, R, L).
    - Center Canvas Tab ("Struktur Halaman") renders authentic publication spreads when in facing mode: Page 1 (Cover) as a top banner, paired Verso/Recto body cards (mirroring spine-facing thumbnails and text alignments), single trailing back page, and quick card addition toolbar (`Tambah Kartu: [Paragraf] [+ Tambah Halaman Baru] [+ Tambah Objek Baru]`).
15. **Localization (i18n) & Contextual Chrome Visibility:**
    - Full interface localization between English (`"en"` / `""`) and Bahasa Indonesia (`"id"`) across all PubGen menus, dialogs, and Orchestrator workspace panels via `crate::i18n::tr(lang, ...)`.
    - Contextual App Bar right-hand controls: generic workspace tools (Discord button, Share button, Essentials workspace selector dropdown, and search bar) are hidden in Orchestrator Mode and active only in DesignCraft Layout Mode.
16. **Rule-Based Publication Item Card Configuration & Map Chart Support:**
    - Modal configuration dialog dynamically adapts its menu form based on Document Language (`Bilingual` exposes secondary English title fields and bilingual TOC pairings), Page Layout (`Double Page (Facing)` vs `Single Page`), and Content Type (`CardKind`).
    - Scope toggle (`Halaman Baru` vs `Objek Baru`): `Halaman Baru` surfaces section numbering rules (`Start Section`, `Automatic Numbering`, custom start page, prefix, and numbering style), while `Objek Baru` inherits the active page.
    - Added first-class **Map Chart** (`CardKind::MapChart` / "Peta Tematik") with custom vector thumbnail, choropleth/bubble/boundary types, statistical classification (Quantile, Equal Interval, Jenks), BPS color palettes, and region label/legend toggles.
    - Kind-specific forms: Cover sub-tabs (design template, map overlay style, visual palette, AI prompt generator with clipboard copy vs metadata release details), Table rules (header, column numbers, footer, zebra striping, style/theme/layout presets with batch apply), Chart visualization types, Image placement areas, and TOC leader styling.
    - Safe actions footer: deletion with inline confirmation (`Konfirmasi Hapus? [Ya, Hapus] [Batal]`), contextual insertion (`Sisipkan Sebelum` / `Sisipkan Sesudah`), and saving.
