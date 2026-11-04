# Mini Plan: Retire the PubGen Panel, Complete the PubGen Menu

- **Date:** 2026-11-03
- **Branch:** `pubgen-0.8.X`
- **Status:** Proposed (not implemented)
- **Area:** `crates/ui-egui` (menus, panels/pubgen, dock seam, panel docking), docs
- **Feature gate:** `pubgen` (unchanged)
- **Supersedes:** `docs/plans/archives/2026-10-28-pubgen-main-menu-replacement-plan.md` and
  `docs/plans/archives/2026-10-28-pubgen-main-menu-option2.md` for the panel-removal half. The
  menu-bar UX work landed separately in `archives/2026-10-30-pubgen-menu-bar-ux-mini-plan.md`.
- **Goal:** the PubGen menu and its dialogs become the only PubGen surface. The floating PubGen
  panel (`window.pubgenPanel`) is deleted, the dock seam collapses back to upstream
  `dock.rs`, and every panel control has a menu row or dialog that reaches the same code.
- **Non-goal:** redesigning the dialogs, moving generation into the engine, or changing the
  PubGen bar layout decided in the 2026-10-30 UX plan.

---

## 1. Executive summary

The 0.8.X branch already replaced most of the panel: the menu bar draws the host tree plus
`Publication`, `Publication Tools`, `Shortcut` and `Help`, and six modal dialogs reuse the panel's
card functions (`dialogs.rs`). What remains is a **redundant floating panel** (the fork's only
dock panel) plus three holes the panel still covers:

1. the **Publication Compiler** card (manifest paths, New Project form, plan summary, progress,
   Cancel, report) has no dialog and no Cancel command;
2. the **AI** one-shot actions are rows inside `pubgen_bar.rs` only - no command ids, no
   enablement rules, and `Retranslate (Glossary)` is not in the menu at all;
3. per-action **status messages** (`header_status`, `compile_status`) are painted by the panel
   header and would disappear with it.

So this plan is small: one new dialog, ten command ids, one status surface, then the deletion of
`dock_extended.rs`, `panels/pubgen/mod.rs::show`, `PUBGEN_PANELS` and `window.pubgenPanel`, with a
saved-layout migration so nobody loses their workspace.

```text
before: menu + dialogs + a duplicate floating PubGen panel
after:  menu + dialogs (the panel's code reused, its shell deleted)
```

Net effect on the upstream diff: `dock_extended.rs` (420 lines) disappears, `panel_docking.rs`
returns to upstream, and `menus/pubgen_bar.rs` keeps the entire PubGen UI.

---

## 2. Current state (verified 2026-11-03 on `pubgen-0.8.X` @ `e64a59de`)

### 2.1 Menu surface

| What | Where | Note |
| :--- | :--- | :--- |
| PubGen bar (default) | `menus/pubgen_bar.rs:28` | host tree + `Publication`, `Publication Tools`, `Shortcut`, `Help`; default on via `pubgen_menus_on` |
| Single `Pubgen` menu (fallback) | `menus/pubgen_bar.rs:301` | drawn when the bar toggle is off; same row builders |
| Pipeline rows | `menus/pubgen_bar.rs:120` | `pubgen.generateFromExcel`, `openProject`, `compileProject`, `profileCustomize`, `vectorChart`, `tableOfContents`, `publicationCover`, `headerFooter`, `audit`; running jobs get a `●` |
| Content rows | `menus/pubgen_bar.rs:153` | manual ingest, profiles, page utilities, dual-language, AI submenu |
| AI submenu | `menus/pubgen_bar.rs:230` | translate page, translate table range, fix terms, setup - direct state calls, not commands |
| About + bar toggle | `menus/pubgen_bar.rs:285` | also appended to `Help` (`pubgen_bar.rs:60`) |
| Bar hook | `menus/bar.rs:86-107` | `if pubgen_bar::menu_bar(...) { return }`, plus the fallback `pubgen_menu` before Help |
| Dialog hook | `app.rs:303-305` | `crate::dock::pubgen_dialogs(self, &ctx)` every frame |

