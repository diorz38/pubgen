# Orchestrator AI Translation (Indonesian → English) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the PGS Orchestrator translate a worksheet's Indonesian content into English as a pre-compile stage, with a persistent reviewable translation memory, guaranteed terminology/number preservation, and three selectable providers (Groq, custom OpenAI-compatible endpoint, Local AI).

**Architecture:** Translation moves *upstream* of layout. A new stage collects translatable units from Orchestrator cards, resolves them against a persisted translation memory (TM), masks glossary terms and numbers into placeholders, sends only TM misses to the selected provider, validates and restores the output, and writes English into the card fields and slot values that generation *already* consumes. Compile then emits a genuinely bilingual document instead of a bilingual layout with empty English.

**Tech Stack:** Rust (edition 2024, resolver 3), `egui`/`eframe` UI, `pubgen-core` (egui-free headless crate), `designcraft-translate` (`ureq` 2.12.1 + `serde_json`), `zip` for the `.pubgen` container, `chrono` for timestamps.

**Spec:** Design agreed in chat on 2026-11-08 (no separate spec file). Prior art: [`docs/plans/archives/2026-10-22-offline-ai-translation-pubgen.md`](archives/2026-10-22-offline-ai-translation-pubgen.md) and [`docs/plans/2028-09-26-future_plan_standalone_pubgen.md`](2028-09-26-future_plan_standalone_pubgen.md). User guide §13: [`docs/pubgen-guide.md:1048`](../pubgen-guide.md).

## Global Constraints

Copied from [`AGENTS.md`](../../AGENTS.md) — every task implicitly includes these.

- **Never crash.** No `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!`, `unimplemented!` in non-test code. No `unsafe` (`unsafe_code = "forbid"`). Exception only for a provably infallible literal, with `#[allow(clippy::expect_used)]` + `.expect("why it cannot fail")`.
- **Errors are `Result<T, E>`** through the crate's error type and `?`. An unfinished feature returns an "unsupported" error. Never `unwrap_or_default()` where a silent default would corrupt a document.
- **Input-derived numbers are hostile** (provider responses, CSV cells, TM files): use `get()` not `[i]`, slice strings at char boundaries, checked/saturating arithmetic, cap input-sized allocations.
- **Bound recursion.** Locks: `lock().unwrap_or_else(PoisonError::into_inner)`. Thread joins are `Result`s.
- **Every crash fix lands with a synthetic regression test that panicked before the fix.**
- New production crate roots declare `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]` before modules; `//!` docs may precede. New workspace crates include `workspace = true` under `[lints]`.
- **Layering:** nothing below L6 depends on egui/eframe/winit/rfd. `pubgen-core` and `designcraft-translate` stay egui-free. Verify with `cargo xtask layers`.
- **The UI is thin:** panels read state and act through `app.run(id, params)`. Colours from `theme::Tokens`.
- **Rust only.** No handwritten JS/TS. **Never break wasm** (`cargo xtask wasm`). All new network/thread code is `#[cfg(not(target_arch = "wasm32"))]` with a wasm stub that returns `Unavailable`.
- **Time goes through `chrono`** (workspace dependency). Never hand-roll calendar conversion.
- **Assets:** every new asset is original/OSI/CC0/redistributable CC and gets a row in `ASSETS.md`. No Adobe iconography. No new font files.
- **Clean room:** no GPL/AGPL/LGPL code. Check a dependency's licence before adding it (allowlist in `AGENTS.md`).
- **CI stays shared.** Do not add or change CI in a feature PR.
- **Quality gate before every commit:** `cargo xtask ci` (fmt, clippy `-D warnings`, tests, assets, layers, wasm). One task id per commit.
- **Secrets:** API keys live only in the `Authorization` header and local prefs. Never logged, never in an error string, never written into a document or a `.pubgen` package.
- **Parallel agents:** separate `CARGO_TARGET_DIR` per agent; edit only owned crates; delete the target dir when done.

---

## 1. Verified Current State

Read the code before changing it. These are the facts this plan argues from (all paths relative to repo root; **ignore the `0.8.4/` snapshot — it is a different, older architecture with no Orchestrator**).

### 1.1 The Orchestrator flow

| Stage | Location | Behaviour |
|---|---|---|
| Host tabs | `apps/pubgen-studio/src/workspaces/orchestrator/project.rs:93` | `OrchestratorTabs`, one tab per `.pubgen` project |
| Open / import | `project.rs:122`, `project.rs:137` | `open_pubgen` / `ingest_workbook` → `WorkspaceDraft::from_ingest` |
| Studio UI | `crates/ui-egui/src/panels/pubgen/workspace.rs` (5267 lines) | `OrchestratorView::{Home,Studio}`; center tabs `StrukturHalaman`, `VisualLivePreview`, `LogEksekusi`; right dock `SumberKonten`, `ProfilEksekusi` |
| Cards | `workspace.rs:39`, `workspace.rs:96` | 8 `CardKind`s; `PublicationCard` has `title`, `title_en`, `subtitle`, `source`, `settings: BTreeMap<String,String>` |
| Language | `workspace.rs:339`, `workspace.rs:725` | `doc_language` string → `pubgen_core::Language::{Id,En,IdEn}` |
| Compile | `workspace.rs:4287` | `execute_compile` → `generate_from_workspace` → switches to DesignCraft mode at `:4324` |
| Generation | `workspace.rs:4018` | **builds a fresh `PubgenState::default()` at `:4019`**, maps cards to catalog rows at `:4242-4250`, calls `excel::generate_publication_pages` at `:4284` |

### 1.2 Generation already consumes translations

This is the finding that makes the plan tractable. `row_template_slots` (`crates/ui-egui/src/panels/pubgen/pages.rs:362`) builds the slot value map and **already falls back to `st.translations`**:

- Divider/narrative title: explicit `title_en` wins, then `st.translations`, then a ` / ` pair (`pages.rs:409-446`).
- Table title: same precedence (`pages.rs:519-544`), emitting `table_title`, `table_title_en`, `table_num`, `table_kind_en`.
- Slot keys produced: `title`, `title#2`, `title_en`, `year`, `volume`, `catalogue`, `code`, `agency`, `chapter_num`, `chapter_title`, `chapter_title_en`, `narrative_body`, `note_body`, `note_title`, `chart_num`, `chart_title`, `chart_title_en`, `table_num`, `table_kind_en`, `table_title`, `table_title_en`, `number`, `source`.

The `#slot:<type>:<key>` labels live in the **template assets** (`assets/templates/{data_tables,dividers,front_matter,covers}`), authored by `#[test] fn test_generate_all_canonical_templates` (`pages.rs:4326`). The parser is `crates/engine/src/cmd/template/types.rs:47` (`parse_slot_tag`), documented at `types.rs:28-45`; it reads **both `label` and `name`**. Runtime binding is `template.page.apply`.

**Consequence:** no doc-format change and no new tagging scheme are needed for titles. Role+language tagging already exists as `*_en` slot keys. The plan only has to *populate* them earlier and extend the convention to table cells and narrative bodies.

### 1.3 Blockers (must fix first)

1. **Groq is not compiled into Studio.** `apps/pubgen-studio/Cargo.toml:20` enables only `features = ["pubgen"]`, and `crates/ui-egui/Cargo.toml:61` is `pubgen = []`. `ai-translate-online` (`ui-egui/Cargo.toml:64` → `designcraft-translate/groq`) is reachable only via `apps/designcraft/Cargo.toml:67`. In a standalone `pgs` build every translate call returns `TranslateError::Unavailable`.
2. **Compile discards translation state.** `generate_from_workspace` starts from `PubgenState::default()` (`workspace.rs:4019`), so `st.translations`, `st.glossary` and `st.translated_bodies` are empty on every compile. `st.translations` is `#[serde(skip)]` (`crates/ui-egui/src/panels/pubgen/state.rs:184`).
3. **`glossary.json` is always written empty.** `crates/pubgen-core/src/workspace_validation.rs:43` hardcodes `Glossary::default()`. The container persists it (`container.rs:62`) but nothing populates it.
4. **`domain/glossary.rs::enforce` is dead code.** `apps/pubgen-studio/src/domain/glossary.rs:43` — written and tested, zero callers.
5. **Studio never persists prefs.** No file under `apps/pubgen-studio/src` references prefs; `main.rs:81` is `Session::new()`. Only `apps/designcraft/src/main.rs:200-204`/`:219-220` reads/writes `prefs.json`. A custom endpoint config would be lost on restart.
6. **`download.rs` is an orphan.** `crates/translate/src/lib.rs:14-17` declares only `models`, `nllb`, `online`, `system`. The `download` feature (`crates/translate/Cargo.toml:30`) is never compiled.
7. **NLLB is a total stub.** `crates/translate/src/nllb.rs:16-29` returns `Unavailable` unconditionally. `ort = "=2.0.0-rc.15"` and `tokenizers = "0.21"` are declared optional (`Cargo.toml:16-17`) but no app enables the `nllb` feature.

### 1.4 Provider surface today

- Prefs (`crates/engine/src/lib.rs:193-284`, `#[serde(default, rename_all = "camelCase")]`): `translation_model_dir:263`, `translation_model_id:265` (default `"nllb600-int8"`), `translation_provider:267` (default `"offline"`), `translation_groq_api_key:270`, `translation_groq_model:272`, `pubgen_glossary_path:275`. All `String`.
- `prefs.set` (`crates/engine/src/cmd/prefs.rs:48-64`) merges via `merge` (`:96-117`), which **rejects unknown keys** but does not validate the provider string. Adding a field needs: struct + `Default` + the params doc at `:53`. `#[serde(default)]` means old `prefs.json` loads unchanged; no migration needed.
- `backend_from_prefs` (`ai.rs:1323-1336`) returns `Option<(String, String, String)>` = `(provider, key, model)` — **no slot for a base URL**. Callers: `ai.rs:429/447, 2142, 2161, 2197`; `panels/pubgen/mod.rs:157-159, 288-290`.
- `TranslateJob::spawn` (`ai.rs:540`) branches `if provider == "groq"` at `:553`; everything else → `Unavailable` at `:581-583`.
- Setup dialog: `PROVIDER_OFFLINE = 0`, `PROVIDER_GROQ = 1` (`ai.rs:19-20`); `AiSetupDialog` (`:30-49`); `groq_pane` (`:135-182`); `offline_pane` (`:184-237`); radios (`:477-480`); dispatch `if provider == PROVIDER_GROQ { groq_pane } else { offline_pane }` (`:483-487`) — **a third radio silently falls into the offline pane unless this becomes a `match`**.
- Client: `OnlineTranslator::new(key, model)` hardcodes `base_url = "https://api.groq.com/openai/v1"` (`online.rs:66`); `with_base` is `#[cfg(all(test, feature = "groq"))]` and private (`:70-75`). The facade `crates/translate/src/lib.rs:121-151` exposes no base-url path.

---

## 2. Design Decisions

### D1 — "Local AI" means a local OpenAI-compatible HTTP server

NLLB ONNX inference is a stub and `download.rs` is unreachable; implementing greedy decode + KV cache is a project of its own. **Local AI = Ollama / LM Studio / llama.cpp server on localhost**, reached through the same OpenAI-compatible client as Groq and Custom. Default base URL `http://localhost:11434/v1` (Ollama), empty API key allowed.

The existing NLLB pane stays but is relabelled honestly as unavailable. **Non-goal:** ONNX inference, model download, `ort` wiring. Do not delete `nllb.rs`/`models.rs` in this plan — out of scope, and `models.rs` tests still run.

### D2 — One client, three provider configs

All three providers speak OpenAI chat completions. Add a public endpoint-aware constructor rather than a second client. `ProviderKind` is a UI/prefs concept; the transport is shared.

### D3 — TM lives in the `.pubgen` package, not in prefs or `PubgenState`

Because `generate_from_workspace` builds a fresh `PubgenState`, anything stored there dies at compile. A dedicated `translation_memory.json` entry in the container is explicit, versioned and survives tab switches and restarts. `metadata.extra: BTreeMap<String,String>` (`crates/pubgen-core/src/workspace.rs:41`) would round-trip a JSON blob with zero container changes, but a large TM in a string map is the wrong shape — rejected.

