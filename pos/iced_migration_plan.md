# Iced UI Migration Plan

Migration of the POS desktop UI from **egui/eframe** (immediate mode) to **iced** (Elm architecture, retained layout).

- Branch: `feature/iced-ui-migration`
- Target framework: **iced 0.14** (plan originally said 0.13; 0.14 was latest stable at Phase 0 — features used: `tiny-skia`, `tokio`, `canvas`, `advanced`)
- Guiding rule: **core logic never changes.** Only the 4 UI-coupled files are rewritten; every other module (`ingest.rs`, `vector.rs`, `translate.rs`, `toc.rs`, `covers.rs`, `profiles.rs`, `project.rs`, `glossary.rs`, `header_footer.rs`, `audit.rs`, `i18n.rs`, `control_client.rs`, `process.rs`, `slots.rs`, `main.rs`) stays as-is.

### Progress log (actuals)

| Phase | Status | Commit | Notes |
|---|---|---|---|
| 0 Spike | ✅ done | `cd2dbfb` | 0.14 not 0.13; winit fork patch coexists fine; `text("").size(0)` crashes cosmic-text |
| 1 AppCore | ✅ done (revised) | `6a3a6c8` | pub fns instead of methods; encoding incident fixed in `e944f55` |
| 2 Skeleton | ✅ done | `0422dda` | `UiApp` owns `main_ui::App`; message/update/view/theme split |
| 3 Chrome | ✅ done | `b4ad646` | `window::drag_resize(id, Direction)` — all 8 edges; OS theme via `iced::system` |
| 4 Widgets | ✅ done | `78efe6b` | menu component, dc_pill, collapsible, drag_number, combo/text helpers |
| 5 Top bar | ✅ done | `321c3f6` | all ~50 menu ids wired to `panels::` fns (sync); Home/Compile/pill/lang in strip |
| 6 Home | ✅ done | `e130324` | hero, pill actions, DC banner, canvas profile cards w/ hover redraw, recent projects |
| 7 Studio | ✅ done | `ac8480b` | left/center/right/bottom full port; profile `combo_box` (searchable); +1225 lines |
| 8 Async | ✅ done | `7916320` | snapshot-detach-merge via `spawn_blocking`; rfd dialogs auto off-thread; DC probe subscription; busy-gated Compile |
| 9 Canvases | ✅ done | `6b32205` | cover/chart/spreads `canvas::Program` ports 1:1; per-mode toolbar; SVG clipboard copy |
| 10 Metadata form | ✅ done | `015382c` | `ui/metadata.rs` left panel form + publication cards grid, indexed CRUD messages, margin steppers w/ clamp, tests |
| 11 Cut over & polish | ✅ done | — | `iced` is the only UI; window icon/title/min-size wired; winit fork audited and kept (shared Craft IME fork, not egui-specific); build docs updated |
| 12 Remove egui | ✅ done | — | logic moved to `actions.rs`; `panels.rs`, `chrome.rs`, egui/eframe deps removed; `grep egui app/src` → 0 hits; 63/63 tests green |

---

## 1. Current State (audit)

### 1.1 File inventory — egui coupling

| File | Size | egui refs | Coupling | Migration unit |
|---|---|---|---|---|
| `panels.rs` | 137 KB | 553 | Heavy — all panels, menus, canvases, actions | Phases 4–9 |
| `metadata_form.rs` | 39 KB | 29 | Medium — editor UI + pure `Metadata` model | Phase 10 (split model out) |
| `main_ui.rs` | 11 KB | 13 | Heavy — eframe entry, `App` struct, theme, polling | Phases 2–3 |
| `chrome.rs` | 6 KB | 13 | Heavy — borderless window chrome | Phase 5 |
| all others (15 files) | ~190 KB | 0 | **None** | untouched |

### 1.2 Widget inventory (what must be mapped)