### 2.2 Dialog surface (already replaces most cards)

`panels/pubgen/dialogs.rs` renders, every frame, through `pubgen_dialogs`:

| Dialog | Reuses | Open flag |
| :--- | :--- | :--- |
| Master Excel | `master_excel_ui` | `dialog_excel_open` |
| Table of Contents | `card_toc_ui` | `dialog_toc_open` |
| Publication Cover | `card_cover_body_ui` | `dialog_cover_open` |
| Header & Footer | `card_header_footer_body_ui` | `dialog_hf_open` |
| Manual CSV & Table | `card1_body_ui` / `card2_body_ui` | `dialog_manual_open`, `dialog_manual_tab` |
| Page Margins & Renumbering | `page_utilities_ui` | `dialog_utils_open` |
| Vector Chart | `charts::dialog` | `chart_dialog` |
| Publication Profile | profile editor | `dialog_profile_customize_open` |
| Table range translate | `dialog_table_range_ui` | `show_table_range_popup` |
| Audit | `audit_dialog_ui` | `audit_open` |
| AI setup | `ai::dialog_ui` | `ai_dialog` |
| About PubGen | `about_pubgen_dialog` | `show_about_dialog` |

All of them are rendered by `show_all_dialogs` (`dialogs.rs:468`) and `pubgen_dialogs`
(`dialogs.rs:486`), which also calls `poll_active_jobs` (`panels/pubgen/mod.rs:109`).

### 2.3 The panel that is still there

| Piece | Where |
| :--- | :--- |
| Module declaration | `dock_extended.rs:9-10` (`#[path = "panels/pubgen/mod.rs"] pub mod pubgen;`) |
| Panel registration | `dock_extended.rs:62` `PUBGEN_PANELS` = `[("pubgen", "PubGen", ...)]` |
| Body arm | `dock_extended.rs:128` `"pubgen" => pubgen::show(app, ui)` |
| Panel body | `panels/pubgen/mod.rs:214` `show` (header band, compiler, Excel, TOC, manual, utilities, About) |
| Duplicate job poll | `panels/pubgen/mod.rs:229-358` (already covered by `poll_active_jobs`) |
| Window row + check | `menus/tree.rs:97-101`, `menus/ui_commands.rs:137`, `menus/state.rs:160` |
| Layout inclusion | `panel_docking.rs:57-58` chains `PUBGEN_PANELS` into `all_panels()` |
| Dock seam | `lib.rs:39-43` selects `dock_extended.rs` under `pubgen` |

### 2.4 The command surface today

`menus/ui_commands.rs:87-137` already registers: `openProject`, `compileProject`, `chartDialog`,
`generateFromExcel`, `profileCustomize`, `vectorChart`, `tableOfContents`, `publicationCover`,
`headerFooter`, `audit`, `pageFromCsv`, `tableFromCsv`, `profileMargins`, `selectProfile`,
`bpsMargins`, `marginsDialog`, `dualLanguage`, `window.pubgenPanel`.

Missing as commands: compiler dialog, new project, compile start/cancel, AI setup, AI page/table
translate, fix terms, retranslate, About. Those exist only as closures in `pubgen_bar.rs` (or only
as panel buttons).

---

## 3. Gap analysis: what only the panel still does

| Panel control | Menu today | Gap |
| :--- | :--- | :--- |
| Compiler: manifest path field + Open... | `pubgen.openProject` (file picker) | no dialog to see/edit the remembered manifest, plan summary or warnings |
| Compiler: New... + New Project form | none | unreachable once the panel is gone |
| Compiler: Compile (step-by-step) | `pubgen.compileProject` runs **synchronously** by default | no progress, no cancel, no report window |
| Compiler: Cancel | none | async job would be unstoppable from the UI |
| Compiler: status line | panel only (`compile_status`) | invisible after removal |
| Header: Translate this Page | submenu row (direct call) | not a command; no enablement; no status line |
| Header: Translate Tables... | submenu row (sets `show_table_range_popup`) | not a command |
| Header: Fix Terms | submenu row (direct call) | not a command; status only in `header_status` |
| Header: Retranslate (Glossary) | **missing** | no route outside the panel |
| Header: AI Translator setup | submenu "Setup..." only | not a command; palette/control cannot open it |
| Header: About (i) | Help > About PubGen... (custom item) | not a command; two implementations |
| Header: profile combo + metadata | Publication Profiles submenu | covered |
| Header: dual/single language | `pubgen.dualLanguage` | covered |
| Excel / TOC / Cover / Manual / HF / Utilities / Charts / Audit | dialogs (`dialog_*`) | covered |
| Page utilities (facing, blank page, numbering, renumber) | `pubgen.marginsDialog` dialog | covered |