**API keys never enter the TM or the package.** Provider config stays in prefs (and, for Studio, in a new local settings file — see Task 2).

### D4 — Mask, don't request

Glossary terms and numbers become `[[n]]` placeholders before the request and are restored after. Unmask **fails loudly** if a placeholder is missing, duplicated or reordered — never silently corrupt a statistical table. Indonesian numeric format (`1.234,56` — dot thousands, comma decimal) is preserved byte-for-byte because it is never sent to the model.

### D5 — Review is a gate, not a formality

AI output lands as `TmOrigin::Ai`. Compile may use it (so the user sees results immediately) but the review surface marks it clearly, and only a human action promotes it to `TmOrigin::Reviewed`. Reviewed entries are never overwritten by a later AI run.

### D6 — Narrative bodies: plain runs, not raw RTF

Translating RTF control words through an LLM is fragile, and the current `write_plain_rtf` (`ai.rs:1280-1283`) already drops inline formatting. Also `format_brs_narrative_story` (`pages.rs:628-690`) detects headings by **hardcoded Indonesian strings** (`:671-682`) which will not match English. **Phase 1 translates narrative titles only; body translation is Phase 5** and re-applies paragraph styles by index rather than re-detecting headings.

---

## 3. Per-Page-Type Translation Workflow

This is the core reference for implementers. `UnitRole` names below are defined in Task 5.

### 3.1 Collection matrix

| CardKind | Translated | Where EN is written | Skipped (never sent) |
|---|---|---|---|
| **Cover** | `title`, `cover_title2` | `card.title_en`, `settings["title_en"]`, `st.cover_title_en` (`state.rs:440`) → `covers.rs:1307-1334` | `year`, `volume`, `catalogue` (`Katalog BPS:<num>`), `code` (ISSN/ISBN), `release_no/date/month`, `pub_office`/`agency`, `office_address`, `pic_name/phone/email/photo` |
| **Narrative** | `title` (Phase 1); `body` paragraphs (Phase 5) | `settings["title_en"]` → `chapter_title_en` slot (`pages.rs:444`); body → `settings["file_en"]` staged RTF → `pages.rs:1682-1716` | `chapter_num` digit; RTF control words; style names |
| **Divider** | `title` (split into number + text) | `settings["title_en"]` → `chapter_title_en` (`pages.rs:409-446`) | leading digit run from `split_divider_title` (`pages.rs:380`) |
| **Table** | `title`; header cells; row-label column; note/source box | `settings["title_en"]` → `table_title_en` (`pages.rs:519-544`); cells → `st.translations` → `pair_cells` (`ai.rs:1140`); note → `translate_note_item` (`tables.rs:932-947`) | numeric cells (`translatable_cell`, `ai.rs:1039`); `(n)` column-numbering row; merged/spanned duplicates; `--` header stack separator |
| **Chart** | `title`; series labels; category labels | `settings["title_en"]` → `chart_title_en` (`pages.rs:500-508`); CSV `series_n_label_en` (`spec.rs:24,29-33`) | axis ticks, value labels (generated numbers); hardcoded bilingual pyramid strings (`emitter.rs:592-668`) |
| **MapChart** | `title` only | CSV `title_en` (`spec.rs:585,613`) → `pages.rs:2601-2644` | **region names** (proper nouns from GeoJSON `nmdesa`/`nmkec`, `layout.rs:1522`); classification breaks (numeric); legend scaffolding |
| **Image** | `title` (caption) | **new wiring required** — the Image branch (`pages.rs:474-483`) sets only `title`, no `title_en` slot exists | the image itself; `image_area` |
| **Toc** | contents/list title; entry titles come from the cards above | `toc_pair_bilingual` merges same-style ID/EN headings (`toc.rs:272`); `" / "` pair + "TOC English" char style (`toc.rs:1021-1038`) | page numbers; dot leaders; entries are **not** translated independently — they inherit from source cards |
| **Header/Footer** | `hf_header_*`, `hf_footer_*` static text | `expand_vars` already supports `{title_en}` → `dda_en` (`header_footer.rs:870-871`) | `{page}`, `{year}`, `{section}`, `{nopub}` variables; rules/offsets |

### 3.2 Universal skip rules

A unit is **not** sent to the provider when any of these hold. Implement once in `translation_units.rs` (Task 5), test each rule.

1. Empty or whitespace-only after trim.
2. Numeric-only: matches `^[\d\s.,%+\-–—/()]*$` (covers `1.234,56`, `(1)`, `2026`, `12,5%`).
3. An identifier: ISSN/ISBN/catalogue code, release number, email, phone, URL, file path.
4. A proper noun from the protected list (GeoJSON region names, agency/institution names, person names).
5. Already bilingual: contains ` / ` or `\u{2028}` and `locale::split_pair` yields both halves.
6. Present in TM as `Reviewed` or `Imported` — use the stored target, never re-request.
7. `do_not_translate` set on a matching glossary entry (`domain/glossary.rs:14`).

### 3.3 Stage sequence (runs before compile)

```
Orchestrator Studio
  └─ "Translate Worksheet" action
       1. collect_units(cards, cached_content)      → Vec<TransUnit>     [Task 5]
       2. apply skip rules (§3.2)                   → filtered units
       3. TM lookup (exact, then whitespace-normalized)
            hit  → reuse target, no network
            miss → continue
       4. glossary.lookup(whole string)             → deterministic EN, no network
       5. mask(text, glossary)                      → Masked             [Task 3]
       6. provider.translate(masked batch)          → Vec<String>        [Task 1]
       7. unmask + validate (placeholder integrity, number census)       [Task 3]
       8. TM insert as TmOrigin::Ai                                       [Task 4]
       9. write back: card.title_en / settings / staged _en content       [Task 7]
      10. execution_log + status; mark worksheet dirty
  └─ Review pane: accept / edit → TmOrigin::Reviewed                      [Task 8]
  └─ Compile → generate_from_workspace seeds st.translations from TM      [Task 7]
```

Failure at any step keeps everything already translated (the existing `JobMsg::Failed(err, partial)` pattern, `ai.rs:534-537`) and logs to `LogEksekusi`. A failed unit leaves its EN field empty — generation then renders Indonesian only for that unit, which is the current behaviour and is safe.

### 3.4 Worked example — Table card

Source card: `title = "Tabel 3.1 Luas Wilayah dan Jumlah Penduduk menurut Kecamatan"`, CSV header row `Kecamatan--Luas Wilayah (km2)--Jumlah Penduduk (Jiwa)`, row labels `Karawang Timur`, `Klari`, numeric cells, footer `Sumber: BPS Kabupaten Karawang`.

1. Title → one unit (`UnitRole::TableTitle`). Glossary masks `Kecamatan` → `[[0]]`, `Luas Wilayah` → `[[1]]`; `3.1` → `[[2]]`.
2. Header stacks split on `--` → three units (`TableHeader`); `(km2)` and `(Jiwa)` are unit annotations, kept verbatim via masking.
3. Row labels → `TableRowLabel` units, but **protected-list check first**: if `Karawang Timur` is a GeoJSON region name it is skipped as a proper noun.
4. Numeric cells → skipped by rule 2.
5. Footer → `TableNote`; `Sumber:` is a glossary term → `Source:`.
6. Results: `title_en = "Table 3.1 Total Area and Population by Subdistrict"`, header EN cells, `Source: BPS Karawang Regency`.
7. `pair_cells` (`ai.rs:1140`) joins ID/EN with `\u{2028}`; the EN half gets Italic (Bold Italic if the source was bold) per the existing table apply path (`ai.rs:1915-1951`).

---

## 4. File Structure

| File | Responsibility | Action |
|---|---|---|
| `crates/translate/src/online.rs` | OpenAI-compatible transport; endpoint-aware ctor, empty-key mode, per-endpoint rate-limit isolation | Modify |
| `crates/translate/src/lib.rs` | Facade: `ProviderConfig`, re-export endpoint ctor | Modify |
| `crates/translate/src/mask.rs` | Placeholder masking/unmasking + validation | **Create** |
| `crates/translate/Cargo.toml` | Neutral `online` feature; `groq = ["online"]` alias | Modify |
| `crates/pubgen-core/src/translation.rs` | `TranslationMemory`, `TmEntry`, `TmOrigin` | **Create** |
| `crates/pubgen-core/src/translation_units.rs` | `TransUnit`, `UnitRole`, collection + skip rules | **Create** |
| `crates/pubgen-core/src/container.rs` | Persist/read `translation_memory.json` (optional entry) | Modify |
| `crates/pubgen-core/src/workspace_validation.rs` | Stop hardcoding an empty glossary | Modify |
| `crates/pubgen-core/src/lib.rs` | Re-export new modules | Modify |
| `crates/engine/src/lib.rs` | New prefs fields + defaults | Modify |
| `crates/engine/src/cmd/prefs.rs` | Params doc for new fields | Modify |
| `crates/ui-egui/src/panels/pubgen/translate_stage.rs` | Orchestrator stage orchestration + write-back | **Create** |
| `crates/ui-egui/src/panels/pubgen/ai.rs` | Third provider pane; `match` dispatch; `backend_from_prefs` → config struct | Modify |
| `crates/ui-egui/src/panels/pubgen/workspace.rs` | Toolbar action, TM field on state, seed `st.translations` | Modify |
| `crates/ui-egui/src/panels/pubgen/mod.rs` | Register `translate_stage`; provider-agnostic quota reporting | Modify |
| `apps/pubgen-studio/src/settings.rs` | Persist provider config for Studio (no prefs file today) | **Create** |
| `apps/pubgen-studio/Cargo.toml` | Enable the online feature | Modify |
| `apps/pubgen-cli/Cargo.toml` + `src/main.rs` | `translate` subcommand | Modify |
| `apps/pubgen-studio/src/domain/glossary.rs` | Revive `enforce` as the deterministic layer | Modify (wire only) |

`workspace.rs` is already 5267 lines. **Do not grow it** beyond the toolbar hook, the state field and the seeding line — all stage logic goes in the new `translate_stage.rs`.

---

## 5. Tasks

Phases 1–2 ship independently of Phases 3–5. Each task ends in a commit and a green `cargo xtask ci`.

### Task 1: Enable the online backend in Studio and add an endpoint-aware client

**Files:**
- Modify: `apps/pubgen-studio/Cargo.toml:20`
- Modify: `crates/translate/Cargo.toml:23-30`
- Modify: `crates/ui-egui/Cargo.toml:64`
- Modify: `crates/translate/src/online.rs:48-75`
- Modify: `crates/translate/src/lib.rs:121-151`
- Test: `crates/translate/src/online.rs` (existing `mod tests`, `:356`)

**Interfaces:**
- Consumes: existing `OnlineTranslator`, `canned_server` test helper (`online.rs:445`).
- Produces:
  - `pub fn OnlineTranslator::with_endpoint(key: &str, model: &str, base_url: &str) -> Self`
  - `pub fn OnlineTranslator::new(key: &str, model: &str) -> Self` (unchanged signature, delegates with the Groq URL)
  - `pub fn designcraft_translate::OnlineTranslator::with_endpoint(key: &str, model: &str, base_url: &str) -> Self` on the facade
  - Cargo feature `designcraft-translate/online`; `groq = ["online"]`

- [ ] **Step 1: Write the failing test**

Add to `crates/translate/src/online.rs` inside `mod tests`:

