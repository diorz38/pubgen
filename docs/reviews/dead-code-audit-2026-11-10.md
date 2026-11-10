# Dead Code Audit — PubGen Studio, pubgen-core, panels/pubgen

**Date:** 2026-11-10
**Branch:** `pgs-0.9.X` (v0.18.0-pgs.0.9.0)
**Scope:** `apps/pubgen-studio` (crate `pgs`, including `workspaces/orchestrator`), `crates/pubgen-core`, `crates/ui-egui/src/panels/pubgen` (exposed as `dock::pubgen` under the `pubgen` feature)
**Status:** Findings only — no source changes made.

---

## 1. Method

- **Binary crate (`pgs`):** `cargo rustc -p pgs --bin pgs --message-format=short -- --force-warn dead_code`. The crate root has `#![allow(dead_code, unused_variables, ...)]` (`apps/pubgen-studio/src/main.rs:9`), which normally hides all dead-code warnings; `--force-warn` overrides the allow for the final crate only, so dependencies build from cache. Result: **19 warnings** (non-test build; items used only by `#[cfg(test)]` tests are flagged).
- **Library crates (`pubgen-core`, `panels/pubgen`):** rustc never flags unused `pub` items in lib crates, so every `pub` item was cross-referenced workspace-wide (all apps, `crates/*`, `xtask`) and each zero-reference candidate verified by reading its definition: re-export chains, trait dispatch, serde persistence, feature gates, and test-only usage were checked before classifying. Key claims were re-verified with independent workspace-wide searches.
- Serde-persisted state (e.g. `OrchestratorWorkspaceState`, `PubgenState`, profile fields) is treated as **live** regardless of code references, because it is (de)serialized into `.pubgen` files / persisted UI state.

## 2. Summary

| Area | Finding |
| :--- | :--- |
| Root app `pgs` | 19 compiler warnings: entire `domain/` tree production-dead (~465 lines), container save path test-only, 2 never-read app fields |
| Orchestrator module | `clear` truly dead; `save`/`is_empty`/`active_project` test-only; **guard dialog can never open at runtime** (functional bug, not just clutter) |
| `pubgen-core` | 5 entire modules dead (superseded headless-compiler architecture), 4 stray methods, 6 fully unused dependencies + 1 effectively unused + 1 test-only dependency |
| `panels/pubgen` | 14 dead `pub` functions, ~17 test-only items; no orphan files |

Common pattern: code superseded by refactors (algorithms moved to `pubgen-core`, compile flow moved to ui-egui, bilingual parsers replaced plain ones) but never deleted.

---

## 3. Root app `pgs` (compiler-confirmed)

Reproduce: `cargo rustc -p pgs --bin pgs -- --force-warn dead_code`

### 3.1 Entire `domain/` tree is production-dead

`mod domain;` (`apps/pubgen-studio/src/main.rs:12`) is the only reference to the module anywhere in the app. Contents are used solely by their own unit tests:

| File | Dead items |
| :--- | :--- |
| `apps/pubgen-studio/src/domain/brs.rs` | `ParsedNarrativePara` (:4), `parse_brs_narrative` (:14), `parse_brs_csv_table` (:90), `estimate_brs_narrative_height` (:116) |
| `apps/pubgen-studio/src/domain/maps.rs` | `GeoFeature` (:6), `GeoMap` (:12), `GeoMap::from_json_str` (:19), `parse_ring` (:110), `parse_hex_color` (:130), `lerp_color` (:142), `render_choropleth_map_svg` (:152) |
| `apps/pubgen-studio/src/domain/glossary.rs` | `Glossary` alias (:11); the `pub use pubgen_core::glossary_fix::{Entry, enforce}` re-export (:9) is also unused but masked by a local `#[allow(unused_imports)]` — the comment admits the algorithm moved to `pubgen-core` |

The live equivalents live in `pubgen-core` (`glossary_fix`) and `crates/ui-egui/src/panels/pubgen/charts/` (GeoJSON/maps).

### 3.2 Orchestrator module (`apps/pubgen-studio/src/workspaces/orchestrator/`)

| Item | Location | Classification |
| :--- | :--- | :--- |
| `RecompileGuardDialog::prompt` | `guard_dialog.rs:27` | **Never called — functional issue.** It is the only way to set `open = true`, so `show()` always returns `GuardDialogAction::None` and the Overwrite / SaveNewVersion arms at `apps/pubgen-studio/src/app.rs:120-129` are unreachable at runtime. The recompile-guard feature is wired but never triggered. |
| `OrchestratorProject::clear` | `project.rs:73` | Truly dead — zero callers, including tests |
| `OrchestratorProject::save` | `project.rs:40` | Test-only (production saves via the engine command `pubgen.saveProject`) |
| `OrchestratorTabs::is_empty` | `project.rs:103` | Test-only |
| `OrchestratorTabs::active_project` | `project.rs:106` | Test-only (production uses `active_project_mut`) |
| `OrchestratorProject.status`, `.ui_synced` | `project.rs:9,11` | Written, never meaningfully read. Not flagged by rustc because `..OrchestratorProject::default()` (e.g. `project.rs:153`) technically reads every field via functional record update. |