Everything marked "covered" is already safe. The plan only has to close the rows above, then delete
the panel.

---

## 4. What replaces the panel (authoritative mapping)

Menu names keep the 2026-10-30 decision: the default PubGen bar has `Publication` (pipeline) and
`Publication Tools` (manual work); the single `Pubgen` menu remains the fallback when the bar
toggle is off. Both are built from the same row functions, so one mapping serves both.

### 4.1 New and changed menu rows

```text
Publication
  Generate from Excel                 Ctrl+Shift+G   (existing)
  ------------------------------------------------------
  Publication Compiler...                            NEW  opens the compiler dialog
  New Publication Project...                         NEW  dialog + New Project form
  Open Publication Project...                        existing command, now opens the dialog too
  Compile Publication                 Ctrl+Shift+B   NEW UI command: async start + dialog
  Cancel Compile                                     NEW, enabled only while job.running
  ------------------------------------------------------
  Vector Chart                        Ctrl+Shift+V   (existing)
  Table of Contents                   Ctrl+Alt+T     (existing)
  Publication Cover                                  (existing)
  Header & Footer Setup                              (existing)
  Audit Publication                   Ctrl+Shift+Alt+A (existing)

Publication Tools
  Manual Ingest & Tables
    Page from CSV...                                 (existing)
    Table from CSV...                                (existing)
  Publication Profiles
    <built-in profiles>                              (existing)
    Customize Active Profile...                      (existing)
    Apply Profile Margins                            (existing)
  Page & Layout Utilities
    Publication Profile                              (existing)
    Apply Profile Margins                            (existing)
    Standard BPS Margins               Ctrl+Shift+M   (existing)
    Page Margins & Renumbering...                    (existing)
  ------------------------------------------------------
  Dual Language Mode                  Ctrl+Shift+L   (existing, checkable)
  AI Translate
    Translate Review/Table on this Page              NEW command id
    Translate Table Pages...                         NEW command id (opens range popup)
    Fix Terms with Glossary                          NEW command id
    Retranslate (Glossary)                           NEW command id
    Setup...                                         NEW command id
  ------------------------------------------------------
  About PubGen                                                        moved to Help only

Help
  ...
  About PubGen...                                    NEW command id (single implementation)
  Show DesignCraft Menus / Show PubGen Menus         (existing toggle)
```

Rules:

- Rows are still built in `pubgen_bar.rs`; the new ids are added as `UI_COMMANDS` rows so the
  palette, shortcut dialog, control channel and MCP see them.
- `Compile Publication` keeps the existing **engine-facing** `pubgen.compileProject` (sync by
  default, params documented). The menu uses a new `pubgen.compileStart` that starts
  `sync:false` and opens the dialog, so scripts keep their contract.
- `window.pubgenPanel` disappears from `UI_COMMANDS`, `menus/tree.rs`, `menus/state.rs` and the
  Window menu.

### 4.2 Control-to-route table (the "what replaces what" answer)