```rust
#[test]
fn with_endpoint_targets_the_supplied_base_url() {
    let base = canned_server(
        "{\"choices\":[{\"message\":{\"content\":\"[\\\"Regency\\\"]\"}}]}".to_string(),
    );
    let mut t = OnlineTranslator::with_endpoint("", "local-model", &base);
    let cancel = AtomicBool::new(false);
    let out = t
        .translate(&["Kabupaten".into()], Lang::Indonesian, Lang::English, None, &cancel, &mut |_, _| {})
        .expect("local endpoint round trip");
    assert_eq!(out, vec!["Regency"]);
}

#[test]
fn empty_key_sends_no_authorization_header() {
    // A local Ollama/LM Studio server has no key. The request must still succeed,
    // and the client must not send "Bearer " with an empty token.
    let base = canned_server(
        "{\"choices\":[{\"message\":{\"content\":\"[\\\"Subdistrict\\\"]\"}}]}".to_string(),
    );
    let mut t = OnlineTranslator::with_endpoint("   ", "m", &base);
    let cancel = AtomicBool::new(false);
    assert!(t
        .translate(&["Kecamatan".into()], Lang::Indonesian, Lang::English, None, &cancel, &mut |_, _| {})
        .is_ok());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p designcraft-translate --features online with_endpoint -- --nocapture`
Expected: FAIL — `cannot find function with_endpoint` (compile error).

- [ ] **Step 3: Add the neutral feature**

In `crates/translate/Cargo.toml`, replace the feature block so `groq` stays as an alias:

```toml
[features]
default = []
# OpenAI-compatible HTTP transport (Groq, custom endpoints, local servers).
online = ["dep:ureq", "dep:serde_json"]
# Backwards-compatible alias: existing callers ask for `groq`.
groq = ["online"]
nllb = ["dep:ort", "dep:tokenizers"]
download = ["online", "dep:sha2"]
```

Keep every `#[cfg(feature = "groq")]` in `online.rs` working by changing those attributes to `#[cfg(feature = "online")]` throughout the file. In `crates/ui-egui/Cargo.toml:64`:

```toml
ai-translate-online = ["designcraft-translate/online"]
```

In `apps/pubgen-studio/Cargo.toml:20`:

```toml
designcraft-ui-egui = { workspace = true, features = ["pubgen", "ai-translate-online"] }
```

- [ ] **Step 4: Implement the endpoint constructor and empty-key mode**

In `crates/translate/src/online.rs`, replace `new`/`with_base` (`:59-75`):

```rust
impl OnlineTranslator {
    /// Groq's hosted endpoint. Kept so existing callers are unchanged.
    pub fn new(key: &str, model: &str) -> Self {
        Self::with_endpoint(key, model, "https://api.groq.com/openai/v1")
    }

    /// Any OpenAI-compatible endpoint: Groq, a custom host, or a local server
    /// (Ollama, LM Studio, llama.cpp). An empty key means no Authorization header.
    pub fn with_endpoint(key: &str, model: &str, base_url: &str) -> Self {
        Self {
            key: key.trim().to_string(),
            model: model.trim().to_string(),
            #[cfg(feature = "online")]
            agent: ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(30)).build(),
            #[cfg(feature = "online")]
            base_url: base_url.trim().trim_end_matches('/').to_string(),
        }
    }

    #[cfg(all(test, feature = "online"))]
    fn with_base(key: &str, model: &str, base_url: String) -> Self {
        Self::with_endpoint(key, model, &base_url)
    }
}
```

Add a helper and use it at both header sites (`list_models` `:81`, `post_chat` `:228`):

```rust
#[cfg(feature = "online")]
fn auth_header(&self) -> Option<String> {
    // Local servers run without a key; "Bearer " with an empty token is rejected
    // by some of them and is never useful.
    (!self.key.is_empty()).then(|| format!("Bearer {}", self.key))
}
```

Replace `.set("Authorization", &format!("Bearer {}", self.key))` with a conditional set on the request builder in both places.

Isolate the rate-limit counter so a local server cannot clobber Groq's number — change `store_remaining` (`:31`) to take the base URL and only store for Groq:

```rust
#[cfg(feature = "online")]
fn store_remaining(r: &ureq::Response, base_url: &str) {
    if !base_url.starts_with("https://api.groq.com") {
        return;
    }
    if let Some(v) = r.header("x-ratelimit-remaining-requests").and_then(|v| v.trim().parse::<u32>().ok()) {
        GROQ_REMAINING_REQUESTS.store(v, Ordering::Relaxed);
    }
}
```

Update both call sites (`:233`, `:237`) to pass `&self.base_url`.

In `crates/translate/src/lib.rs`, add the facade passthrough inside `impl OnlineTranslator` (`:125-151`):

```rust
    /// Any OpenAI-compatible endpoint (custom host or local server).
    pub fn with_endpoint(key: &str, model: &str, base_url: &str) -> Self {
        Self { inner: online::OnlineTranslator::with_endpoint(key, model, base_url) }
    }
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p designcraft-translate --features online`
Expected: PASS — all pre-existing tests plus the two new ones.

Run: `cargo build -p pgs`
Expected: builds; `designcraft_translate::online_available()` is now `true` in a Studio build.

- [ ] **Step 6: Commit**

```bash
git add apps/pubgen-studio/Cargo.toml crates/translate/Cargo.toml crates/translate/src/online.rs crates/translate/src/lib.rs crates/ui-egui/Cargo.toml
git commit -m "T1: endpoint-aware translation client and online backend for Studio"
```

---

### Task 2: Provider config (Groq / Custom / Local) in prefs and Studio settings

**Files:**
- Modify: `crates/engine/src/lib.rs:263-275` (fields), `:413-418` (defaults)
- Modify: `crates/engine/src/cmd/prefs.rs:53` (params doc)
- Create: `apps/pubgen-studio/src/settings.rs`
- Modify: `apps/pubgen-studio/src/main.rs` (load/save settings)
- Test: `crates/engine/src/cmd/prefs.rs` (existing test module), `apps/pubgen-studio/src/settings.rs`

**Interfaces:**
- Consumes: `crate::Prefs`, `prefs.set` merge (`prefs.rs:96-117`).
- Produces:
  - Prefs fields `translation_custom_base_url`, `translation_custom_api_key`, `translation_custom_model`, `translation_local_base_url`, `translation_local_model` (all `String`)
  - `translation_provider` accepted values: `"groq" | "custom" | "local" | "offline"` (legacy NLLB)
  - `pub struct ProviderSettings { pub provider: String, pub groq_key: String, pub groq_model: String, pub custom_base_url: String, pub custom_key: String, pub custom_model: String, pub local_base_url: String, pub local_model: String }`
  - `pub fn ProviderSettings::load() -> Self` / `pub fn save(&self) -> Result<(), String>` (Studio, JSON at the platform config dir)
  - `pub const DEFAULT_LOCAL_BASE_URL: &str = "http://localhost:11434/v1";`

- [ ] **Step 1: Write the failing test**

In `crates/engine/src/cmd/prefs.rs` tests:

```rust
#[test]
fn prefs_set_accepts_the_custom_provider_fields() {
    let mut s = Session::new();
    let r = s
        .execute(
            "prefs.set",
            &json!({
                "translationProvider": "custom",
                "translationCustomBaseUrl": "https://api.example.com/v1",
                "translationCustomApiKey": "k",
                "translationCustomModel": "gpt-4o-mini",
                "translationLocalBaseUrl": "http://localhost:11434/v1",
                "translationLocalModel": "qwen2.5:7b"
            }),
        )
        .expect("prefs.set accepts the new keys");
    assert_eq!(r["translationProvider"], "custom");
    assert_eq!(s.prefs.translation_provider, "custom");
    assert_eq!(s.prefs.translation_custom_base_url, "https://api.example.com/v1");
    assert_eq!(s.prefs.translation_local_model, "qwen2.5:7b");
}

#[test]
fn prefs_defaults_keep_the_legacy_provider() {
    let p = crate::Prefs::default();
    assert_eq!(p.translation_provider, "offline");
    assert!(p.translation_custom_base_url.is_empty());
    assert_eq!(p.translation_local_base_url, "http://localhost:11434/v1");
}
```

In `apps/pubgen-studio/src/settings.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_without_a_key_in_the_default_file() {
        let dir = std::env::temp_dir().join(format!("pgs-settings-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("settings.json");
        let s = ProviderSettings { provider: "local".into(), local_model: "qwen2.5:7b".into(), ..Default::default() };
        s.save_to(&path).expect("save");
        let back = ProviderSettings::load_from(&path);
        assert_eq!(back.provider, "local");
        assert_eq!(back.local_model, "qwen2.5:7b");
        assert_eq!(back.local_base_url, DEFAULT_LOCAL_BASE_URL);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_missing_settings_file_yields_defaults() {
        let path = std::env::temp_dir().join(format!("pgs-absent-{}.json", std::process::id()));
        let s = ProviderSettings::load_from(&path);
        assert_eq!(s.provider, "groq");
        assert!(!path.exists(), "loading must not create the file");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p designcraft-engine prefs_set_accepts_the_custom -- --nocapture`
Expected: FAIL — unknown key rejected by `merge`, field does not exist.

Run: `cargo test -p pgs settings_round_trip -- --nocapture`
Expected: FAIL — `ProviderSettings` not defined.

- [ ] **Step 3: Add the prefs fields**

In `crates/engine/src/lib.rs` after `:272`:

```rust
    /// Base URL of a custom OpenAI-compatible endpoint (no trailing slash).
    #[serde(default)]
    pub translation_custom_base_url: String,
    /// Key for the custom endpoint. Header only; never logged or exported.
    #[serde(default)]
    pub translation_custom_api_key: String,
    #[serde(default)]
    pub translation_custom_model: String,
    /// Local OpenAI-compatible server (Ollama, LM Studio, llama.cpp).
    #[serde(default = "default_local_base_url")]
    pub translation_local_base_url: String,
    #[serde(default)]
    pub translation_local_model: String,
```

Add near the Default impl:

```rust
fn default_local_base_url() -> String {
    "http://localhost:11434/v1".to_string()
}
```

And in `impl Default for Prefs` after `:417`:

```rust
            translation_custom_base_url: String::new(),
            translation_custom_api_key: String::new(),
            translation_custom_model: String::new(),
            translation_local_base_url: default_local_base_url(),
            translation_local_model: String::new(),
```

Update the params doc at `crates/engine/src/cmd/prefs.rs:53` to list the new camelCase keys and the four provider values.

- [ ] **Step 4: Create Studio settings persistence**

`apps/pubgen-studio/src/settings.rs`:

```rust
//! Provider settings for Studio. `pgs` does not read or write DesignCraft's
//! prefs.json, so without this a configured endpoint is lost on restart.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_LOCAL_BASE_URL: &str = "http://localhost:11434/v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProviderSettings {
    pub provider: String,
    pub groq_key: String,
    pub groq_model: String,
    pub custom_base_url: String,
    pub custom_key: String,
    pub custom_model: String,
    pub local_base_url: String,
    pub local_model: String,
}

impl Default for ProviderSettings {
    fn default() -> Self {
        Self {
            provider: "groq".into(),
            groq_key: String::new(),
            groq_model: "llama-3.3-70b-versatile".into(),
            custom_base_url: String::new(),
            custom_key: String::new(),
            custom_model: String::new(),
            local_base_url: DEFAULT_LOCAL_BASE_URL.into(),
            local_model: String::new(),
        }
    }
}

impl ProviderSettings {
    /// Platform config dir from environment variables (no new dependency);
    /// falls back to the current dir when none is set.
    pub fn path() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("pubgen-studio").join("settings.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::path())
    }

    /// A missing or malformed file is not an error: the user gets defaults and
    /// configures a provider in the setup dialog.
    pub fn load_from(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::path())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("settings dir: {e}"))?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(|e| format!("settings encode: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("settings write: {e}"))
    }
}
```

`dirs_next` is **not** a workspace dependency (verified) — the env-var lookup above avoids adding one. Declare the module in `apps/pubgen-studio/src/main.rs`, load it into the app state, and save it when the setup dialog is accepted.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p designcraft-engine prefs_ && cargo test -p pgs settings_`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/engine/src/lib.rs crates/engine/src/cmd/prefs.rs apps/pubgen-studio/src/settings.rs apps/pubgen-studio/src/main.rs apps/pubgen-studio/Cargo.toml
git commit -m "T2: Groq/custom/local provider config with Studio persistence"
```

---

### Task 3: Masking and validation

**Files:**
- Create: `crates/translate/src/mask.rs`
- Modify: `crates/translate/src/lib.rs:14-17` (declare `pub mod mask;`)
- Test: `crates/translate/src/mask.rs`