Everything else in the module (`open`, `import`, `new_tab`, `activate`, `request_close`, `close`, `sync_from_ui`, `publish_snapshot`, `host_titles`, `render_close_dialog`, …) is live via `app.rs`.

### 3.3 Rest of the app

| Item | Location | Classification |
| :--- | :--- | :--- |
| `PubgenStudioApp.project_path`, `.container` | `apps/pubgen-studio/src/app.rs:11-12` | Fields stored but never read |
| `PubgenContainer::new`, `::save` | `apps/pubgen-studio/src/project/container.rs:29,108` | Test-only; production uses only `open` (in `main.rs`, to check `cached_layout`) |
| `tmp_path` | `project/container.rs:164` | Test-only (called only by `save`) |
| `PubgenContainer.name`, `.path` | `project/container.rs:18-19` | Fields never read |

Note: `manifest`/`glossary`/`content`/`extras`/`metadata` fields are read only inside the test-only `save`, so they are effectively production-dead too; rustc does not flag them because its field analysis is not transitive through dead functions.

`settings.rs`, `file_assoc.rs`, `window_frame.rs`, `mode.rs` produced no warnings.

---

## 4. `crates/pubgen-core`

### 4.1 Five entirely dead modules

Zero references from any consumer (`pgs`, `pubgen-cli`, `ui-egui`; `engine` does not depend on this crate). Reachable only from each other and their own tests. All are re-exported from `lib.rs:22-28`, which is the only external-facing path — and nothing uses it.

| Module | Dead items |
| :--- | :--- |
| `src/compiler.rs` | `PublicationCompiler` (:16), `new` (:27), `session` (:32), `session_mut` (:37), `compile_file` (:42), `compile` (:49) |
| `src/manifest.rs` | `PubgenManifest` (:10), `DocumentSettings` (:37), `ChapterEntry` (:47), `BuildSettings` (:58), `ValidationReport` (:69), `load` (:77), `validate` (:83) |
| `src/plan.rs` | `PublicationOp` (:8), `CommandPlan` (:20) + `new`/`push`/`len`/`is_empty`; variants `ApplyHeaderFooter` (:15), `ExportPdf` (:16) never constructed |
| `src/report.rs` | `CompileReport` (:7), `ChapterReport` (:28), `OutputArtifacts` (:35), `DiagnosticItem` (:43), `DiagnosticSeverity` (:51) |
| `src/workspace_compile.rs` | `WorkspaceDraft::command_plan` (:6) — only its own test (:81) |

This is a superseded "headless compiler" architecture: the live path uses `workspace.rs` types plus ui-egui's compile panel. `crates/ui-egui/src/panels/pubgen/compile.rs:832,842` defines its own local `ChapterReport`/`CompileReport` instead of using these. The serde derives on `manifest`/`report` types do not make them live — no live code path (de)serializes them; the `.pubgen` container uses `workspace.rs` types.

### 4.2 Smaller items

- `WorkspaceDraft::is_valid` — `src/workspace_validation.rs:105` — test-only (:143, :149).
- `LayoutGeometry::width_pt` / `height_pt` — `src/profile.rs:59,63` — only the dead compiler calls them (`compiler.rs:90-91`). The rest of `profile` is live via `pubgen-cli`.

### 4.3 Dead dependencies (`Cargo.toml`)

Verified by workspace-wide grep of the crate source:

| Dependency | Evidence |
| :--- | :--- |
| `designcraft-doc`, `designcraft-compose`, `designcraft-render`, `designcraft-geom`, `designcraft-color`, `designcraft-pdf` | Zero references anywhere in the crate |
| `designcraft-engine` | Used only by the dead `compiler.rs:7` |
| `tempfile` | Used only in tests; belongs in `[dev-dependencies]` |

Live deps: `serde`, `serde_json`, `chrono`, `thiserror`, `calamine`, `csv`, `rust_xlsxwriter`, `zip`, `designcraft-textimport`.

### 4.4 Over-exposed but NOT dead

Internal-only `pub` items reachable from live functions (could be `pub(crate)`): `translation::MAX_ENTRIES`; `translation_units::{is_skippable, split_header_stacks, protected_names}`; many `excel.rs` helpers (`is_ignored_sheet`, `is_cover`, `is_dda_*`, `TableSheetMetaCore`, `extract_table_sheet_metadata_core`, `format_excel_date`, `format_excel_month_year`, `matches_table_note_keyword`, `is_table_note_or_source_row`, `detect_header_end_row`). ui-egui keeps near-duplicate copies of several.