| Panel element | New route | Mode |
| :--- | :--- | :--- |
| Header: Dual/Single language | `Publication Tools > Dual Language Mode` | command, checkable |
| Header: AI Translator button + Groq left | `Publication Tools > AI Translate > Setup...` | command, dialog |
| Header: Translate this Page | `Publication Tools > AI Translate > Translate ... on this Page` | command, one-shot + status |
| Header: Translate Tables... | `... > Translate Table Pages...` | command, opens range dialog |
| Header: Fix Terms | `... > Fix Terms with Glossary` | command, one-shot + status |
| Header: Retranslate (Glossary) | `... > Retranslate (Glossary)` | command, one-shot + status |
| Header: About (i) | `Help > About PubGen...` | command, dialog |
| Header: profile + metadata | `Publication Tools > Publication Profiles` | existing rows |
| Compiler: manifest path | `Publication > Publication Compiler...` | dialog (`compile_card_ui`) |
| Compiler: Open... | `Publication > Open Publication Project...` | command, picker + dialog |
| Compiler: New... | `Publication > New Publication Project...` | command, dialog + form |
| Compiler: plan summary, warnings | dialog | - |
| Compiler: Compile | `Publication > Compile Publication` | command, async + dialog |
| Compiler: Cancel | `Publication > Cancel Compile` + dialog button | command, enabled while running |
| Compiler: progress + report | dialog + `●` dot on the menu row + status bar | - |
| Compiler: status line | status bar (`app.status`) | - |
| Excel card | `Publication > Generate from Excel` | dialog (existing) |
| TOC card | `Publication > Table of Contents` | dialog (existing) |
| Cover card | `Publication > Publication Cover` | dialog (existing) |
| Manual page/table | `Publication Tools > Manual Ingest & Tables` | dialogs (existing) |
| Page utilities | `Publication Tools > Page & Layout Utilities > Page Margins & Renumbering...` | dialog (existing) |
| Charts | `Publication > Vector Chart` / `Object > Chart Options...` | dialog (existing) |
| Audit | `Publication > Audit Publication` | dialog (existing) |
| Panel itself | nothing - the menu + dialogs are the whole surface | deleted |

---

## 5. Design decisions

### 5.1 The compiler gets a dialog, not a status item

`compile_card_ui` (`compile.rs:1635`) already renders the manifest field, Open/New, the plan
summary, Compile/Cancel, the step progress and the report. Wrapping it is cheaper and safer than
inventing a new compiler UI:

```rust
// dialogs.rs
pub fn dialog_compile_ui(app: &mut DesignApp, ctx: &egui::Context, st: &mut PubgenState) {
    if !st.dialog_compile_open { return; }
    let t = Tokens::get(ctx);
    let rtl = crate::i18n::is_rtl(&app.ui.language);
    let mut open = true;
    pubgen_dialog_modal(ctx, tr(&app.ui.language, "Publication Compiler"), rtl, &mut open, 620.0, 680.0,
        |ui| compile_card_ui(app, ui, st, &t));
    st.dialog_compile_open = open;
}
```

- Add `dialog_compile_open: bool` to `PubgenState` (default false, temp storage like the others).
- `pubgen.projectNew` sets `st.compile_new_dialog = Some(Box::new(new_project_dialog_for(st)))` and
  `st.dialog_compile_open = true`; make `new_project_dialog_for` `pub(crate)`.
- `pubgen.compileStart` calls `compile_project(app, {"sync": false})` then opens the dialog.
- `pubgen.compileCancel` takes `st.compile_job`, calls `job.cancel()`; the next
  `poll_compile_job` reports `Cancelled`.
- The Dialog's Compile button keeps using `start_compile` (already in `compile.rs:2116`); the menu
  command reuses it through a small `pub(crate) fn start_compile_from_menu(app, st)` helper.

### 5.2 One command table for menu, palette and control

New ids (all under `#[cfg(feature = "pubgen")]`):

| id | label | shortcut | kind |
| :--- | :--- | :--- | :--- |
| `pubgen.compilerDialog` | Publication Compiler... | - | dialog |
| `pubgen.projectNew` | New Publication Project... | - | dialog + form |
| `pubgen.compileStart` | Compile Publication | Ctrl+Shift+B | async job |
| `pubgen.compileCancel` | Cancel Compile | - | job control |
| `pubgen.aiSetup` | AI Translator Settings... | - | dialog |
| `pubgen.aiTranslatePage` | Translate Page with AI | - | one-shot |
| `pubgen.aiTranslateTables` | Translate Table Pages... | - | dialog |
| `pubgen.aiFixTerms` | Fix Glossary Terms | - | one-shot |
| `pubgen.aiRetranslate` | Retranslate (Glossary) | - | one-shot |
| `pubgen.about` | About PubGen... | - | dialog |