**Interfaces:**
- Consumes: nothing (pure text).
- Produces:
  - `pub enum TokenKind { Number, Glossary }`
  - `pub struct Token { pub index: usize, pub original: String, pub replacement: Option<String>, pub kind: TokenKind }`
  - `pub struct Masked { pub text: String, pub tokens: Vec<Token> }`
  - `pub fn mask(text: &str, glossary: &[(String, String)]) -> Masked`
  - `pub enum MaskError { Missing(usize), Duplicated(usize), Reordered }` (implements `std::fmt::Display` + `std::error::Error`)
  - `pub fn unmask(output: &str, masked: &Masked) -> Result<String, MaskError>`
  - `pub fn number_census(text: &str) -> Vec<String>` — every numeric token, for post-translation comparison

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn glossary() -> Vec<(String, String)> {
        vec![("Kecamatan".into(), "Subdistrict".into()), ("Luas Wilayah".into(), "Total Area".into())]
    }

    #[test]
    fn masks_glossary_and_indonesian_numbers() {
        let m = mask("Luas Wilayah Kecamatan Karawang Timur adalah 1.234,56 km2.", &glossary());
        assert!(!m.text.contains("1.234,56"), "the number must not reach the model: {}", m.text);
        assert!(!m.text.contains("Kecamatan"));
        assert!(m.text.contains("[[0]]"));
        let out = m.text.replace("adalah", "is").replace("Km2", "km2");
        // Simulate the model returning the placeholders untouched.
        let restored = unmask(&out, &m).expect("placeholders intact");
        assert!(restored.contains("Subdistrict"), "{restored}");
        assert!(restored.contains("Total Area"), "{restored}");
        assert!(restored.contains("1.234,56"), "{restored}");
    }

    #[test]
    fn longest_glossary_match_wins() {
        let g = vec![("Wilayah".into(), "Region".into()), ("Luas Wilayah".into(), "Total Area".into())];
        let m = mask("Luas Wilayah", &g);
        let restored = unmask(&m.text, &m).expect("intact");
        assert_eq!(restored, "Total Area", "the shorter term must not win");
    }

    #[test]
    fn missing_placeholder_is_an_error_not_a_silent_default() {
        let m = mask("Kecamatan Karawang 1.234,56", &glossary());
        let broken = m.text.replace("[[0]]", "Subdistrict");
        let err = unmask(&broken, &m).err().expect("must fail");
        assert!(matches!(err, MaskError::Missing(_)), "{err}");
    }

    #[test]
    fn an_invented_placeholder_is_an_error() {
        let m = mask("Kecamatan A dan Kecamatan B", &glossary());
        // Both occurrences share one token index by design; a model inventing an
        // extra copy must be caught.
        let broken = format!("{} [[9]]", m.text);
        assert!(unmask(&broken, &m).is_err());
    }

    #[test]
    fn unknown_placeholder_is_an_error() {
        let m = mask("Kecamatan", &glossary());
        let broken = format!("{} [[42]]", m.text);
        assert!(unmask(&broken, &m).is_err());
    }

    #[test]
    fn number_census_preserves_indonesian_format() {
        assert_eq!(number_census("Total 1.234,56 jiwa (2026) 12,5%"), vec!["1.234,56", "2026", "12,5"]);
    }

    #[test]
    fn empty_glossary_and_plain_text_are_unchanged() {
        let m = mask("Halo dunia.", &[]);
        assert_eq!(m.text, "Halo dunia.");
        assert!(m.tokens.is_empty());
        assert_eq!(unmask(&m.text, &m).expect("intact"), "Halo dunia.");
    }

    #[test]
    fn masking_is_char_boundary_safe() {
        let m = mask("Kecamatan — élève 日本語 1.000", &glossary());
        assert!(unmask(&m.text, &m).is_ok());
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p designcraft-translate mask:: -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement `mask.rs`**

Write the module with these exact behaviours:

- Placeholder syntax `[[n]]`, `n` a zero-based token index.
- Glossary matching: sort terms by descending char length; match case-insensitively at word boundaries (`char::is_alphanumeric() || '_'`), reusing the boundary logic already proven in `apps/pubgen-studio/src/domain/glossary.rs:19-21`. Each distinct matched term gets one token; repeated occurrences reuse the same index. `Token.replacement = Some(official_en)`.
- Number matching: after glossary masking, scan the remaining text for runs of `[\d.,]` optionally followed by `%`, with at least one digit. Keep the run verbatim (`1.234,56`, `2026`, `12,5`). `Token.replacement = None` (restore the original).
- Never split a `char`; operate on `char_indices` and collect via `chars()`.
- Cap tokens at 512 per unit; beyond that, leave the remainder unmasked and record nothing (a unit that long is a narrative paragraph, handled by `segment()`).
- `unmask`: find every `[[n]]` in the output in order; require the index sequence to be non-decreasing and each expected index present at least once; reject unknown indices. Substitute `replacement` when `Some`, else `original`. Return `MaskError` on any violation — **never** fall back to the raw model text.
- `number_census`: return the numeric runs in order, without `%`, for comparing source and restored output.

Declare `pub mod mask;` in `crates/translate/src/lib.rs` and re-export `pub use mask::{mask, unmask, Masked, MaskError};`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p designcraft-translate`
Expected: PASS (no feature needed — masking is pure text and always compiled).

- [ ] **Step 5: Commit**

```bash
git add crates/translate/src/mask.rs crates/translate/src/lib.rs
git commit -m "T3: placeholder masking with strict unmask validation"
```

---

### Task 4: Translation memory in pubgen-core and the `.pubgen` container

**Files:**
- Create: `crates/pubgen-core/src/translation.rs`
- Modify: `crates/pubgen-core/src/lib.rs:7-30` (re-export)
- Modify: `crates/pubgen-core/src/container.rs:43-130`
- Modify: `crates/pubgen-core/src/workspace_validation.rs:43`
- Test: `crates/pubgen-core/src/translation.rs`, `crates/pubgen-core/src/container.rs`

**Interfaces:**
- Consumes: `chrono::Utc`, existing `save_pubgen`/`open_pubgen`, `WorkspaceDraft`.
- Produces:
  - `pub enum TmOrigin { Ai, Reviewed, Imported }` (serde `snake_case`)
  - `pub struct TmEntry { pub source: String, pub target: String, pub origin: TmOrigin, pub role: String, pub updated: String }`
  - `pub struct TranslationMemory { pub entries: Vec<TmEntry> }` (`Default`, `Serialize`, `Deserialize`)
  - `impl TranslationMemory { pub fn lookup(&self, source: &str) -> Option<&TmEntry>; pub fn insert(&mut self, entry: TmEntry) -> bool; pub fn promote(&mut self, source: &str) -> bool; pub fn update_target(&mut self, source: &str, target: &str) -> bool; pub fn pending_review(&self) -> Vec<&TmEntry>; pub fn to_map(&self) -> std::collections::BTreeMap<String, String> }`
  - `pub fn now_stamp() -> String` (chrono, RFC 3339)
  - `save_pubgen` gains a `tm: &TranslationMemory` parameter; `OpenedProject` gains `pub tm: TranslationMemory`
  - `WorkspaceDraft` gains `pub tm: TranslationMemory`

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn entry(src: &str, tgt: &str, origin: TmOrigin) -> TmEntry {
        TmEntry { source: src.into(), target: tgt.into(), origin, role: "table_title".into(), updated: now_stamp() }
    }

    #[test]
    fn lookup_is_whitespace_normalized() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("Luas  Wilayah", "Total Area", TmOrigin::Reviewed));
        assert_eq!(tm.lookup("Luas Wilayah").map(|e| e.target.as_str()), Some("Total Area"));
    }

    #[test]
    fn a_reviewed_entry_is_never_overwritten_by_ai() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("Kecamatan", "Subdistrict", TmOrigin::Reviewed));
        let changed = tm.insert(entry("Kecamatan", "District", TmOrigin::Ai));
        assert!(!changed, "AI must not clobber a reviewed term");
        assert_eq!(tm.lookup("Kecamatan").map(|e| e.target.as_str()), Some("Subdistrict"));
    }

    #[test]
    fn ai_may_refresh_an_older_ai_entry() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("Kecamatan", "District", TmOrigin::Ai));
        assert!(tm.insert(entry("Kecamatan", "Subdistrict", TmOrigin::Ai)));
        assert_eq!(tm.lookup("Kecamatan").map(|e| e.target.as_str()), Some("Subdistrict"));
    }

    #[test]
    fn promote_moves_ai_to_reviewed() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("PDRB", "GRDP", TmOrigin::Ai));
        assert_eq!(tm.pending_review().len(), 1);
        assert!(tm.promote("PDRB"));
        assert!(tm.pending_review().is_empty());
        assert!(!tm.promote("PDRB"), "promoting twice is a no-op");
    }

    #[test]
    fn update_target_marks_the_entry_reviewed() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("PDRB", "GDP", TmOrigin::Ai));
        assert!(tm.update_target("PDRB", "GRDP"));
        assert_eq!(tm.lookup("PDRB").map(|e| e.origin), Some(TmOrigin::Reviewed));
    }

    #[test]
    fn to_map_feeds_the_generation_cache() {
        let mut tm = TranslationMemory::default();
        tm.insert(entry("Kecamatan", "Subdistrict", TmOrigin::Ai));
        tm.insert(entry("PDRB", "GRDP", TmOrigin::Reviewed));
        let map = tm.to_map();
        assert_eq!(map.get("Kecamatan").map(String::as_str), Some("Subdistrict"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn empty_source_is_rejected() {
        let mut tm = TranslationMemory::default();
        assert!(!tm.insert(entry("   ", "x", TmOrigin::Ai)));
        assert!(tm.entries.is_empty());
    }
}
```

In `crates/pubgen-core/src/container.rs` tests:

```rust
fn meta(title: &str) -> PublicationMetadata {
    PublicationMetadata {
        profile_id: "bps_dda".into(),
        layout: PageLayout::Single,
        language: Language::Id,
        title: title.into(),
        title_en: None,
        cards: Vec::new(),
        extra: Default::default(),
    }
}

#[test]
fn tm_round_trips_through_the_package() {
    let dir = std::env::temp_dir().join(format!("pubgen-tm-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("tm.pubgen");
    let manifest = ProjectManifest { format_version: 1, name: "TM test".into(), template: false, slots: Vec::new() };
    let mut tm = TranslationMemory::default();
    tm.insert(TmEntry { source: "Kecamatan".into(), target: "Subdistrict".into(), origin: TmOrigin::Reviewed, role: "table_header".into(), updated: now_stamp() });
    save_pubgen(&path, &manifest, &meta("TM test"), &Glossary::default(), &tm, &ProjectContent::default()).expect("save");
    let opened = open_pubgen(&path).expect("open");
    assert_eq!(opened.tm.lookup("Kecamatan").map(|e| e.target.as_str()), Some("Subdistrict"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_package_without_a_tm_entry_opens_with_an_empty_tm() {
    // Backwards compatibility: packages written before this change have no
    // translation_memory.json and must still open.
    let dir = std::env::temp_dir().join(format!("pubgen-notm-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("old.pubgen");
    let manifest = ProjectManifest { format_version: 1, name: "Old".into(), template: false, slots: Vec::new() };
    save_pubgen(&path, &manifest, &meta("Old"), &Glossary::default(), &TranslationMemory::default(), &ProjectContent::default()).expect("save");
    // Remove the entry to simulate an older package.
    rewrite_zip_without(&path, "translation_memory.json");
    let opened = open_pubgen(&path).expect("old package still opens");
    assert!(opened.tm.entries.is_empty());
    std::fs::remove_dir_all(&dir).ok();
}
```

`PublicationMetadata` has **no** `Default` (verified: `#[derive(Debug, Clone, Serialize, Deserialize)]` at `workspace.rs:28`), hence the explicit `meta()` helper above. Add a `rewrite_zip_without(path, name)` test helper that reads the archive and rewrites it omitting one entry.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pubgen-core translation:: -- --nocapture`
Expected: FAIL — module missing.

- [ ] **Step 3: Implement the TM type**

Create `crates/pubgen-core/src/translation.rs` with the crate-root lint attributes already used by neighbouring modules. Behaviours:

- `lookup`: trim + collapse internal whitespace runs to one space on both sides; compare case-sensitively first, then case-insensitively as a fallback. Return the `Reviewed`/`Imported` entry in preference to an `Ai` one when both exist.
- `insert`: reject empty/whitespace source or target. If an existing entry is `Reviewed` or `Imported` and the new one is `Ai`, return `false` without changing anything. Otherwise replace and return `true`.
- `promote`: `Ai` → `Reviewed`, refresh `updated`; return `false` if absent or already reviewed.
- `update_target`: set target, set origin `Reviewed`, refresh `updated`; `false` if absent.
- `to_map`: every entry with a non-empty target, keyed by the *normalized* source. This is what seeds `st.translations`.
- `now_stamp`: `chrono::Utc::now().to_rfc3339()`.
- Cap entries at 100_000 on insert; beyond that return `false` (a hostile or corrupt file must not grow without bound).

- [ ] **Step 4: Persist it in the container**

In `crates/pubgen-core/src/container.rs`:

- `save_pubgen`: add `tm: &TranslationMemory` after `glossary`; `reserve_name(&mut names, "translation_memory.json")?;` then `write_json(&mut writer, "translation_memory.json", tm, options)?;`.
- `open_pubgen`: add `translation_memory.json` to the skip list at `:105`. Read it **optionally** — a helper `read_json_opt` returning `Ok(None)` when the entry is absent, so old packages open with an empty TM. Add `pub tm: TranslationMemory` to `OpenedProject`.
- Update every `save_pubgen` call site (verified, three exist): `apps/pubgen-studio/src/workspaces/orchestrator/project.rs:44` (pass `&draft.tm`), the test helper at `project.rs:353`, and `crates/ui-egui/src/panels/pubgen/compile.rs:2053` (pass `&TranslationMemory::default()` here for now — the Orchestrator state gains its `tm` field only in Task 7, which switches this call to `&orch.tm`). `excel_export.rs` does not call it.
- Add `pub tm: TranslationMemory` to `WorkspaceDraft` (`workspace_validation.rs:20-31`) and initialize it in `from_ingest` (`:34`); also set `tm: opened.tm` in `OrchestratorProject::open` (`project.rs:23-37`).
- `IngestReport` (verified: `excel.rs:21-30`) has no glossary field — leave `from_ingest`'s `Glossary::default()` in place here; Task 8 populates the glossary at the ui-egui boundary instead. Do not invent a field.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p pubgen-core && cargo test -p pgs`
Expected: PASS, including the existing tab tests in `project.rs:326-478` (update their `save_pubgen` helpers for the new parameter).

- [ ] **Step 6: Commit**

```bash
git add crates/pubgen-core/src/translation.rs crates/pubgen-core/src/lib.rs crates/pubgen-core/src/container.rs crates/pubgen-core/src/workspace_validation.rs apps/pubgen-studio/src/workspaces/orchestrator/project.rs
git commit -m "T4: persisted translation memory in the .pubgen container"
```

---

### Task 5: Unit collection and skip rules

**Files:**
- Create: `crates/pubgen-core/src/translation_units.rs`
- Modify: `crates/pubgen-core/src/lib.rs` (re-export)
- Test: `crates/pubgen-core/src/translation_units.rs`

**Interfaces:**
- Consumes: `WorkspaceDraft`, `PublicationCard`, `CardKind`, `TranslationMemory`, CSV bytes from `draft.csv`, text bytes from `draft.text`.
- Produces:
  - `pub enum UnitRole { CardTitle, CoverTitle, TableTitle, TableHeader, TableRowLabel, TableNote, NarrativeTitle, NarrativeBody, ChartTitle, ChartSeries, MapTitle, ImageCaption, TocTitle, DividerTitle, HeaderFooter }` (serde `snake_case`, `as_str() -> &'static str`)
  - `pub struct TransUnit { pub key: String, pub text: String, pub role: UnitRole, pub card_id: String }`
  - `pub fn collect_units(draft: &WorkspaceDraft) -> Vec<TransUnit>`
  - `pub fn is_skippable(text: &str, protected: &[String]) -> bool`
  - `pub fn protected_names(draft: &WorkspaceDraft) -> Vec<String>`
  - `pub fn split_header_stacks(cell: &str) -> Vec<String>` (splits on `--`, max 3, matching `tables.rs:131-149`)

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// `WorkspaceDraft` has no `Default` (verified: `#[derive(Debug, Clone)]`,
    /// `workspace_validation.rs:21`), so tests build it explicitly.
    fn draft() -> WorkspaceDraft {
        WorkspaceDraft {
            manifest: pubgen_core::ProjectManifest { format_version: 1, name: "test".into(), template: false, slots: Vec::new() },
            metadata: pubgen_core::PublicationMetadata {
                profile_id: "bps_dda".into(),
                layout: pubgen_core::PageLayout::Single,
                language: pubgen_core::Language::Id,
                title: "test".into(),
                title_en: None,
                cards: Vec::new(),
                extra: Default::default(),
            },
            glossary: pubgen_core::Glossary::default(),
            csv: Vec::new(),
            text: Vec::new(),
            assets: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn numeric_cells_are_skipped() {
        for n in ["1.234,56", "2026", "(1)", "12,5%", "-", "1 234", "0", "3.1"] {
            assert!(is_skippable(n, &[]), "must skip {n}");
        }
    }

    #[test]
    fn identifiers_are_skipped() {
        for s in ["ISSN 2089-1234", "Katalog BPS: 1101001", "someone@bps.go.id", "0812-3456-7890", "https://example.com", "csv/T1.csv"] {
            assert!(is_skippable(s, &[]), "must skip {s}");
        }
    }

    #[test]
    fn protected_proper_nouns_are_skipped() {
        let protected = vec!["Karawang Timur".to_string()];
        assert!(is_skippable("Karawang Timur", &protected));
        assert!(is_skippable("karawang  timur", &protected), "normalised match");
        assert!(!is_skippable("Luas Wilayah", &protected));
    }

    #[test]
    fn already_bilingual_text_is_skipped() {
        assert!(is_skippable("Geografi / Geography", &[]));
        assert!(is_skippable("Geografi\u{2028}Geography", &[]));
    }

    #[test]
    fn real_indonesian_labels_are_collected() {
        for s in ["Luas Wilayah menurut Kecamatan", "Jumlah Penduduk", "Sumber: BPS Kabupaten Karawang", "Ringkasan Poin Indikator Utama"] {
            assert!(!is_skippable(s, &[]), "must collect {s}");
        }
    }

    #[test]
    fn empty_and_whitespace_are_skipped() {
        assert!(is_skippable("", &[]));
        assert!(is_skippable("   ", &[]));
    }

    #[test]
    fn header_stacks_split_on_double_dash() {
        assert_eq!(split_header_stacks("Kecamatan--Luas Wilayah--Jumlah Penduduk"), vec!["Kecamatan", "Luas Wilayah", "Jumlah Penduduk"]);
        assert_eq!(split_header_stacks("Kecamatan"), vec!["Kecamatan"]);
    }

    #[test]
    fn cover_identifiers_are_not_collected() {
        let mut draft = draft();
        draft.metadata.cards.push(card(CardKind::Cover, "Perkembangan NTP Oktober 2026", &[
            ("cover_year", "2026"), ("cover_catalogue", "Katalog BPS: 1101001"),
            ("cover_code_num", "ISSN 2089-1234"), ("cover_pic_name", "Budi Santoso"),
            ("cover_office_address", "Jl. PHH Mustofa No. 1"),
        ]));
        let units = collect_units(&draft);
        let texts: Vec<&str> = units.iter().map(|u| u.text.as_str()).collect();
        assert!(texts.iter().any(|t| t.contains("Perkembangan NTP")), "{texts:?}");
        for forbidden in ["2026", "Katalog BPS: 1101001", "ISSN 2089-1234", "Budi Santoso", "Jl. PHH Mustofa No. 1"] {
            assert!(!texts.contains(&forbidden), "must not collect {forbidden}: {texts:?}");
        }
    }

    #[test]
    fn map_region_names_are_protected_but_the_title_is_collected() {
        let mut draft = draft();
        draft.csv.push(("csv/M1.csv".into(), b"nmdesa;title;title_en;value\nKarawang Timur;Peta Kepadatan;;120\n".to_vec()));
        draft.metadata.cards.push(card(CardKind::MapChart, "Peta Kepadatan Penduduk", &[("source", "csv/M1.csv")]));
        let protected = protected_names(&draft);
        assert!(protected.iter().any(|p| p == "Karawang Timur"));
        let units = collect_units(&draft);
        assert!(units.iter().any(|u| u.role == UnitRole::MapTitle));
        assert!(!units.iter().any(|u| u.text == "Karawang Timur"));
    }

    #[test]
    fn units_are_deduplicated_by_key() {
        let mut draft = draft();
        draft.metadata.cards.push(card(CardKind::Table, "Luas Wilayah", &[]));
        draft.metadata.cards.push(card(CardKind::Table, "Luas Wilayah", &[]));
        let units = collect_units(&draft);
        assert_eq!(units.iter().filter(|u| u.text == "Luas Wilayah").count(), 1);
    }

    fn card(kind: CardKind, title: &str, settings: &[(&str, &str)]) -> pubgen_core::PublicationCard {
        pubgen_core::PublicationCard {
            id: format!("{kind:?}-{title}"),
            kind,
            scope: pubgen_core::CardScope::NewPage,
            source: None,
            title: title.into(),
            settings: settings.iter().map(|(k, v)| ((*k).to_string(), serde_json::json!(v))).collect(),
        }
    }
}
```

If `WorkspaceDraft` has no `Default`, build it explicitly in the helper. Adjust `card()` to the real `PublicationCard` field types (`settings` is `BTreeMap<String, serde_json::Value>` in core, `BTreeMap<String, String>` in the UI — use the core type here).

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pubgen-core translation_units -- --nocapture`
Expected: FAIL — module missing.

- [ ] **Step 3: Implement collection**

Create `crates/pubgen-core/src/translation_units.rs`. Implement §3.2 exactly, in this order: trim/empty → numeric regex-free char scan → identifier prefixes (`ISSN`, `ISBN`, `Katalog BPS:`, `@`, `http`, `://`, path separators with a known extension) → protected-list normalized match → bilingual pair detection (` / `, `\u{2028}`).

`collect_units` walks `draft.metadata.cards` and dispatches on `CardKind` per the §3.1 matrix. For Table cards, parse the staged CSV from `draft.csv` by the card's `source` key: row 0 → `split_header_stacks` → `TableHeader`; first column of each subsequent row → `TableRowLabel`; rows recognised as notes by the same prefixes `excel.rs:707-717` uses (`Catatan`, `Note`, `Sumber`, `Source`, `Keterangan`, `Ket`) → `TableNote`; everything numeric → skipped. For Chart cards read `series_n_label` keys; for MapChart read the `title` column only. For Narrative/Divider collect the title only (Phase 1). For Image collect the caption (`UnitRole::ImageCaption`). For Toc collect the contents title only. For Cover collect `title` and `cover_title2` only.

`key` = the trimmed source text (matching the existing `st.translations` key convention). Deduplicate by `key`, keeping the first role seen. Cap at 50_000 units.

`protected_names` collects GeoJSON region names referenced by MapChart cards plus agency/institution and person-name fields from Cover cards.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p pubgen-core`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pubgen-core/src/translation_units.rs crates/pubgen-core/src/lib.rs
git commit -m "T5: per-card-kind translation unit collection and skip rules"
```

---

### Task 6: Provider selection and the third setup pane

**Files:**
- Modify: `crates/ui-egui/src/panels/pubgen/ai.rs:19-20, 30-49, 55-75, 135-237, 477-487, 1323-1336`
- Modify: `crates/ui-egui/src/panels/pubgen/mod.rs:157-164, 288-291`
- Test: `crates/ui-egui/src/panels/pubgen/ai.rs` (existing test module, `:2246+`)

**Interfaces:**
- Consumes: `ProviderSettings` (Task 2), `OnlineTranslator::with_endpoint` (Task 1), prefs fields (Task 2).
- Produces:
  - `pub struct BackendConfig { pub provider: String, pub key: String, pub model: String, pub base_url: String }`
  - `pub fn backend_config_from_prefs(app: &DesignApp) -> Option<BackendConfig>` (replaces `backend_from_prefs`; keep a thin `backend_from_prefs` wrapper returning the old 3-tuple only if an unmigrated caller remains, otherwise delete it and update all five call sites)
  - `pub const PROVIDER_CUSTOM: usize = 2;` and `pub const PROVIDER_LOCAL: usize = 3;`
  - `pub fn custom_pane(...)`, `pub fn local_pane(...)` mirroring `groq_pane`
  - `TranslateJob::spawn(config: &BackendConfig, units: Vec<(String, String)>, glossary: Vec<(String, String)>, ctx: egui::Context) -> Self`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn backend_config_resolves_each_provider() {
    let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
    app.session.prefs.translation_provider = "custom".into();
    app.session.prefs.translation_custom_base_url = "https://api.example.com/v1/".into();
    app.session.prefs.translation_custom_api_key = "k".into();
    app.session.prefs.translation_custom_model = "gpt-4o-mini".into();
    let cfg = backend_config_from_prefs(&app).expect("custom configured");
    assert_eq!(cfg.provider, "custom");
    assert_eq!(cfg.base_url, "https://api.example.com/v1", "trailing slash trimmed");
    assert_eq!(cfg.model, "gpt-4o-mini");

    app.session.prefs.translation_provider = "local".into();
    app.session.prefs.translation_local_model = "qwen2.5:7b".into();
    let cfg = backend_config_from_prefs(&app).expect("local configured");
    assert_eq!(cfg.provider, "local");
    assert_eq!(cfg.base_url, "http://localhost:11434/v1");
    assert!(cfg.key.is_empty(), "a local server needs no key");
}

#[test]
fn an_unconfigured_provider_yields_none() {
    let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
    app.session.prefs.translation_provider = "groq".into();
    app.session.prefs.translation_groq_api_key = "  ".into();
    assert!(backend_config_from_prefs(&app).is_none());
    app.session.prefs.translation_provider = "custom".into();
    assert!(backend_config_from_prefs(&app).is_none(), "no base URL");
}

#[test]
fn a_custom_endpoint_requires_a_scheme() {
    let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
    app.session.prefs.translation_provider = "custom".into();
    app.session.prefs.translation_custom_base_url = "api.example.com/v1".into();
    app.session.prefs.translation_custom_model = "m".into();
    assert!(backend_config_from_prefs(&app).is_none(), "must not guess a scheme");
}

#[test]
fn the_legacy_offline_provider_is_not_silently_groq() {
    let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
    app.session.prefs.translation_provider = "offline".into();
    // NLLB is a stub; it must not resolve to a working backend.
    assert!(backend_config_from_prefs(&app).is_none());
}
```

`DesignApp` has no `Default` — the tests use the same construction the existing UI tests use (`ai.rs:2674`): `DesignApp::new(designcraft_engine::Session::new(), crate::Services::default())`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online backend_config -- --nocapture`
Expected: FAIL — `backend_config_from_prefs` missing.

- [ ] **Step 3: Implement `BackendConfig` and provider resolution**

Replace `backend_from_prefs` (`ai.rs:1323-1336`) with:

```rust
/// The backend the panel should use right now. `None` when the selected
/// provider is not configured; never falls back to a different provider.
pub fn backend_config_from_prefs(app: &DesignApp) -> Option<BackendConfig> {
    let p = &app.session.prefs;
    let cfg = match p.translation_provider.as_str() {
        "groq" => BackendConfig {
            provider: "groq".into(),
            key: p.translation_groq_api_key.trim().to_string(),
            model: if p.translation_groq_model.trim().is_empty() {
                "llama-3.3-70b-versatile".to_string()
            } else {
                p.translation_groq_model.trim().to_string()
            },
            base_url: "https://api.groq.com/openai/v1".into(),
        },
        "custom" => BackendConfig {
            provider: "custom".into(),
            key: p.translation_custom_api_key.trim().to_string(),
            model: p.translation_custom_model.trim().to_string(),
            base_url: p.translation_custom_base_url.trim().trim_end_matches('/').to_string(),
        },
        "local" => BackendConfig {
            provider: "local".into(),
            key: String::new(),
            model: p.translation_local_model.trim().to_string(),
            base_url: if p.translation_local_base_url.trim().is_empty() {
                "http://localhost:11434/v1".to_string()
            } else {
                p.translation_local_base_url.trim().trim_end_matches('/').to_string()
            },
        },
        // NLLB is a stub; resolving it would promise a backend that cannot run.
        _ => return None,
    };
    if cfg.model.is_empty() || cfg.base_url.is_empty() {
        return None;
    }
    if !cfg.base_url.starts_with("http://") && !cfg.base_url.starts_with("https://") {
        return None;
    }
    if cfg.provider == "groq" && cfg.key.is_empty() {
        return None;
    }
    if !designcraft_translate::online_available() {
        return None;
    }
    Some(cfg)
}
```

Update all five caller sites (`ai.rs:429/447, 2142, 2161, 2197`) and `mod.rs:157-159, 288-290`. Change `TranslateJob::spawn` to take `&BackendConfig` and construct `OnlineTranslator::with_endpoint(&config.key, &config.model, &config.base_url)` for **all three** providers — delete the `if provider == "groq"` branch and its `Unavailable` else-arm. Keep the wasm stub.

In `mod.rs`, the quota reporting filter `provider == "groq"` stays correct (only Groq has a meaningful remaining-requests header), but `groq_quota_suffix` (`ai.rs:1304`) and `push_groq_quota_popup` (`:1314`) must print the actual provider label for custom/local instead of "Groq".

- [ ] **Step 4: Add the third and fourth panes**

Add `PROVIDER_CUSTOM = 2` and `PROVIDER_LOCAL = 3` at `ai.rs:19-20`. Extend `AiSetupDialog` (`:30-49`) with `custom_base_url`, `custom_key`, `custom_model`, `custom_models`, `custom_models_error`, `local_base_url`, `local_model`, `local_models`, `local_models_error`. Populate them in `open` (`:55-75`).

**Change the dispatch at `:483-487` from `if/else` to an exhaustive `match`** so a new provider can never silently fall into the offline pane:

```rust
match dialog.provider {
    PROVIDER_GROQ => groq_pane(app, st, dialog, ui),
    PROVIDER_CUSTOM => custom_pane(app, st, dialog, ui),
    PROVIDER_LOCAL => local_pane(app, st, dialog, ui),
    _ => offline_pane(app, st, dialog, ui),
}
```

Add radios for Custom and Local at `:477-480`. `custom_pane` mirrors `groq_pane` (`:135-182`) with an added base-URL `TextEdit` above the key field, a "Refresh models" button calling `list_models` against the custom endpoint, and a "Use AI" button that verifies with one live `translate(["tes"])` then writes `translationProvider: "custom"` plus the three custom fields via `prefs.set`. `local_pane` mirrors it with the base URL defaulting to `http://localhost:11434/v1`, **no key field**, and helper text naming Ollama / LM Studio / llama.cpp.

Relabel the offline pane's heading to make the stub honest: "Local model (NLLB) — not available in this build". Do not remove it.

In `apps/pubgen-studio`, on accept also call `ProviderSettings::save()` (Task 2) so the config survives restart.

- [ ] **Step 5: Run the tests and look at the dialog**

Run: `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online backend_config`
Expected: PASS.

Run: `cargo xtask ci`
Expected: green.

**Look at the result** (AGENTS.md requires it for UI work): run `cargo run --release -p pgs`, open Publication Tools ▸ AI Translate ▸ Setup…, and take a `ui.screenshot` via the control channel (`docs/control-protocol.md`) showing all four provider radios and the Custom pane with its base-URL field. Read the PNG and confirm the dispatch shows the right pane for each radio. If no frame is presented, use `ui.render`.

- [ ] **Step 6: Commit**

```bash
git add crates/ui-egui/src/panels/pubgen/ai.rs crates/ui-egui/src/panels/pubgen/mod.rs apps/pubgen-studio/src
git commit -m "T6: Groq/custom/local provider selection and setup panes"
```

---

### Task 7: The pre-compile stage and write-back

**Files:**
- Create: `crates/ui-egui/src/panels/pubgen/translate_stage.rs`
- Modify: `crates/ui-egui/src/panels/pubgen/mod.rs` (declare the module)
- Modify: `crates/ui-egui/src/panels/pubgen/workspace.rs:325-380` (state field), `:4019` (seed), `:4287` (toolbar/log)
- Modify: `crates/ui-egui/src/panels/pubgen/pages.rs:474-483` (Image EN slot)
- Test: `crates/ui-egui/src/panels/pubgen/translate_stage.rs`

**Interfaces:**
- Consumes: `collect_units`, `TransUnit`, `UnitRole`, `TranslationMemory`, `TmEntry`, `TmOrigin`, `now_stamp` (Tasks 4–5); `mask`/`unmask`/`number_census` (Task 3); `BackendConfig`, `TranslateJob` (Task 6); `BpsGlossary` (`ai.rs`).
- Produces:
  - `pub struct StageResult { pub translated: usize, pub reused: usize, pub skipped: usize, pub failed: Vec<(String, String)> }`
  - `pub fn plan_units(draft: &WorkspaceDraft, tm: &TranslationMemory, glossary: &BpsGlossary) -> (Vec<TransUnit>, StageResult)` — TM/glossary hits resolved locally, only misses returned
  - `pub fn apply_results(draft: &mut WorkspaceDraft, tm: &mut TranslationMemory, units: &[TransUnit], results: &[(String, String)]) -> StageResult`
  - `pub fn seed_pubgen_state(st: &mut PubgenState, tm: &TranslationMemory)`
  - `pub fn write_card_english(card: &mut PublicationCard, role: UnitRole, source: &str, english: &str)`

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Same explicit builder as Task 5: `WorkspaceDraft` has no `Default`.
    fn draft() -> WorkspaceDraft {
        WorkspaceDraft {
            manifest: pubgen_core::ProjectManifest { format_version: 1, name: "test".into(), template: false, slots: Vec::new() },
            metadata: pubgen_core::PublicationMetadata {
                profile_id: "bps_dda".into(),
                layout: pubgen_core::PageLayout::Single,
                language: pubgen_core::Language::Id,
                title: "test".into(),
                title_en: None,
                cards: Vec::new(),
                extra: Default::default(),
            },
            glossary: pubgen_core::Glossary::default(),
            csv: Vec::new(),
            text: Vec::new(),
            assets: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn reviewed_tm_entries_are_reused_without_a_request() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Luas Wilayah menurut Kecamatan"));
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry { source: "Luas Wilayah menurut Kecamatan".into(), target: "Total Area by Subdistrict".into(), origin: TmOrigin::Reviewed, role: "table_title".into(), updated: now_stamp() });
        let (misses, res) = plan_units(&draft, &tm, &BpsGlossary::default());
        assert!(misses.is_empty(), "a reviewed hit must not reach the provider");
        assert_eq!(res.reused, 1);
    }

    #[test]
    fn a_whole_string_glossary_hit_is_deterministic() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Kecamatan"));
        let mut g = BpsGlossary::default();
        // `GlossaryTerm::new` is private (ai.rs:837) — construct the struct directly.
        g.terms.push(GlossaryTerm { id: "Kecamatan".into(), official: "Subdistrict".into(), ..Default::default() });
        let (misses, res) = plan_units(&draft, &TranslationMemory::default(), &g);
        assert!(misses.is_empty());
        assert_eq!(res.reused, 1);
    }

    #[test]
    fn apply_results_writes_title_en_and_the_tm() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Luas Wilayah"));
        let units = vec![TransUnit { key: "Luas Wilayah".into(), text: "Luas Wilayah".into(), role: UnitRole::TableTitle, card_id: draft.metadata.cards[0].id.clone() }];
        let mut tm = TranslationMemory::default();
        let res = apply_results(&mut draft, &mut tm, &units, &[("Luas Wilayah".into(), "Total Area".into())]);
        assert_eq!(res.translated, 1);
        assert_eq!(draft.metadata.cards[0].settings.get("title_en").and_then(|v| v.as_str()), Some("Total Area"));
        assert_eq!(tm.lookup("Luas Wilayah").map(|e| e.origin), Some(TmOrigin::Ai));
    }

    #[test]
    fn a_number_lost_in_translation_is_a_failure_not_a_write() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Tabel 3.1 Luas Wilayah"));
        let units = vec![TransUnit { key: "Tabel 3.1 Luas Wilayah".into(), text: "Tabel 3.1 Luas Wilayah".into(), role: UnitRole::TableTitle, card_id: draft.metadata.cards[0].id.clone() }];
        let mut tm = TranslationMemory::default();
        // The model dropped "3.1": the number census must reject this.
        let res = apply_results(&mut draft, &mut tm, &units, &[("Tabel 3.1 Luas Wilayah".into(), "Table Total Area".into())]);
        assert_eq!(res.translated, 0);
        assert_eq!(res.failed.len(), 1);
        assert!(tm.entries.is_empty(), "a rejected unit must not enter the TM");
    }

    #[test]
    fn seed_fills_the_generation_cache() {
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry { source: "Kecamatan".into(), target: "Subdistrict".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
        let mut st = PubgenState::default();
        seed_pubgen_state(&mut st, &tm);
        assert_eq!(st.translations.get("Kecamatan").map(String::as_str), Some("Subdistrict"));
    }

    #[test]
    fn an_empty_translation_never_overwrites_an_existing_title_en() {
        let mut c = table_card("Luas Wilayah");
        c.settings.insert("title_en".into(), serde_json::json!("Human Edited"));
        write_card_english(&mut c, UnitRole::TableTitle, "Luas Wilayah", "");
        assert_eq!(c.settings.get("title_en").and_then(|v| v.as_str()), Some("Human Edited"));
    }

    fn table_card(title: &str) -> pubgen_core::PublicationCard {
        pubgen_core::PublicationCard {
            id: format!("table-{title}"), kind: CardKind::Table, scope: pubgen_core::CardScope::NewPage,
            source: None, title: title.into(), settings: Default::default(),
        }
    }
}
```

Type decisions (verified): `translate_stage.rs` operates on the **core** `WorkspaceDraft` — that is what `OrchestratorProject.draft` holds (`project.rs:11-19`) and what `collect_units` consumes. `write_card_english` writes the core card's `settings["title_en"]` (`BTreeMap<String, serde_json::Value>`); the UI card's `title_en: String` is refreshed by the existing core→UI mapping in Step 4. The UI `PublicationCard` (`workspace.rs:96`) has no `id`; keys refer to core card ids.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online translate_stage -- --nocapture`
Expected: FAIL — module missing.