| egui widget | Count | iced 0.13 equivalent |
|---|---|---|
| `RichText` / `Label` | 46 | `text()` + `font::size`, `font::weight`, styling |
| `ComboBox` | 13 | `PickList` (fixed options) / `combo_box` (custom + editable) |
| `TextEdit` (singleline) | 10 | `text_input` (+ `on_input` message per field) |
| `CollapsingHeader` | 9 | custom `collapsible` component (`container` + state in UI model) |
| `Frame` | 7 | `container` with `style` / custom `widget::Container` styling |
| `Button` | 7 | `button` (+ styling module) |
| `ScrollArea` | 6 | `scrollable` |
| `DragValue` | 5 | **no equivalent** — build `drag_number` custom widget or use `text_input` + validation (decision in Phase 0) |
| `Grid` | 4 | `column` + `row`, or `Grid`-style component; metadata form → custom 2-col grid component |
| `MenuBar` (+ 6 menu fns) | 1 | **not in core** — custom `menu_bar` component (button + popover `opaque`/`mouse_area`, open-menu state in UI model) |

### 1.3 Custom painting (rewritten as `widget::Canvas` programs)

- `render_profile_cards_grid` — 3 clickable home cards with hand-drawn mini page mockups (`painter.rect`, `line_segment`, `text`)
- `render_cover_canvas`, `render_chart_canvas`, `render_spreads_canvas` — preview renderers (Pure
- status pill in top bar (`rect`, `circle_filled`, `text`)
- `chrome.rs` caption buttons (minimize/maximize/close glyphs drawn with `line_segment`/`rect_stroke`)

### 1.4 Interaction patterns that must be re-architected

1. **Direct mutation.** Every widget callback currently mutates `App` fields in place (`&mut entry.title`, `app.selected_profile = ...`). In iced: each becomes `Message` → `update()` handler. Estimated message variants: **120–200**.
2. **Blocking work on UI thread.** `launch`, `translate`, `batch_fill`, `ingest_linked_narratives`, `import_sources`, `run_qa_audit`, `generate_vector_chart`, HTTP via `ureq` (`ControlClient`) all run synchronously. In iced: wrap in `Task::perform(...)` with `tokio::task::spawn_blocking` (or std thread + channel) so the UI never stalls.
3. **Polling.** `poll_designcraft` + `request_repaint_after(400ms)` → `Subscription` using `time::every` + health-check task.
4. **Window chrome.** `ViewportCommand::StartDrag/BeginResize/Minimized/Maximized/Close` → `iced::window` ops (`window.drag()`, `window.maximize()`, `window.close()`); resize borders via `mouse_area` + `window.resize_direction` handling or `event::WindowEvent` cursor tracking (custom `resize_border` component).
5. **File dialogs.** `rfd` is framework-independent — keep it, but call it inside `Task` on a blocking thread.

---

## 2. Target Architecture

### 2.1 Module layout (new)

```
app/src/
  main.rs               # unchanged (dispatch run())
  main_ui.rs            # becomes thin: chooses egui or iced entry
  ui/                   # NEW — all iced code lives here
    mod.rs              # PosApp state + run()
    message.rs          # Message enum (single, flat, grouped by area)
    update.rs           # update() handlers (thin wrappers → app logic)
    view.rs             # view() — composes screens
    subscription.rs     # DC health polling, window events
    theme.rs            # Palette for Dark/Light/System, spacing, text styles
    chrome.rs           # port of chrome.rs (titlebar, caption buttons, resize borders)
    menu.rs             # reusable menu_bar component
    widgets/
      mod.rs
      drag_number.rs    # DragValue replacement (Phase 0 decision)
      collapsible.rs    # CollapsingHeader replacement
      pill.rs           # DC status pill (canvas-backed)
    pages/
      home.rs           # welcome page + profile cards + recent projects
      studio.rs         # 3-column workspace composition
    panels/
      top.rs, left.rs, center.rs, right.rs, bottom.rs
    canvas/
      cover.rs, chart.rs, spreads.rs, profile_cards.rs
    metadata.rs         # port of metadata_form editor UI
  metadata_form.rs      # slimmed: Metadata model + pure logic only
  panels.rs             # deleted at the end (Phase 11) — logic fns moved first
```

### 2.2 State separation

Current `App` mixes UI state and orchestration. Split:

- **`AppCore`** (existing `App` minus eframe impl) — project, dc/client handles, log, recent projects, LLM config. Unchanged fields; gains `&mut self` methods extracted from `panels.rs` action fns (`run_with_client`, `launch`, `kill_dc`, `translate`, `batch_fill`, `import_*`, `save_project`, ...). These already exist as free fns taking `&mut App` — **move them onto the struct, no logic edits**.
- **`UiState`** (new, in `ui/`) — theme, ui_lang, view, center_tab, preview_mode, menu open-state, collapsible open-state, text-input drafts (iced needs owned input strings; today TextEdit borrows fields directly — acceptable to keep borrowing `AppCore` fields where they're `String`s, since iced text_input takes `&String` + `on_input(Message)`).
- `update(msg: Message) -> (Task<Message>)` mutates both.

### 2.3 Message design (sketch)

Flat enum grouped by area, ~150 variants expected:

```rust
enum Message {
    View(ViewMsg),      // Home/Studio switch, tab select, preview mode
    Theme(AppTheme), ThemeLang(UiLang),
    Menu(MenuMsg),      // menu open/closed, per-item actions
    Chrome(ChromeMsg),  // window drag/resize/min/max/close, caption hover
    Input(InputMsg),    // per-field text edits (llm_base, glossary_term, page titles, ...)
    Combo(ComboMsg),    // picklist selections (chart_type, cover_style, page_kind, ...)
    Action(ActionMsg),  // compile, translate, import, export, discover, batch_fill, ... → Task
    Dc(DcMsg),          // launched, exited(status), connected(ControlClient), health(Ok/Off)
    Log(String),
    Project(ProjectMsg),
    Metadata(MetadataMsg), // from metadata form, incl. page card add/remove/edit
}
```

Every `ActionMsg` handler returns `Task::perform` for blocking work; results arrive as `Dc`/`Log`/`Project` messages.

### 2.4 Cargo strategy

- Add `iced = { version = "0.13", features = ["wgpu", "tokio", "image", "canvas"] }` (verify feature names at Phase 0) + `tokio` for blocking tasks.
- Keep `egui`/`eframe` deps during migration; both UIs coexist. `cargo run --features iced-ui` launches the new UI.
  - **Final state (Phases 11–12):** the feature flag was removed; `iced`/`tokio`/`image` are normal dependencies and egui/eframe are gone from the tree.
- `wgpu` vs `tiny-skia`: try `tiny-skia` first for lighter builds; fall back to `wgpu` if canvas/text rendering disappoints.

---

## 3. Phases

Each phase: compiles, runs, and is testable on its own. Order chosen to de-risk the hard parts early.

### Phase 0 — Spike & decisions (0.5 day) — **DONE** (`cd2dbfb`)
- [x] iced added behind `iced-ui` cargo feature; **0.14** pinned (default-features off; features `tiny-skia`, `tokio`, `canvas`, `advanced` — `advanced` needed for `canvas::Geometry` path).
- [x] Borderless window runs on Windows; `window.drag()` works; workspace `[patch.crates-io]` winit fork does **not** conflict with iced (Cargo.lock resolves iced_winit against the fork).
- [x] Menu dropdown prototype works (~100 LOC, `mouse_area` + `opaque` catcher).
- [x] DragValue decision: custom `drag_number` component (scroll + drag + buttons), not text-input.
- [x] Gotchas recorded: `text("").size(0)` panics cosmic-text ("line height cannot be 0") → use `space()`; `Settings.window` moved to `.window()` builder in 0.14; `ScrollDelta` is an enum; `Color::lighten` absent → wrote `theme::lighten`; `PickList` uses `text_size()` + `V: ToString`; `Rule::Style` has no `Default`.

### Phase 1 — Extract `AppCore` (0.5–1 day, zero UI change) — **REVISED, DONE**
> Implemented with a lower-risk approach than planned: instead of converting all free fns to `&mut self` methods (massive `app`→`self` rename + call-site churn), the ~28 logic fns in `panels.rs` were made `pub fn(app: &mut App)` unchanged. The iced `ui::App` owns an instance of `main_ui::App` (fields are pub) and calls `actions::launch(&mut app)` directly. (Phases 11–12 later moved these fns to `actions.rs` and deleted the egui `App` impl.) Same capability, zero refactor risk.
- [ ] Move `App` struct + `new/load_recent_projects/save_recent_projects/add_recent_project/log/set_publication_profile` out of eframe impl (drop `impl eframe::App` from it into a new struct wrapper or keep and add `impl eframe::App for UiAdapter`).
- [ ] Move free fns from `panels.rs` (lines ~1029–2057: `run_with_client`, `launch`, `kill_dc`, `build_translator`, `import_glossary`, `export_glossary`, `translate`, `overset_pass`, `write_converted`, `ingest_linked_narratives`, `import_sources`, `import_excel_metadata_from_path`, `load_template`, `open_project_path`, `open_project`, `save_project`, `discover`, `fill_live`, `build_batch_script`, `batch_fill`, `open_in_designcraft`, `open_pdf_viewer`, `run_qa_audit`, `export_cover_svg`, `generate_vector_chart`, `generate_choropleth_map`, `extract_toc`, `load_sample_brs`, `pick_file`) onto `App` as `&mut self` methods. Signature-only changes.
- [ ] egui UI still works (calls `app.method()` instead of `fn(app)`).
- **Exit:** `cargo test` green; app behaves identically; `panels.rs` now contains only `fn xxx(ui: &mut egui::Ui, app: &mut App)` view fns.

### Phase 2 — iced skeleton (1 day) — **DONE** (`0422dda`)
- [x] `ui/message.rs` flat enum + `ui/update.rs` handlers + `ui/view.rs` screens (no separate `subscription.rs` — subscription fn lives in `ui/mod.rs`).
- [x] `time::every(400ms)` tick subscription (DC poll handler placeholder, real port = Phase 8).
- [x] `ui/theme.rs`: `AppTheme` → iced `Palette` (`pubgen-dark`/`pubgen-light`, colors `#1F2228`/`#F0F2F6`/accent `#1A73E8`); `theme::lighten` helper.
- [x] i18n: `i18n::tr` reused verbatim (framework-free confirmed).
- **Exit met:** borderless window, theme + lang switch, tick fires.

### Phase 3 — Window chrome (1–2 days) — **DONE** (`b4ad646`)
- [x] `ui/chrome.rs`: title bar press-drag → `window::drag`; caption buttons as `canvas::Program` vector glyphs (no font dependency, Windows-native hover incl. close red `#E81123`); double-click toggles maximize.
- [x] Resize: **`window::drag_resize(id, Direction)` exists in 0.14** — exact winit `BeginResize` equivalent. 8-zone overlay (edges 4px, corners 10px, center pass-through), correct diagonal cursors (`ResizingDiagonallyUp`→`NeswResize` verified in iced_winit conversion.rs); overlay disabled while maximized.
- [x] Maximize sync: no maximized-changed event → `is_maximized` query chained after every `WindowResized`.
- [x] OS theme: `iced::system::theme()` at startup + `theme_changes()` subscription — `AppTheme::System` follows OS live (review finding resolved).
- **Exit met** (pending visual user check: edge/corner resize, caption hover, dbl-click).

### Phase 4 — Reusable widgets (1–2 days) — **DONE** (`78efe6b`)
- [x] `widgets/pill.rs` — DC status pill (container + colored dot glyph + label, hover tint, border = status color; NOT canvas-backed — simpler and looks right).
- [x] `collapsible` helper in `widgets/mod.rs` (generic fn, open state = `UiApp.open_panels: BTreeSet<&'static str>`); port of the 9 usages happens with their panels (Phases 7/10).
- [x] `ui/menu.rs` — declarative `MenuDef`/`MenuItem` model (`Action`/`Selected`/`Header`/`Separator`); popover under open title; sibling titles stay clickable while open (menu switching works).
- [x] `widgets/mod.rs` shared helpers: `action_button` (primary/secondary), `text_row` (label + `text_input`), `combo_row` (`PickList`, `V: ToString + Hash`), `widgets/drag_number.rs` component.
- **Exit met:** widgets live in the window; menu reflects real app state (checkmarks).

### Phase 5 — Top bar + menus (1 day) — **DONE** (`321c3f6`)
- [x] Top strip (below window titlebar): Home ⌂ toggle, menu bar, centered project title, accent **Compile** button (`batch_fill(None)`), DC pill (launch/reprobe), ui-lang `PickList`.
- [x] `run_menu_action()` dispatches ~50 stable ids to `panels::` fns — **sync calls** (acceptable per risk register; Task wrap = Phase 8).
- [x] `UiLang`: added `Hash` derive + `Display` impl (PickList bounds); `import_excel_metadata` made pub.
- [x] Rejected a `Box::leak`-per-frame id approach in favor of literal static ids (memory-leak bug caught before commit).
- **Exit met:** every menu item triggers its action; theme switch live-updates palette; language switch live.

### Phase 6 — Home page (2 days) — **DONE** (`e130324`)
- [x] `ui/home.rs`: hero title + bilingual subtitle, pill action row (+ New File / Open / Import XLSX / ⚡ Open Sample BRS conditional), DesignCraft banner card with monogram badge + purple launch button.
- [x] Profile cards: `canvas::Program` (`CardProgram`) with exact painter-mockup port (DDA facing pages, BRS banner page, Generic ruled page); hover = bg lighten + accent 1.5px border via `CursorMoved → request_redraw`; click → new project + profile + Studio.
- [x] Recent projects: empty-state card or bordered list with 📄 name/path + open button.
- [x] `view.rs` routes `AppView::Home ↔ Studio`.
- **Exit met** (canvas text via widgets below canvas, not frame.fill_text — simpler and sharper).

### Phase 7 — Studio: left/center/right + bottom (3–4 days) — **DONE** (`ac8480b`, biggest phase)
- [x] `ui/panels/{left,center,right,bottom}.rs` — full 3-column port + status bar.
- [x] Left: content tree, AI panel (`secure()` key input), glossary CRUD + CSV import/export, translate + diff list, cover studio (PickList + clipboard AI prompt), visual assets (chart type + title), TOC generator (3 kinds + copy).
- [x] Center: live header/footer preview card (BRS/DDA resolvers), slot character-fit table (OK/OVER colors), QA checklist + issues, action bar, tab bar, Log tab (`scrollable`).
- [x] Right: profile picker via `combo_box` (searchable — bonus vs egui), margins/spreads info, DC launch/kill/open, discover/fill, generate/x4, collapsible slot rows (budget input, source/translated text, Remove/Unname via IPC).
- [x] Bottom: pages/sourced/slots-filled counters + last output.
- [x] Additive core touches: `Display` for `CoverStyle`/`ChartType`/`PubgenProfile`, `Hash` on both enums.
- [x] Deferred (tracked): Content-tab metadata cards → Phase 10; Preview canvases → Phase 9; log auto-scroll (no bottom-offset API; manual scroll OK); slot text single-line vs egui multiline.
- **Exit met:** studio usable end-to-end (open project → edit → fill → compile).

### Phase 8 — Async actions (2 days) — **DONE** (`7916320`)
> Approach revision (implemented as written): **snapshot-detach-merge** — clone sendable `App` fields into a temp `App`, move the live `ControlClient` into it (same take() semantics as egui `run_with_client`), run the original fn on `spawn_blocking`, ship the temp `App` back as `Message::JobDone(JobPayload)` (`Arc<Mutex<Option<App>>>` — Message must be Clone for widget bounds), merge fields onto the real `App`. Logic fns stayed byte-identical; rfd dialogs run off-thread automatically.
- [x] ~20 slow ops detached: file dialogs, xlsx/docx imports, project open/save, translate, batch_fill ×3, discover, fill_live, QA audit, chart/map/cover/TOC generators, glossary CSV, zip auto-extract.
- [x] UI-instant ids stay sync: view/tab/theme/lang, launch/kill (process handle), exit, reconnect.
- [x] DC health probe: `Tick` → sync process-exit poll (`try_wait`) → gated async 250ms `connect_timeout` (500ms/2s interval) → `DcProbe(ProbePayload)` drives pill lifecycle.
- [x] Busy: single-flight guard (`UiApp.busy`) + Compile button grays to "Working…"; concurrent jobs logged + dropped.
- [x] Additive core: `Project` + `Clone` derive, `DcStatus` + `Clone, Copy, Default`; `tokio` (feature `rt`) added as optional dep tied to `iced-ui`.
- [x] Cleanup: dead demo code removed (`ValueMsg`, `menu_choice`, `ViewChanged`, `drag_value`); `drag_number` made closure-driven for Phase 10 reuse.
- **Exit met:** UI responsive during ops, no double-submit, pill lifecycle correct. Deferred: none known.

### Phase 9 — Preview canvases (2 days) — **DONE** (`6b32205`)
- [x] `ui/preview.rs` (single module, not `canvas/` dir): three `canvas::Program` ports.
- [x] Cover: all 4 styles 1:1 (Horizon banner+stripe+rule+catalogue, Modern sidebar, Tricolor bands, Minimal frame), live metadata w/ fallbacks, bilingual conditional, A4 aspect, paper shadow+stroke.
- [x] Chart: title + Donut (rings+"100%"+legend), Line (gridlines+polyline+markers), Bar (bars+year labels) for Bar/HorizontalBar.
- [x] Spreads: rows of 4 page cards (`canvas::Program` per card) — mini paper + kind badge colors, Hal. n, kind label, truncated title/source; bilingual empty state.
- [x] Toolbar: mode buttons + per-mode controls (style/chart PickLists, 📋 Salin SVG → `iced::clipboard`, 💾 Ekspor, ➕ Tambah ke Proyek).
- [x] Gotchas: `Canvas` has no lifetime param in 0.14 (owns Program); `fill_circle` absent → `Path::circle` + `fill`; `Text::default()` works for fill_text.
- **Exit met** (visual font-metrics check pending user).

### Phase 10 — Metadata form (2–3 days) — **DONE**
- [x] Split `metadata_form.rs`: models (`Metadata`, `PageEntry`, `PageLayout`, `Lang`, `Paper`, `Margins`, `PageItemScope`, `PageKind`, `ImageArea`, `ExcelCoverData`), `apply_profile`, and validation stay pure; added `Display`, `Hash`, `MARGINS_MIN`/`MAX`, `MarginSide`, `CoverField`.
- [x] Port margins `DragValue` using compact label + value + `[-]`/`[+]` step buttons with clamping `5.0..=100.0 mm`.
- [x] Port publication cards: dynamic 2-column card grid in `ui/metadata.rs`, per-card scope/kind/source picklists, title and chapter/title_en inputs, cover 14-field editor, chart type & image area combos, remove button (×), and bottom "+ Tambah Halaman Baru" / "+ Tambah Objek Baru" buttons.
- [x] Indexed messages in `MetadataMsg` handled in `ui/update.rs` (field updates, margin stepping, card CRUD, kind option syncing).
- [x] Tested: 64/64 tests green (`cargo test --features iced-ui`), 61/61 tests green (`cargo test`).
- **Exit met:** full metadata editing parity, card CRUD, bilingual editor path, and unit tests.

### Phase 11 — Cut over & polish (1 day) — **DONE**
- [x] Default feature flip: `iced-ui` on. During Phase 12 the flag itself was removed, so `iced` is unconditional.
- [x] **Audit `storytold/winit` fork delta vs upstream** — the fork is the Craft-suite language-support branch (W1–W27: Windows/macOS/X11/Wayland IME, DPI, clipboard, scroll fixes) based on upstream `v0.30.13`. It is not egui-specific and also patches iced's transitive `winit`, so the `[patch.crates-io]` entry is kept.
- [x] Window icon (`branding::iced_icon` + `winresource` build step), title, min-size (800×500), and recent-project persistence verified.
- [x] Visual polish: palette consolidated in `ui/theme.rs` (accent/background values are no longer duplicated in UI code). Full screenshot-by-screenshot comparison was not run in this pass.
- [x] Update `AGENT.md`/README build instructions for the iced-only build.
- **Exit met:** default build is iced; no egui path remains (completed together with Phase 12).

### Phase 12 — Remove egui (0.5 day) — **DONE**
- [x] Pure actions moved from `panels.rs` into `actions.rs`; `crate::actions::` call sites updated in `ui/update.rs`.
- [x] Deleted `app/src/panels.rs` and `app/src/chrome.rs`; removed the egui entry point and `impl eframe::App` from `main_ui.rs`.
- [x] Removed egui-only UI methods from `metadata_form.rs`, `branding::egui_icon`, and the egui/eframe dependencies from `app/Cargo.toml` + workspace `Cargo.toml`.
- [x] Re-ran `cargo check`, `cargo build`, `cargo test` (63/63 green), and `cargo clippy --all-targets`.
- **Exit met:** egui-free tree; `grep egui app/src --include="*.rs"` → 0 hits.

### Post-migration note (2026-11-03)
- App renamed to **POS (Publication Orchestrator Studio)**. Cargo package/binary is now `pos` (`pos.exe`), and every app-icon file uses the `pos` name (`pos.ico`, `pos.icns`, `pos.svg`, `hicolor/.../ai.storyteller.pos.*`).
- The icon is embedded in the executable (icon + VERSIONINFO) and shown in the custom title bar and Home hero, in addition to the native window/taskbar icon.
- Backward compatibility kept: `.pubgen` project format, `pubgen.json` manifest, `PUBGEN_*` environment variables, and the app-data folder are unchanged.
- Verified after the rename: `cargo test` 63/63 green, `cargo build` produces `pos.exe`, and `cargo run -- --smoke` reports `SMOKE PASS`.

---

## 4. Risk register

| Risk | Impact | Mitigation |
|---|---|---|
| iced borderless resize UX worse than egui `BeginResize` | Medium | **Resolved** — `window::drag_resize(id, Direction)` in 0.14; all 8 zones ported (`b4ad646`) |
| Menu/dropdown component quality | Medium | **Resolved** — custom declarative menu_bar shipped (`78efe6b`); no iced_aw needed |
| 120–200 message variants → combinatorial update bloat | Medium | Actual so far: 14 top-level variants + flat `&'static str` action ids routed in one dispatcher — leaner than planned |
| Canvas text metrics differ (labels in previews) | Low | Accept minor diff; custom font load if unacceptable |
| Blocking port churn during Phase 7 (actions still sync) | Low | **Resolved** — Phase 8 detached all slow ops (`7916320`) |
| iced version churn | Medium | **Resolved** — pinned 0.14 (not 0.13); egui fallback kept until Phase 12 |
| Hidden egui dep leaks into "pure" modules later | Low | CI grep gate: `! grep -rn "egui::" app/src --include="*.rs" | grep -v ui/ | grep -v main_ui.rs` (until Phase 12: zero outside UI files) |
| PowerShell text cmdlets corrupt UTF-8 (bit us in Phase 1: `Get-Content` misreads UTF-8 as ANSI, `Set-Content -Encoding utf8` adds BOM) | High if triggered | Never rewrite whole source files with `Get-Content`/`Set-Content`. Use the Edit tool or .NET `[System.IO.File]::ReadAllText/WriteAllText` with explicit `[System.Text.UTF8Encoding]::new($false)` |
| `git add -A` sweeps unrelated user worktree changes into commits | Low | **Triggered once** (`pgs_logo`/`pos_logo` swept into Phase 3 commit, split back out). Stage explicit paths only |
| `storytold/winit` git-fork patch applies to iced's stack too (supply chain + API drift) | Medium | **Resolved** — audited in Phase 11: it is the shared Craft IME/language-support fork (W1–W27) on upstream `v0.30.13`, not an egui workaround; kept because iced's `winit` needs the same input fixes. |

### Open suggestions from branch review (do during their phases)

- Palette consolidation (accent/`#1F2228`/`#F0F2F6` duplicated between egui inline colors and `ui/theme.rs`) → Phase 4/11 polish
- Margins clamp range `5..100` duplicated from `metadata_form.rs` `DragValue.range` → Phase 10 (export `MARGINS_MIN/MAX`)
- Window title/size already de-duplicated via `APP_TITLE`/`DEFAULT_WINDOW_SIZE` consts ✅ (`e944f55`)
- **GUI Usability & UX Review (User Perspective)**: comprehensive findings and prioritized UX recommendations documented in [`gui_design_review.md`](file:///e:/Github/designcraft-pubgen-0.7.X/standalone/gui_design_review.md) covering primary action consolidation, metadata terminology disambiguation, engine status humanization, and left panel accordion grouping.

---

## 5. Testing strategy

- Existing unit tests (`main_ui.rs` tests, others) keep passing — they don't touch egui.
- Phase 1 is pure refactor: `cargo test` must stay green; that's the safety net.
- Per-phase manual smoke script (checklist): open sample project → edit metadata → discover → fill live → compile → preview tabs → log — run on both UIs until Phase 12.
- New unit tests: message→state handlers for `InputMsg`/`ComboMsg` (pure), `UiState` transitions, palette mapping. Canvas programs get golden-image tests only if time permits (skip: visual check suffices).

---

## 6. Effort summary

| Phase | Est. | Cumulative |
|---|---|---|
| 0 Spike | 0.5 d | 0.5 d |
| 1 AppCore extraction | 0.5–1 d | 1.5 d |
| 2 Skeleton | 1 d | 2.5 d |
| 3 Chrome | 1–2 d | 4.5 d |
| 4 Widgets | 1–2 d | 6.5 d |
| 5 Top bar | 1 d | 7.5 d |
| 6 Home | 2 d | 9.5 d |
| 7 Studio panels | 3–4 d | 13.5 d |
| 8 Async | 2 d | 15.5 d |
| 9 Canvases | 2 d | 17.5 d |
| 10 Metadata form | 2–3 d | 20.5 d |
| 11 Cutover | 1 d | 21.5 d |
| 12 Remove egui | 0.5 d | 22 d |

**Total: ~3–4.5 weeks solo, sequential.** Phases 2–4 and 9 are partially parallelizable. Biggest schedule risk: Phase 0 spike results (menu + resize + DragValue) — re-baseline after it.

**Actual pace (Phases 0–9 done in ~3 session-days, well ahead of the 17.5d estimate).** Both de-risk spikes resolved better than planned (`drag_resize` exists; menu shipped clean). Remaining estimate: ~1 week for Phases 10–12.