---

## 5. `crates/ui-egui/src/panels/pubgen` (`dock::pubgen`)

No orphan files: all 37 `.rs` files are declared; `testutil.rs` and `charts/map_load_tests.rs` are properly `#[cfg(test)]`-gated. The ui-egui crate compiles clean under default lints, so private dead code is already covered; the items below are `pub` items rustc cannot flag.

### 5.1 Confirmed dead (zero callers anywhere)

| File:line | Item | Note |
| :--- | :--- | :--- |
| `excel.rs:755` | `matches_table_note_keyword_with_profile` | |
| `excel.rs:1589` | `parse_divider_sheet` | Compat wrapper; production calls `parse_divider_sheet_bilingual` directly (:1840) |
| `excel.rs:1583` | `parse_divider_sheet_with_template` | Only caller is the dead `parse_divider_sheet` |
| `locale.rs:114` | `chapter_prefix` | Superseded by inline logic (`pages.rs:378`) / profile fields |
| `locale.rs:134` | `chart_prefix` | Same |
| `locale.rs:144` | `toc_title` | Same (the profile *fields* of these names are live; the functions are not) |
| `locale.rs:154` | `detect_from_records` | |
| `ai.rs:1457` | `en_text` | `en_text` hits in `pages.rs` are local variables / a test-local closure (:6144), not this 3-arg fn |
| `ai.rs:1589` | `slice_signature` | |
| `ai.rs:1752` | `header_status_row` | |
| `ai.rs:2706` | `execute_fix_terms` | No `fixTerms` command wired anywhere |
| `translate_stage.rs:437` | `translate_narrative` | Zero callers; the `_with` variant is test-only |
| `profiles.rs:238` | `matches_graphic` | Only dead member of the `matches_*` family |
| `profiles.rs:53` | `ProfileSector::label` | No `.sector` read or `sector.label()` call in the workspace |

### 5.2 Test-only (referenced solely from `#[cfg(test)]` code)

Keep, gate behind `#[cfg(test)]`, or remove with their tests:

- `excel.rs`: `parse_rtf_review_sheet` (:1047), `parse_rtf_review_sheet_with_template` (:1012), `parse_note_sheet` (:1305), `parse_chart_sheet` (:1377), `worksheet_to_csv` (:1726), `worksheet_to_csv_with_theme` (:1721), `convert_excel_to_csv_data` (:1764), `convert_excel_to_csv_data_with_dir` (:1769), `build_merged_table_headers` (:1596), `is_g_sheet` (:1188)
- `locale.rs:172` `split_table_title`
- `page_utils.rs:265` `execute_page_from_image` (~64 lines)
- `translate_stage.rs:429` `translate_narrative_with`
- `ai.rs:1014` `collect_translation_units`, `ai.rs:1581` `write_plain_rtf`
- `ai_cover_job.rs:512` `aspect_for_dimensions`, `ai_cover_job.rs:131` `FakeImageGenProvider` (pub and NOT test-gated — candidate for `#[cfg(test)]`)

Main cluster: backward-compat wrapper chains in `excel.rs` (`plain → _with_template/_with_dir → _bilingual/_with_profile`); production uses the deepest variant.

### 5.3 Verified live

`charts/*` (dialog, emitter, geojson, layout, palette, spec, stock_maps incl. `EmbeddedMap` used by build.rs), `covers.rs`, `header_footer.rs`, `dialogs.rs`, `toc.rs`, `state.rs`, `pages.rs`, `tables.rs`, `vocab.rs`, `audit.rs`, `ai_cover.rs`, and the `workspace` module consumed by pubgen-studio.

---

## 6. Suggested remediation order (not yet performed)

1. **Guard-dialog wiring bug** (`guard_dialog.rs:27`): decide whether the recompile guard should trigger (call `prompt` before recompile) or be removed — this is a functional decision, not cleanup.
2. **Safe deletions:** dead wrappers in `excel.rs`/`locale.rs`/`ai.rs`/`profiles.rs`/`translate_stage.rs`; `OrchestratorProject::clear`; unused `pubgen-core` dependencies; `tempfile` → `[dev-dependencies]`.
3. **Superseded architectures:** the app `domain/` tree and pubgen-core's five dead modules (confirm no out-of-tree consumers first; the crate is `publish = false`).
4. **Test-only items:** gate behind `#[cfg(test)]` or remove with their tests; then consider removing `#![allow(dead_code, ...)]` from `apps/pubgen-studio/src/main.rs:9` and fixing what surfaces.
5. **Never-read fields** (`app.rs:11-12`, `container.rs:18-19`, `project.rs:9,11`): remove or wire up, depending on intent.

Any removal should land with `cargo xtask ci` (fmt, clippy `-D warnings`, tests, assets, layers, wasm) per the repository quality gates.