- [ ] **Step 3: Implement the stage module**

Create `crates/ui-egui/src/panels/pubgen/translate_stage.rs`:

- `plan_units`: call `collect_units`; for each unit, TM `lookup` → reuse; else `glossary.lookup(whole text)` → deterministic EN, insert as `TmOrigin::Imported`; else mask and keep as a miss. Accumulate `StageResult` counts.
- The worker sends masked text; on return, `unmask` then compare `number_census(source)` with `number_census(restored)`. **If the censuses differ, reject the unit** and record it in `failed`. This is the guard that makes statistical tables safe.
- `apply_results`: for each accepted pair, `tm.insert(TmOrigin::Ai)` and `write_card_english`. Never write an empty target over a non-empty existing one.
- `write_card_english`: dispatch on role → `CardTitle`/`TableTitle`/`ChartTitle`/`MapTitle`/`NarrativeTitle`/`DividerTitle`/`ImageCaption`/`CoverTitle` set `card.title_en` **and** `settings["title_en"]`; `TableHeader`/`TableRowLabel`/`TableNote` go only into the TM (generation reads them via `st.translations`/`pair_cells`); `HeaderFooter` sets the matching `hf_*` field's EN variant.
- `seed_pubgen_state`: `st.translations.extend(tm.to_map())`; set `st.bilingual_output = true` when the TM is non-empty and the profile is bilingual; load the glossary from the draft.