`Ctrl+Shift+B` moves from `pubgen.compileProject` to `pubgen.compileStart`; the old command keeps
working without a chord. All ids are handled in the existing `run_ui_inner` Pubgen arm
(`menus/run.rs:1127`) via a small `pubgen_command(app, id, p)` dispatcher, keeping that file's
growth to one match arm.

### 5.3 Enablement lives in one function

`menus::state::menu_enabled` ends with a generic rule: a known `UI_COMMANDS` id is enabled only
with an open document (unless it starts with `app.`, `window.` or `help.`). That would wrongly
disable the compiler rows, which must work with no document because the compiler creates one.

Add the hook near the top of `menu_enabled`, **before** the `ui_label(id)` fallback:

```rust
#[cfg(feature = "pubgen")]
if let Some(on) = crate::menus::pubgen_bar::enabled(app, id) { return on; }
```

`pubgen_bar::enabled` mirrors the panel's `add_enabled` conditions exactly:

| id | enabled when |
| :--- | :--- |
| `pubgen.compileCancel` | `compile_job` exists and is not finished |
| `pubgen.aiTranslatePage` | document + review/table target + backend + bilingual + no running job |
| `pubgen.aiTranslateTables` | document and page_count > 0 |
| `pubgen.aiFixTerms` | (review or table) + translations + glossary |
| `pubgen.aiRetranslate` | the fix conditions + backend |
| dialog/compiler/about ids | always (the compiler creates the document) |

The same function feeds the disabled tooltip ("Open a document first", already in
`menus/tree.rs:250`).

### 5.4 Status without a panel header

Two surfaces, no new state machine:

1. `app.status(...)` for one-line results ("Glossary applied: N terms", "Compiling step 3 of 7",
   "Compilation cancelled"). The AI command handlers and `poll_compile_job` write it.
2. The existing `●` marker in the Publication menu for running compile/cover jobs
   (`pubgen_bar.rs:136-145`), so progress is visible even with no dialog open.

`header_status` stays for dialogs that show it, but every assignment is mirrored to
`app.status`. A test asserts the status bar changes after a command.

---

## 6. Removing the panel (exact edits)

1. **`crates/ui-egui/src/lib.rs`** - drop the fork seam and declare the module once:

   ```rust
   pub mod dock;

   #[cfg(feature = "pubgen")]
   #[path = "panels/pubgen/mod.rs"]
   pub mod pubgen;
   ```

2. **`crates/ui-egui/src/menus/bar.rs`** - call the menu directly:
   `crate::menus::pubgen_bar::pubgen_menu(app, ui)` (both RTL sites).

3. **`crates/ui-egui/src/app.rs`** - `crate::pubgen::pubgen_dialogs(self, &ctx);`.

4. **`crates/ui-egui/src/dock_extended.rs`** - delete the module declaration, `PUBGEN_PANELS`,
   the `"pubgen"` arm of `panel_body`, the two wrappers and its test module. The file then equals
   `dock.rs`; delete it.

5. **`crates/ui-egui/src/panel_docking.rs`** - delete the `#[cfg(feature = "pubgen")]` chain of
   `PUBGEN_PANELS` in `all_panels()` (lines 57-58). The rest of the file returns to upstream.

6. **`crates/ui-egui/src/panels/pubgen/mod.rs`** - delete `show` (214-401) and the three fixtures
   that call it (403-505); keep `with_state`, `poll_active_jobs`, `PANEL_ID`,
   `open_chart_dialog`. Move the fixtures to the dialogs they now exercise.

7. **`crates/ui-egui/src/menus/tree.rs`** - delete the `Window` block that appends
   `window.pubgenPanel` (97-101); keep the `Object > Chart Options...` row.