- [ ] **Step 4: Wire it into the Orchestrator**

In `workspace.rs`:
- Add `pub tm: TranslationMemory` to `OrchestratorWorkspaceState` (`:325-380`) with `#[serde(default)]`, and carry it through the `.pubgen` mapping (`:576-735`) and `apply_ingest_report_to_state` (`:750`).
- At `:4019`, immediately after `let mut st = super::PubgenState::default();`, add `super::translate_stage::seed_pubgen_state(&mut st, &state.tm);`.
- Add a "Translate Worksheet" button to the Studio toolbar near the compile action, enabled when `backend_config_from_prefs(app).is_some()` and the worksheet has cards. On click: `plan_units`, then `TranslateJob::spawn` with the masked misses; poll it alongside the existing jobs and call `apply_results` on completion, pushing to `state.execution_log` and setting `state.is_dirty = true`.
- Log to `LogEksekusi` in the existing format: `[{now}] Translated {n} unit(s), reused {m}, skipped {k}, failed {f}.`

In `pages.rs:474-483`, add the missing Image EN slot so a translated caption can render: insert `text.insert("title_en".into(), json!(title_en_val));` in the Image branch, and add a `#slot:text:title_en` frame to the image template asset (or reuse the existing caption frame's second part). **If the template asset has no EN frame, add one via the template-authoring test at `pages.rs:4326` and record the asset change in `ASSETS.md`.**

Two carry-overs from Task 4: switch `compile.rs:2053` from `&TranslationMemory::default()` to `&orch.tm`, and in the core→UI card mapping (`workspace.rs:722-735`) copy `settings["title_en"]` into the UI card's `title_en: String` so the editor shows the translated title.

- [ ] **Step 5: Run the tests and verify end to end**

Run: `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online translate_stage`
Expected: PASS.

Run: `cargo xtask ci`
Expected: green.

**Manual verification with a real provider** (report exactly what ran): open `plan/pubgen/samples/brs/ntp202610.xlsx` in the Orchestrator, set `doc_language` to `Indonesian-English`, configure Local AI against a running Ollama, click Translate Worksheet, then Compile. Take a `ui.screenshot` of a generated table page and confirm the EN title and italic EN cells appear. Read the PNG. Then close and reopen the `.pubgen` and confirm the TM survived (Translate Worksheet reports all units reused, zero requests).

- [ ] **Step 6: Commit**

```bash
git add crates/ui-egui/src/panels/pubgen/translate_stage.rs crates/ui-egui/src/panels/pubgen/mod.rs crates/ui-egui/src/panels/pubgen/workspace.rs crates/ui-egui/src/panels/pubgen/pages.rs ASSETS.md
git commit -m "T7: pre-compile translation stage with card write-back"
```

---

### Task 8: Review surface and glossary revival

**Files:**
- Modify: `crates/ui-egui/src/panels/pubgen/workspace.rs` (card editor EN field, review list, glossary seeding)
- Modify: `apps/pubgen-studio/src/domain/glossary.rs` (wire `enforce`)
- Test: `crates/ui-egui/src/panels/pubgen/translate_stage.rs`, `apps/pubgen-studio/src/domain/glossary.rs`

**Interfaces:**
- Consumes: `TranslationMemory::pending_review/promote/update_target` (Task 4), `domain::glossary::enforce` (existing).
- Produces:
  - `pub fn review_pane(ui: &mut egui::Ui, tm: &mut TranslationMemory, dirty: &mut bool)`
  - `pub fn deterministic_fix(text: &str, entries: &[domain::glossary::Entry]) -> (String, usize)`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn accepting_a_draft_promotes_it_to_reviewed() {
    let mut tm = TranslationMemory::default();
    tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
    assert_eq!(tm.pending_review().len(), 1);
    tm.promote("PDRB");
    assert!(tm.pending_review().is_empty());
    // A later AI run must not undo the human decision.
    tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
    assert_eq!(tm.lookup("PDRB").map(|e| e.origin), Some(TmOrigin::Reviewed));
}

#[test]
fn editing_a_draft_promotes_and_replaces() {
    let mut tm = TranslationMemory::default();
    tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
    assert!(tm.update_target("PDRB", "GRDP"));
    assert_eq!(tm.lookup("PDRB").map(|e| e.target.as_str()), Some("GRDP"));
    assert_eq!(tm.lookup("PDRB").map(|e| e.origin), Some(TmOrigin::Reviewed));
}

#[test]
fn glossary_enforce_is_applied_to_ai_output() {
    let entries = vec![domain::glossary::Entry { term: "district".into(), replacement: "Subdistrict".into(), do_not_translate: false, case_preserve: true }];
    let (fixed, n) = deterministic_fix("Karawang Timur district", &entries);
    assert_eq!(n, 1);
    assert_eq!(fixed, "Karawang Timur Subdistrict");
}

#[test]
fn do_not_translate_entries_are_left_alone() {
    let entries = vec![domain::glossary::Entry { term: "NTP".into(), replacement: "whatever".into(), do_not_translate: true, case_preserve: false }];
    let (fixed, n) = deterministic_fix("Perkembangan NTP", &entries);
    assert_eq!(n, 0);
    assert_eq!(fixed, "Perkembangan NTP");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online accepting_a_draft -- --nocapture`
Expected: FAIL — `deterministic_fix` missing (the TM tests from Task 4 should already pass; these confirm the wiring).

- [ ] **Step 3: Implement the review pane and wire `enforce`**

`review_pane` lists `tm.pending_review()` as rows: source (read-only), target (`TextEdit`), Accept and Reject buttons. Accept → `promote`; editing then Accept → `update_target`; Reject → remove the entry. Set `dirty = true` on any change. Show it as a fourth right-dock tab (`OrchestratorRightTab::Terjemahan`) or a modal opened from the Studio toolbar — pick the right-dock tab, it matches `SumberKonten`/`ProfilEksekusi`.

In the card editor, next to the existing `title_en` field (shown when bilingual, `workspace.rs:3204-3210`), add a badge: "AI draft" when the matching TM entry is `Ai`, "Reviewed" when `Reviewed`, and an Accept button that promotes it.

`deterministic_fix` is a thin wrapper over `domain::glossary::enforce`. Call it on every AI result inside `apply_results` **after** unmask and **before** the number-census check, so official terminology is enforced even when the model ignored the prompt. Convert `BpsGlossary` terms to `domain::glossary::Entry` at the boundary.

Populate the draft's glossary at the ui-egui boundary — in the Orchestrator open/ingest path (`apply_ingest_report_to_state`, `workspace.rs:753`) and in `seed_pubgen_state` — by converting `BpsGlossary::load(...)` terms (source `id`/`official`) into core `GlossaryEntry` pairs, so `glossary.json` stops being written empty. This cannot live in `pubgen-core`'s `from_ingest`: the bundled `assets/pubgen/bps_glossary.json` is a ui-egui asset.

- [ ] **Step 4: Run the tests and look at the pane**

Run: `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online && cargo test -p pgs`
Expected: PASS.

Run the app, translate a worksheet, open the review tab, edit one target, Accept it, and take a `ui.screenshot`. Read the PNG and confirm the badge changes from "AI draft" to "Reviewed" and that a second Translate Worksheet run does not overwrite it.

- [ ] **Step 5: Commit**

```bash
git add crates/ui-egui/src/panels/pubgen/workspace.rs crates/ui-egui/src/panels/pubgen/translate_stage.rs apps/pubgen-studio/src/domain/glossary.rs
git commit -m "T8: translation review gate and deterministic glossary enforcement"
```

---

### Task 9: Headless `pgs-cli translate`

**Files:**
- Modify: `apps/pubgen-cli/Cargo.toml` (add `designcraft-translate = { workspace = true, features = ["online"] }`)
- Modify: `apps/pubgen-cli/src/main.rs` (USAGE const, dispatch, subcommand)
- Test: `apps/pubgen-cli/src/main.rs`

**Interfaces:**
- Consumes: `pubgen_core::{open_pubgen, save_pubgen, TranslationMemory, collect_units}`, `designcraft_translate::{OnlineTranslator, mask, unmask}`.
- Produces: a `translate` subcommand: `pgs-cli translate <project.pubgen> --provider <groq|custom|local> --base-url <url> --model <id> [--key-env <VAR>] [--dry-run]`, plus `pub struct TranslateArgs { project: PathBuf, provider: String, base_url: String, model: String, key_env: Option<String>, dry_run: bool }` and `pub fn parse_translate_args(args: &[String]) -> Result<TranslateArgs, String>`.

The CLI has no clap dependency (verified: `main.rs` parses `args.first()` by hand against a `USAGE` const) — parse by hand in the same style.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn translate_dry_run_reports_units_without_network() {
    let dir = std::env::temp_dir().join(format!("pgs-cli-tr-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("t.pubgen");
    write_fixture_package(&path);
    let args = parse_translate_args(&[
        "t.pubgen".to_string(), "--provider".into(), "local".into(),
        "--base-url".into(), "http://localhost:11434/v1".into(), "--model".into(), "m".into(),
        "--dry-run".into(),
    ]).expect("args parse");
    let report = run_translate(&args).expect("dry run");
    assert!(report.units > 0);
    assert_eq!(report.translated, 0, "a dry run must not call the provider");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_key_is_read_from_the_environment_not_an_argument() {
    // --key-env names a variable; the key itself must never appear in argv,
    // shell history, or a process listing.
    let args = parse_translate_args(&[
        "t.pubgen".to_string(), "--provider".into(), "custom".into(),
        "--base-url".into(), "https://x/v1".into(), "--model".into(), "m".into(),
        "--key-env".into(), "MY_KEY".into(),
    ]).expect("args parse");
    assert_eq!(args.key_env.as_deref(), Some("MY_KEY"));
    let err = parse_translate_args(&["t.pubgen".into(), "--api-key".into(), "secret".into()]);
    assert!(err.is_err(), "a literal key argument must not exist");
}
```

Add `write_fixture_package` and `run_translate(&TranslateArgs) -> Result<TranslateReport, String>` helpers returning a `TranslateReport { units, translated, reused, skipped, failed }` (serde-serialized to stdout as JSON).

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p pubgen-cli translate_dry_run -- --nocapture`
Expected: FAIL — subcommand missing.

- [ ] **Step 3: Implement the subcommand**

Add `designcraft-translate = { workspace = true, features = ["online"] }` to `apps/pubgen-cli/Cargo.toml`, extend the `USAGE` const and the `main` dispatch (`Some("translate") => report(run_translate(&parse_translate_args(&args[1..])?))`). The handler: opens the package, collects units, resolves TM/glossary hits, masks the misses, and — unless `--dry-run` — calls `OnlineTranslator::with_endpoint`, unmasks, validates the number census, inserts into the TM and saves the package. Read the key from the variable named by `--key-env` via `std::env::var`; never accept a literal key argument. Print the report as JSON to stdout. Exit non-zero when any unit failed.

Unlike `compile` (`main.rs:158-195`), this does **not** spawn an external `designcraft` binary — `pubgen-core` is egui-free and the translate crate is pure, so the whole stage runs in-process.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p pubgen-cli && cargo xtask ci`
Expected: PASS / green.

- [ ] **Step 5: Commit**

```bash
git add apps/pubgen-cli/Cargo.toml apps/pubgen-cli/src/main.rs
git commit -m "T9: headless pgs-cli translate subcommand"
```

---

### Task 10: Narrative body translation (Phase 5, optional)

**Files:**
- Modify: `crates/pubgen-core/src/translation_units.rs` (enable `NarrativeBody`)
- Modify: `crates/ui-egui/src/panels/pubgen/translate_stage.rs`
- Modify: `crates/ui-egui/src/panels/pubgen/pages.rs:628-690` (heading detection)
- Test: `crates/ui-egui/src/panels/pubgen/translate_stage.rs`

**Interfaces:**
- Consumes: `designcraft_translate::segment` (`lib.rs:175`), `textimport::import`, `st.translated_bodies`, `translated_body_rtf` (`ai.rs:1236`).
- Produces: `pub fn translate_narrative(rtf_bytes: &[u8], cfg: &BackendConfig, glossary: &[(String,String)]) -> Result<Vec<u8>, TranslateError>` returning EN RTF.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn narrative_paragraph_count_and_styles_survive_translation() {
    let rtf = br"{\rtf1\ansi\b Heading_one\b0\par Body paragraph one.\par Body paragraph two.\par}";
    let out = translate_narrative_with(rtf, &|texts| {
        // Canned provider: return one EN string per input paragraph.
        texts.iter().map(|t| format!("EN {t}")).collect()
    }).expect("translation");
    let imported = designcraft_textimport::import("t.rtf", &out).expect("reimport");
    // Use the same paragraph-count API the plain-text path uses (ai.rs:793).
    assert_eq!(paragraph_count(&imported.story), 3, "paragraph count must be preserved");
    assert!(imported.story.text.contains("EN "), "{:?}", imported.story.text);
}

#[test]
fn a_paragraph_the_model_dropped_keeps_its_indonesian_text() {
    let rtf = br"{\rtf1\ansi Satu.\par Dua.\par}";
    let out = translate_narrative_with(rtf, &|_texts| vec!["One.".to_string()]) // one short
        .expect("must not fail the whole body");
    let imported = designcraft_textimport::import("t.rtf", &out).expect("reimport");
    assert!(imported.story.text.contains("Dua."), "the untranslated paragraph must survive");
}
```

`designcraft_textimport::import` takes `(name, bytes)` (`textimport/src/lib.rs:112`). `paragraph_count` is a small test helper over the `Story` paragraph API — use the same accessor the existing plain-text extraction at `ai.rs:793` uses. Provide `translate_narrative_with(rtf, fake_provider)` as the testable seam so no network is needed.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online narrative_paragraph -- --nocapture`
Expected: FAIL — function missing.

- [ ] **Step 3: Implement it**

Import the RTF, extract `story.text` paragraph by paragraph (the plain-text path already used at `ai.rs:793`), `segment()` each paragraph, mask, translate, unmask, validate the number census per paragraph, then rebuild the story **by paragraph index**, copying each source paragraph's style and char runs onto the translated text. A paragraph whose translation failed or came back short keeps its Indonesian text. Emit RTF preserving those styles rather than using `write_plain_rtf` (`ai.rs:1280`), which flattens everything to Arial fs22.

Fix `format_brs_narrative_story` (`pages.rs:628-690`) so heading detection uses **paragraph style names**, not the hardcoded Indonesian strings at `:671-682`; otherwise English narratives lose their heading structure.

- [ ] **Step 4: Run the tests and inspect a generated spread**

Run: `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online && cargo xtask ci`
Expected: green.

Generate a DDA publication with a narrative card, take `ui.screenshot` of the ID/EN spread, read the PNG and confirm paragraph styles and the TOC entries derived from them are intact.

- [ ] **Step 5: Commit**

```bash
git add crates/pubgen-core/src/translation_units.rs crates/ui-egui/src/panels/pubgen/translate_stage.rs crates/ui-egui/src/panels/pubgen/pages.rs
git commit -m "T10: narrative body translation preserving paragraph styles"
```

---

### Task 11: Documentation

**Files:**
- Modify: `docs/pubgen-guide.md:1048-1080` (§13)
- Modify: `docs/pubgen-guide_id.md:945-975`
- Modify: `docs/site/guide/12-ai-translate.md`
- Modify: `ROADMAP.md` (progress log line)
- Modify: `ASSETS.md` (only if Task 7 added a template frame)

- [ ] **Step 1: Rewrite §13**

Document: the three providers and how to configure each (Groq key; custom base URL + key + model; Local AI with Ollama/LM Studio and no key); the pre-compile Translate Worksheet stage; the per-page-type matrix from §3.1 including what is deliberately never translated; the review gate and the AI-draft vs Reviewed distinction; TM persistence in `.pubgen`; masking and the number-integrity guarantee; `pgs-cli translate`. State the TLS limitation honestly (ureq 2 uses Mozilla roots only — a private CA needs plain HTTP on localhost or a publicly-trusted certificate). Remove the stale claim that translation is post-generation only.

- [ ] **Step 2: Mirror it in the Indonesian edition and the site guide**

Same content in `docs/pubgen-guide_id.md` and `docs/site/guide/12-ai-translate.md`.

- [ ] **Step 3: Add the ROADMAP progress line and verify**

Run: `cargo xtask ci`
Expected: green (the assets gate checks `ASSETS.md`).

- [ ] **Step 4: Commit**

```bash
git add docs/pubgen-guide.md docs/pubgen-guide_id.md docs/site/guide/12-ai-translate.md ROADMAP.md ASSETS.md
git commit -m "T11: document Orchestrator translation, providers and review gate"
```

---

## 6. Verification

| Check | Command | When |
|---|---|---|
| Full gate | `cargo xtask ci` | every commit |
| Translate crate | `cargo test -p designcraft-translate --features online` | Tasks 1, 3 |
| Core | `cargo test -p pubgen-core` | Tasks 4, 5 |
| UI stage | `cargo test -p designcraft-ui-egui --features pubgen,ai-translate-online` | Tasks 6, 7, 8, 10 |
| Studio | `cargo test -p pgs` | Tasks 2, 4, 8 |
| CLI | `cargo test -p pubgen-cli` | Task 9 |
| Layering | `cargo xtask layers` | every commit (core/translate must stay egui-free) |
| wasm | `cargo xtask wasm` | every commit (all new network code needs a stub) |
| Visual | `ui.screenshot` via the control channel, then read the PNG | Tasks 6, 7, 8, 10 |

**End-to-end acceptance:** import `plan/pubgen/samples/brs/ntp202610.xlsx`, set `Indonesian-English`, configure Local AI, Translate Worksheet, Compile → a table page shows an EN title and italic EN cells with numbers byte-identical to the source; save, close, reopen → Translate Worksheet reports every unit reused with zero provider requests; edit one reviewed entry → a later AI run does not overwrite it.

Report only what actually ran. If a live provider was not exercised, say so and name the canned-server tests that covered the path instead.

## 7. Risks and Non-Goals

| Risk | Mitigation |
|---|---|
| Model corrupts a statistical number | Masking keeps numbers out of the request; the number census rejects any mismatch (Task 7) |
| Model ignores terminology | Masking guarantees glossary terms; `enforce` is a second deterministic net (Task 8) |
| Model returns wrong item count | Existing hard error + item-by-item salvage (`online.rs:181-189`, `:162-165`) — unchanged |
| Custom endpoint uses a private CA | ureq 2.12.1 ships `webpki-roots` only. Document the limit; localhost HTTP works. Adding `rustls-native-certs` is a separate, licence-checked decision |
| Strict server rejects `reasoning_effort` | Only injected when the model id contains `gpt-oss` (`online.rs:266-270`, `:302-306`) |
| Rate-limit counter clobbered across endpoints | `store_remaining` only stores for Groq (Task 1) |
| RTF formatting lost | Phase 5 rebuilds by paragraph index; Phase 1 translates titles only (D6) |
| English headings not detected | `format_brs_narrative_story` switches to style-name detection (Task 10) |
| TM grows without bound | 100_000-entry cap (Task 4); 512-token mask cap (Task 3) |
| Key leakage | Header only; never in TM, `.pubgen`, logs or error strings; CLI reads it from an env var (Task 9) |

**Non-goals:** NLLB ONNX inference and model download (D1); languages other than ID→EN; translating region proper nouns, identifiers, dates or person names; XLSX export of translations; changing CI; restructuring `workspace.rs`.

## 8. Suggested Order

Tasks 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 11, with 10 last and optional. Tasks 1–2 (providers) and 3–5 (engine) are independent and can run in parallel with separate `CARGO_TARGET_DIR`s. Task 7 is the first point where a user sees a bilingual document; everything before it is infrastructure.