8. **`crates/ui-egui/src/menus/ui_commands.rs` / `state.rs`** - remove the `window.pubgenPanel`
   row and its `checked` arm; add the ten ids from 5.2.

9. **Rename the module path** `crate::dock::pubgen` -> `crate::pubgen` (65 references in 10 files:
   `pubgen_bar.rs` 27, `charts/dialog.rs` 17, `run.rs` 10, `compile.rs` 3, `bar.rs` 2,
   `state.rs` 2, `app.rs`, `panel_docking.rs`, `audit.rs`, `charts/spec.rs`). A mechanical,
   compiler-checked change; done last so the diff is easy to review.

10. **`xtask`/docs** - nothing tool-specific exists for the panel (no `diff-dock`); update
    `AGENTS.md`-adjacent docs only where they say "open the PubGen panel":
    `development_pubgen_feature.md`, `user_complete_guide.md`, `docs/pubgen-guide.md`,
    `plan/pubgen/PLAN.md`, `ROADMAP.md`.

### 6.1 Saved-layout migration

`PUBGEN_PANELS` is what makes `"pubgen"` a valid layout id. Removing it makes every saved
workspace that contains the panel **invalid**, and `panel_docking::show` reacts by resetting the
whole dock (`panel_docking.rs:1044-1047`, "Invalid panel layout; restored the default dock").

Do not accept that. Add a pruner before the reset:

```rust
/// A saved layout without panels this build no longer has (the retired PubGen panel):
/// drops the ids and any tabs that became empty, keeping every other panel where it was.
fn pruned(layout: Layout<String>) -> Option<Layout<String>> { ... }

if app.ui.docking.as_ref().is_some_and(|l| !valid(l)) {
    if let Some(fixed) = app.ui.docking.take().and_then(pruned).filter(valid) {
        app.ui.docking = Some(fixed);
    } else {
        app.ui.docking = None;
        forget_panels(app);
        app.status(tr(..., "Invalid panel layout; restored the default dock"));
    }
}
```

Test: a workspace JSON with `pubgen` floating beside `canvas` loads with Properties/Pages/Layers
intact and no `pubgen` tab, and no "restored the default dock" status.

---

## 7. Phases

### Phase 0 - Coverage lock (S, 0.5 day)

- [ ] Walk `panels/pubgen/widgets.rs` + `compile.rs` and tick every control against section 4.2.
- [ ] Add the ten rows to `UI_COMMANDS` (labels, params docs, chords) with handlers that return
      "unsupported" until phase 1 wires them, so the table is the contract.
- [ ] Test: every id used by `pubgen_bar` exists in `UI_COMMANDS` or the engine.

**Acceptance:** section 4.2 has a row per control; the id inventory test passes.

### Phase 1 - Commands and enablement (M, 1 day)

- [ ] Implement `pubgen.compilerDialog`, `projectNew`, `compileStart`, `compileCancel`,
      `aiSetup`, `aiTranslatePage`, `aiTranslateTables`, `aiFixTerms`, `aiRetranslate`,
      `pubgen.about`.
- [ ] Point the existing `pubgen_bar` rows at the command ids instead of direct closures.
- [ ] Add `pubgen_bar::enabled` and the `menus/state.rs` hook (early, before the
      `ui_label(id)` document fallback, so the compiler rows stay enabled with no document).
- [ ] Add `Retranslate (Glossary)` and `About PubGen...` (Help) rows.
- [ ] Mirror `header_status`/`compile_status` into `app.status`.
- [ ] Tests: gates matrix, id uniqueness, palette finds the new rows.

**Acceptance:** every panel action is reachable from the menu, the palette and the control
channel; the panel is still drawing (comparison) but not needed.

### Phase 2 - Compiler dialog (M, 1-1.5 days)

- [ ] `dialog_compile_open` + `dialog_compile_ui` wrapping `compile_card_ui`.
- [ ] `projectNew` opens the dialog with the New Project form; `compileStart` starts async and
      opens it; `compileCancel` cancels.
- [ ] Keep the dialog open across document switches; show the report after success and the error
      after failure.
- [ ] `#[cfg(not(target_arch = "wasm32"))]` on the dialog and its rows.
- [ ] Test: fake running job -> Cancel enabled; cancel -> status "cancelled"; report visible.

**Acceptance:** compile with the dialog closed still finishes (global poll) and the status bar
reports it; opening the dialog shows progress and the report.

### Phase 3 - Delete the panel (S, 0.5 day)

- [ ] Apply section 6 steps 4-8 and 6.1.
- [ ] Move the removed fixtures to dialog fixtures; delete `show` tests.
- [ ] Test: `"pubgen"` is not a registered panel; old layout pruned; `window.pubgenPanel` gone.

**Acceptance:** no PubGen tab, no PubGen panel row, old workspace survives.

### Phase 4 - Collapse the dock seam (S, 0.5-1 day)

- [ ] Apply 6.1 step 1-3, 9 (module rename).
- [ ] Delete `dock_extended.rs`; `dock.rs` is the only dock file in both feature modes.
- [ ] `cargo check -p designcraft-ui-egui` with and without `--features pubgen`.

**Acceptance:** `git diff upstream/main -- crates/ui-egui/src/dock.rs panel_docking.rs` is empty;
the fork touches `lib.rs` (module line), `bar.rs` (3 hooks), `app.rs` (1 hook), `tree.rs`
(Object row), `ui_commands.rs`/`state.rs` (cfg rows) only.

### Phase 5 - Docs and visual evidence (S, 0.5 day)

- [ ] Update the guide/plan/ROADMAP references from "PubGen panel" to the menu paths.
- [ ] Screenshot: Publication menu open, compiler dialog, status bar after an AI action.
- [ ] Record the removal in `docs/plans/archives/` after merge (move this file, mark Done).

**Acceptance:** no doc tells the user to open a panel that does not exist.

Total: **3.5-5 days**, one developer.

---

## 8. Tests

### New unit tests

- [ ] `every_pubgen_menu_row_has_a_command` (walk both bar variants).
- [ ] `every_pubgen_command_has_a_row_or_dialog` (no dead ids).
- [ ] `command_ids_are_unique`.
- [ ] `compile_cancel_is_enabled_only_while_running` (uses `CompileJob::fake_running`).
- [ ] `ai_rows_follow_the_panel_gates` (doc x target x backend x bilingual).
- [ ] `compiler_rows_stay_enabled_without_a_document` (the generic UI-command fallback must not
      win).
- [ ] `status_surfaces_without_the_panel` (action -> `app.ui.status`).
- [ ] `retranslate_and_about_are_reachable`.
- [ ] `saved_layout_with_pubgen_panel_is_pruned` (JSON fixture).
- [ ] `pubgen_is_not_a_registered_panel` (`panel_label("pubgen") == "Panel"`).
- [ ] `window_menu_has_no_pubgen_row`.

### Updated tests

- [ ] `dock_extended::pubgen_menu_tests` -> `pubgen_bar` (menu render, dialogs render, ordering).
- [ ] `panels/pubgen/mod.rs` fixtures -> dialog fixtures (`CRAFT_UI_VISUAL_FIXTURES`).
- [ ] `panel_docking` tests lose their `#[cfg(feature = "pubgen")]` expectations.
- [ ] `menus/tests.rs` workspace snapshot no longer contains a `pubgen` panel.

### Manual checks

- [ ] Default bar: `Publication`, `Publication Tools`, `Shortcut`, `Help`; toggle shows the host
      bar plus the single `Pubgen` menu.
- [ ] Compile a sample manifest with the dialog closed; the `●` marker and status bar update,
      then the report appears when the dialog is opened.
- [ ] Cancel a compile mid-way; no output file is written.
- [ ] AI rows disabled without a document/backend/target, each with the right reason.
- [ ] Translate a review page, Fix Terms, Retranslate; the status bar reports the result.
- [ ] Window menu lists no PubGen panel; a workspace saved before this change opens intact.
- [ ] Control channel: `ui.menu.list` / `engine.commands` include the new ids.

### Gates

```
cargo fmt -p designcraft-ui-egui
cargo clippy -p designcraft-ui-egui --features pubgen --all-targets -- -D warnings
cargo test  -p designcraft-ui-egui --features pubgen
cargo check -p designcraft-ui-egui                 # no pubgen feature
cargo xtask wasm
cargo xtask ci
```

---

## 9. Upstream conflict budget after this plan

| File | Change | Size |
| :--- | :--- | :--- |
| `crates/ui-egui/src/lib.rs` | +2 module lines, -5 dock cfg seam | net -3 |
| `crates/ui-egui/src/menus/bar.rs` | 3 PubGen hooks (already there) | ~10 lines |
| `crates/ui-egui/src/app.rs` | 1 dialogs hook | 2 lines |
| `crates/ui-egui/src/menus/tree.rs` | Object > Chart Options row only (Window block removed) | ~8 lines |
| `crates/ui-egui/src/menus/ui_commands.rs` | cfg-gated PubGen rows | data only |
| `crates/ui-egui/src/menus/state.rs` | checked arm (minus panel) + enabled hook | ~5 lines |
| `crates/ui-egui/src/menus/run.rs` | one dispatch arm (existing) | ~5 lines |
| `crates/ui-egui/src/panel_docking.rs` | **reverted to upstream** | 0 |
| `crates/ui-egui/src/dock.rs` | **untouched** | 0 |
| `crates/ui-egui/src/dock_extended.rs` | **deleted** | -420 |
| `crates/ui-egui/src/menus/pubgen_bar.rs` | new, fork-owned | - |
| `crates/ui-egui/src/panels/pubgen/**` | fork-owned | - |

Rules unchanged from the earlier plan: every hook is `#[cfg(feature = "pubgen")]`, append-only,
marked `// pubgen:`, committed separately from upstream merges.

---

## 10. Risks and mitigations

| Risk | Impact | Mitigation |
| :--- | :--- | :--- |
| Compile progress disappears with the panel | High | compiler dialog + status bar + `●` marker; global poll already exists (`dialogs.rs:489`) |
| A saved workspace containing `pubgen` resets the whole dock | High | `pruned()` before the invalid-layout reset + JSON fixture test |
| Menu enablement drifts from the panel's old gates | Medium | one `pubgen_bar::enabled`, matrix test |
| AI one-shot rows run without feedback | Medium | status bar mirror + disabled reasons |
| `dock::pubgen` -> `pubgen` rename touches 10 files | Low | mechanical, compiler-checked; separate commit |
| Wasm build | Low | compiler rows/dialog `#[cfg(not(wasm32))]`, same as today |
| Panel fixtures stop compiling | Low | move them to the dialogs they now cover |
| Fallback `Pubgen` menu and the PubGen bar diverge | Medium | both call the same `pipeline()` / `content()` builders |
| Upstream refactors `menu_bar`/`run_ui_inner` | Low | hooks fail loudly at compile time; re-apply |

---

## 11. Rollback

Phases are separate commits. Restoring the panel means reverting the phase-3 commit (re-adding
`PUBGEN_PANELS`, the body arm, `show` and its tests, the Window row and the command) and the
phase-4 commit (restoring `dock_extended.rs` and the `lib.rs` seam). The menu and dialogs keep
working throughout, so the rollback is purely additive.

---

## 12. Open decisions

1. **Menu naming:** keep `Publication` + `Publication Tools` (recommended; matches the 2026-10-30
   UX plan) or merge them into one `Pubgen` menu. The single `Pubgen` fallback stays either way.
2. **Compile from the menu:** async + dialog (recommended) or keep the synchronous
   `pubgen.compileProject` behaviour for the menu row too.
3. **AI page actions:** one-shot rows with a status line (recommended, matches the panel) or rows
   that open the AI dialog with the target preselected.
4. **Status destination:** the app status bar (recommended) or a transient toast near the canvas.
5. **Keep the single `Pubgen` fallback menu and the "Show DesignCraft Menus" toggle?** Recommended
   yes until the bar split has a release of feedback.
