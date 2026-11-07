# GUI consistency review: Publication Orchestrator vs. DesignCraft

Status: open — F1 fixed (2026-11-07), see §8 · Scope: `apps/pubgen-studio` + `crates/ui-egui` Orchestrator surface · Reference: DesignCraft mode (same shell, same tokens) · Base: `studio` @ `34c4ac6f`

Product decisions that scope the work: §7 (2026-11-07) — dense tool surface, mode pills as chrome, ID
primary with complete EN, two columns.

Goal: the Orchestrator mode of PubGen Studio must read as the same application as DesignCraft mode.
DesignCraft is the reference. Anything the Orchestrator renders that is not reachable from
`theme::Tokens` or `widgets::*` is, by definition of this repo's UI contract, a defect.

## 1. Method and evidence

- Ran both modes of the shipped debug binaries:
  - `target\debug\pubgen-studio.exe` (Orchestrator) captured with Win32 `PrintWindow(hwnd, hdc, 2)`;
    that binary has no control channel, so no `ui.screenshot`.
  - `target\debug\designcraft.exe --sample --control 7981` + JSON `ui.screenshot` for the reference.
- Captured **both** Orchestrator screens: the Home/start screen, and the Worksheet (*Studio*) screen by
  launching `pubgen-studio` with an existing package
  (`plan\pubgen\samples\brs\Perkembangan Nilai Tukar Petani.pubgen`); a launch argument pointing at a
  missing `.pubgen` silently leaves the app on Home, which is how the first pass missed the Worksheet.
- Sampled pixel values from the captures; cross-checked every sampled colour against `theme.rs`.
- Read the implementations: `crates/ui-egui/src/panels/pubgen/workspace.rs` (4 666 lines — the entire
  Orchestrator body), `crates/ui-egui/src/menus/pubgen_bar.rs` (936), `crates/ui-egui/src/chrome/*`,
  `theme.rs`, `widgets.rs`, `panel_docking.rs`, `icons.rs`, and everything under `apps/pubgen-studio/src/`.
- Adversarial re-verification pass over 8 claims; counts and line numbers below are from that pass
  (committed state at `34c4ac6f`).

Measured values worth keeping:

| Sample | Orchestrator | DesignCraft |
|---|---|---|
| app bar | `#535353` = `t.app_bar` | `#535353` = `t.app_bar` |
| content background | `#424242` = `t.panel_darker` (Home) / `#5E5E5E` = `t.pasteboard` (Worksheet centre) | `#5E5E5E` = `t.pasteboard` |
| accent seen | `#2563EB` (Tailwind blue-600) | `#378EF0` = `t.accent` |
| mode pill fill | `#2A574D` (literal) | n/a |
| status bar | 17 px band, blank (text fields all empty) | zoom / page / preflight / spread count |
| tab strip | host tabs 28 px `#424242` inside the central panel | document tabs 35 px `#424242` in the chrome band |
| right dock | 320 px fixed, `#535353` body, 27 px strip | 256 px default, resizable, 27 px strip |
| card surface | `#535353` = `t.panel`, 1 px border, radius 2 | n/a (no card concept) |
| status dot / link | `#4ADE80` green-400, `#38BDF8`-family cyan | tokens only |

Local captures (untracked, not committed): `.ui-shots\orchestrator-home.png`,
`.ui-shots\orchestrator-worksheet.png`, `.ui-shots\designcraft.png`.

## 2. Verdict

Orchestrator is a second visual language mounted inside DesignCraft's shell. It inherits the app bar,
the status bar and the `Tokens` plumbing, then renders a Tailwind + emoji landing-page aesthetic on top.
Consistency: roughly 40 %. The single most damaging part is not taste — it is that the Orchestrator
renders unreadable text in the Light and MediumLight themes, because it paints hardcoded white over
token-driven light surfaces.

## 3. Scorecard

| Dimension | State | Verdict |
|---|---|---|
| Surface/text colours | `t.panel`, `t.panel_darker`, `t.text`, `t.text_dim` | matches |
| Accent/status colours | 64 literal colours, ~50 unique Tailwind hexes | broken |
| Typography | 10 label sizes (11.0 ×57, plus 9.0–24.0) vs one `LABEL_SIZE = 11.0` | broken |
| Shape/radius | 52 `CornerRadius` sites, 43 at 2–4, pills 16, hero 10 vs `Tokens::radius = 1` | broken |
| Iconography | 39 emoji, 20 codepoints, NotoEmoji face; `icons.rs` used only in the dock | mixed |
| Chrome / IA | host bar kept, but own tab strip inside content, no search/picker/Share/Discord | partial |
| Theming (5 brightnesses) | nothing in production reads `Brightness`; white over light tokens | broken |
| Accessibility | 2 `widget_info` in 4 666 lines; 0 in `pubgen_bar.rs` | broken |
| Localization | 158 `tr()` calls, large ID-only surfaces, mixed-language frames | partial |
| Status bar | blank band in Orchestrator | partial |

## 4. Findings (ranked)

### F1 — Light/MediumLight themes render invisible labels (severity: high)

Galleys are laid out with token colours and then painted `Color32::WHITE`:

- `crates/ui-egui/src/panels/pubgen/workspace.rs:2326` — `tab_button` paints the centre tab label white;
  galley format was `t.text` (line 2322). On Light (`panel #f0f0f0`) and MediumLight (`panel #b8b8b8`)
  the label disappears.
- `crates/ui-egui/src/panels/pubgen/workspace.rs:1142` — host tab strip; galley built with
  `t.text_strong`/`t.text_dim` (lines 1110, 1113), painted white over `t.panel` / `t.tab_strip`.
- `crates/ui-egui/src/panels/pubgen/workspace.rs:2744` — tag pill painted white on `t.well`
  (`#cfcfcf` Light, `#9a9a9a` MediumLight).
- Green/red status text ignores the theme: `#4ade80` on `t.panel_darker` at
  `workspace.rs:2953`, `:2259`, `:2847`, `:2988` (~1.6:1 in Light); red `#f87171` at `:2265`, `:2849`, `:2990`.

`Brightness` is read only inside `#[cfg(test)]` (`workspace.rs:4492`, `panels/pubgen/mod.rs:450`,
`thumbnails.rs:985`), so nothing adapts.

Fix: paint with the format colour the galley already carries (`text.galley.job.sections[0].format.color`)
instead of `Color32::WHITE`; add `success`/`danger` tokens and use them for status text. DesignCraft's
`dock_extended.rs::selected_dock_tab_has_readable_rendered_contrast_in_every_theme` is the test to copy —
Orchestrator should fail an equivalent assertion today.

### F2 — Two palettes in one product (high)

`crates/ui-egui/src/theme.rs` states "Every colour the UI uses. Widgets never hard-code colours."
`workspace.rs` contains 63 `Color32::from_rgb`, 1 `from_rgba_premultiplied` (`:2350`) and 23
`Color32::WHITE`. The literal set is Tailwind:

| Tailwind | Sites |
|---|---|
| `#2563eb` blue-600 | `:1413`, `:2184`, `:2368`, `:2672`, `:3735`, `:3993` |
| `#0e7490` cyan-700 | `:2905` (Generate Publication) |
| `#166534` green-800 | `:2927` (Generate & Publish PDF) |
| `#dc2626` red-600 | `:3724`, `:4006` (destructive) |
| `#10b981` emerald-500 | `:1479`, `:2402`, `:3411` |
| chart palette `#8b1e2d…#c7d2fe` | `:2342–:2428` |

There is no `success`, `warning`, `danger` or `info` token in `Tokens` at all, which is why the author
reached for literals. Fix: extend the token struct first (all five brightnesses), then replace literals.

### F3 — Three product names in one window (medium)

- Window title: `"Pubgen Studio"` (`apps/pubgen-studio/src/main.rs`).
- Centre app-bar title: unconditionally `format!("DesignCraft — {}", doc.title())`, no mode branch —
  `crates/ui-egui/src/chrome/app_bar.rs:436-439`. In Orchestrator it is wrong (and, at 1440 px with the
  8 PubGen menus, it is suppressed entirely by the fit check).
- Mode pill text `"Designcraft"` (lowercase c, untranslated) — `pubgen_bar.rs:91`; menu item
  `"🎨 Switch to DesignCraft Studio"` — `pubgen_bar.rs:239`; View menu `"Publication Orchestrator"` vs
  `"DesignCraft Layout Studio"` — `:264`.

Fix: one product name per mode supplied to the app bar; normalise the pill label to "DesignCraft".

### F4 — Emoji instead of the existing icon set (medium)

39 emoji / 20 codepoints in `workspace.rs` (📊 ⚡ 🚀 🗑 💾 📑 👁 🔍 ⚠ ➡ ● ▼ ▶ …), plus flag emoji in the
menu. `crates/ui-egui/src/icons.rs` already provides `paint` (`:1254`) and `button` (`:1267`) with named
glyphs (`"panel-properties"`, `"panel-articles"`, `"double-chevron-right"` `:421`, `"home"`, `"search"`),
and the dock already uses them (`workspace.rs:1679`, `:1699`, `:1737`).

Emoji do render monochrome (`monochrome_emoji_fonts`, `Cargo.toml:31`; fallback faces appended in
`theme.rs:480-524`), so colour is not the problem — the *typeface* is: NotoEmoji has different weight and
advance width than Source Sans 3, so labels visibly jump. 🇮🇩/🇬🇧 have no monochrome glyph; the fallback
(regional-indicator letters vs tofu) is unverified.

Fix: replace emoji with `icons::paint`; add the handful of missing names (`close`, `chevron-left`,
`spark`, `download`) rather than painting `×`/`«`/`✓` as text (`workspace.rs:1136`, `:1659`).

### F5 — Type and radius are ad hoc (medium)

`workspace.rs` label sizes: `semibold` 10.0/10.5 ×11/11.0 ×57/11.5 ×6/12.0 ×9/12.5 ×2/13.0/13.5 ×4/15.0 ×2/24.0;
`FontId::proportional` 9.0/9.5/10.0 ×6/10.5 ×27/11.0 ×8/11.5/12.0/14.0/16.0/22.0. `CornerRadius::same`:
1 ×8, 2 ×13, 3 ×13, 4 ×17, plus 10 (hero) and 16 (home pills).

Reference: `widgets.rs:20` `LABEL_SIZE = 11.0`, `:14` `FIELD_H = 21.0`, `:18` `BUTTON_H = 20.0`;
`theme.rs:126` `radius = 1`; window/menu radius 4 (`theme.rs:564`).

Fix: collapse to 11.0 body + one heading (15–16) + one hero (24, Home only); radius from tokens; if
pill-shaped buttons belong to the product, add `Tokens::radius_pill` rather than a literal 16.

### F6 — Dock parity is close; tab handling is not (medium)

Matches: 27 px strip (`panel_docking.rs:1139` vs `workspace.rs:1719`), `t.tab_strip`, `t.panel`,
`t.text_strong`/`t.text_dim`, 1 px border, accent colour (`panel_docking.rs:1141-1146`).

Drift: reference tab label is `FontId::proportional(12.0)` left-aligned (`surface.rs:174`,
`render/group.rs:406`, `TAB_PADDING 24.0`); Orchestrator centres `semibold(11.0)` and adds a 2 px accent
top line the reference never draws (`workspace.rs:1749`, `:1777`). Reference collapse is an 18 px
full-height column bar with a `widget_info` label (`render/dock.rs:23`, `:598`); Orchestrator's is a
22×23 px in-strip button with a vector icon and no label (`:1729-1737`).

Worse, one Orchestrator screen carries **three rival tab treatments** (verified on the Worksheet capture):
host tabs 28 px `t.tab_strip`, `semibold(11.5)`, square (`workspace.rs:1116`); dock tabs 27 px, 11.0, 2 px
accent top line (`:1719`); and the centre segmented control (`📑 Page Structure / 👁 Visual Live Preview /
📋 Execution Log`) which is a third style — ~24 px, radius 3, `t.well` + `t.field_border` fill, semibold
11.5, painted white (`:2314-2337`). DesignCraft has exactly one tab component. The host tabs also render
*inside* the central panel directly above the dock's tab bar, so the right column shows two stacked tab
bands; DesignCraft's document tabs live in the 35 px chrome band. And the collapsed-dock tooltip says
`"{Sources & Content}: {Click to edit}"` (`:1641-1709`) — a tab is not an edit affordance.

Fix: one tab component, one height; host tabs into the chrome tab-strip band; copy the reference's
collapse bar metrics.

Roles are also unstated (verified against the Worksheet capture, now specified in §7.6): tab 1 is the
publication's *information* read from the `.pubgen`, tab 2 is *generation* into DesignCraft pages — but the
labels ("Sources & Content" / "Profile & Execution") describe neither cleanly, and the generation facts
float outside their tab: `● Clean (QA Fit 100%)` sits in the centre header, the Generate buttons in the
right panel's lower block, `Source: …` + `25 embedded content file(s)` in tab 1. The default tab already
matches the intended flow (`right_dock_tab: OrchestratorRightTab::SumberKonten`, `workspace.rs:518`), and the
labels stay as shipped (§7.6), so only placement and the leakage of generation status need work.

### F7 — Accessibility dropped (medium)

`widget_info` count: 2 in `workspace.rs` (`:1119`, `:1132`), 10 in `widgets.rs`, **0** in `pubgen_bar.rs`.
Unlabelled click targets: mode pills (`pubgen_bar.rs:46-75`, only `.on_hover_text`), `tab_button`
(`:2314`), dock expand/collapse (`:1656`, `:1735`), dock tabs (`:1750`, `:1778`), margin steppers
(`:2183`, labels "-"/"+" only), cards (`:2563`), tag pills (`:2739`). Reference tabs register accesskit
roles (`craft-ui tabs/mod.rs:185-189`), as does the dock collapse (`dock.rs:598`).

Fix: `widget_info(Role::Tab/Button, ...)` on each painted hit area. This also fixes Playwright/kittest
automation and the control channel's `ui.click` targeting by name.

### F8 — Localization is half-applied (medium)

158 `crate::i18n::tr()` calls coexist with large Indonesian-only surfaces: QA log (`:2231-2250`), card
creation (`:2684-2732`), studio panel (`:2877-2994`), cover form (`:3256-3315`), guard dialog buttons
(`:3992` "💾 Simpan Versi Baru (v2)", `:4005` "⚠️ Timpa Tata Letak"), map header (`:3415`). English mode
only swaps `tr()` keys, so one frame renders ID and EN together. `pubgen_bar.rs` also hardcodes English
rows next to translated ones (`:333`, `:450`, `:497`). No emoji were found inside `tr()` keys (emoji are
concatenated outside the key), which is the right pattern — extend it.

Fix: key the literals; or, if that is out of scope, remove the EN switch until it is true end-to-end.

### F9 — Menu bar bypasses the host command system (medium)

`pubgen_bar.rs` paints `"✓  "`, `"●  "`, `"   "` prefixes on `ui.button` / `selectable_label` instead of
using `Item::Cmd` like `menus/bar.rs`. Consequences: no shortcut column, no disabled-reason, no unified
check state, no screen-reader name. The file's own comments cite the open accessibility audit (P1-3, P1-5,
P1-6) — those are work items, not remarks.

Fix: express rows as `Item::Cmd` and let `menus::state::checked` drive the checkmark; keep the running-job
dot as a status suffix on the item rather than in the label string.

### F10 — Dead chrome and an untrue comment (medium; cheap)

- `apps/pubgen-studio/src/chrome/top_bar.rs` — `render_top_bar` ("PUBGEN STUDIO" `#3B82F6`, 📁,
  "✔ Layout Active" `#10B981`, "⏳ Awaiting Compilation" `#F59E0B`, hardcoded 15.0 sizes) has **zero call
  sites**; `chrome/mod.rs` still re-exports it.
- `apps/pubgen-studio/src/workspaces/orchestrator/mod.rs` — `render_orchestrator` / `render_close_dialog`:
  stock `egui::Window`, `ui.heading("ðŸ“Š Publication Orchestrator")` (double-encoded mojibake), a
  `Color32::WHITE` text button, English-only. Unused.
- `apps/pubgen-studio/src/workspaces/designcraft/mod.rs` — `render_designcraft`, `"🎨 DesignCraft Layout
  Canvas"`, `Color32::GRAY`, hardcoded 16.0/14.0 sizes. Unused.
- `apps/pubgen-studio/src/app.rs` comment still claims an "Undecorated borderless custom topbar with Home
  icon, Mode Switcher, Menu Bar, Search, Workspaces, and Caption buttons" — in Orchestrator only Home, the
  pills, the menus and caption buttons exist; Search/Workspaces/Share/Discord are gated off by
  `if !is_orchestrator` (`app_bar.rs:297`).

Fix: delete the three modules, delete the comment, keep the mode-switch pills (they are the good part).

### F11 — Chrome semantics don't carry over (low/medium)

Status bar is an empty 17 px band in Orchestrator (measured: no glyphs); the reference shows zoom, page
navigation, `[Basic] (working)`, preflight state, spread count (`chrome/status_bar.rs`, no mode branch).
The right app-bar cluster collapses from search + workspace picker + Share + Discord to a lone language
selector whose flag is hand-painted (`app_bar.rs:377-431`, text padded with `"      {} ▾"`).

Fix: hide the status bar in Orchestrator, or replace its content with Orchestrator facts (active
publication, profile, running job, dirty state). Same for the right cluster: keep a search field (filters
cards), drop what has no meaning.

### F12 — Surface promises what the engine does not do (trust)

Save in the sources panel is a no-op that fakes success — `is_dirty = false` plus
`execution_log.push("Proyek publikasi berhasil disimpan ke cache.")` (`workspace.rs:1822-1900`); "🔍 Run QA
Audit" always reports "0 error, 0 warning" (`:2222-2232`); "+ Add New Page/Object" craft cards without
asking the engine. This is a design-consistency problem in the strict sense: DesignCraft never shows a
control that lies, and users carry that expectation across modes.

## 4b. Findings on the Worksheet screen specifically

Everything in §4 was measured on both screens; these four were only visible on the Worksheet capture
(`.ui-shots/orchestrator-worksheet.png`, sample `Perkembangan Nilai Tukar Petani.pubgen`, 14 content cards).

### F13 — Card metadata is clipped when the title wraps (layout bug, high visibility)

`measured_card_height` (`workspace.rs:2534`) lays out the title as one line and adds fixed
`PILL_H 16 / FOOTER_H 16 / LINE_GAP 2 / PAD_Y 12`; the card is then allocated that exact height. When a
title wraps to three lines (`"Tabel 1 Indeks Harga Diterima Petani, Indeks Harga Dibayar Petani per
Subkelompok Pengeluaran serta Perubahannya (2018=100)"`), the `Pg. N · Click to edit` row is cut in half by
the card's bottom border — reproduced on 3 of 6 visible cards in the capture ("Pg. 2", "Pg. 4", "Pg. 6").
Nothing warns; the row is simply swallowed.

Fix: measure the real wrapped galley (`ctx.fonts(|f| …)` with the card's `wrap_width`) instead of assuming
one line; assert in a test that a 3-line title still shows `Pg.` and the link. DesignCraft's
`widgets::section_row` uses a fixed 23 px row because its titles never wrap — the card grid cannot copy
that assumption.

### F14 — Right dock is fixed-width and cannot be resized (medium)

`show()` computes `right_w = if expanded {320.0.min(total_width*0.35).max(280.0)} else {38.0}`
(`workspace.rs:1565-1636`); there is no drag handle and no persisted width. DesignCraft docks are
user-resizable and remember size (`panel_docking.rs`, `craft-ui render/dock.rs` `COLLAPSE_WIDTH`,
drag bar with `widget_info`). On a 1440 px window the Worksheet gives 320 px to the dock forever; at
900 px minimum width the clamp (280 px floor) leaves 620 px for the card grid.

Fix: reuse the dock resizer; persist width with the workspace state.

### F15 — Settings block is a boxed form, the reference is sections and rules (medium)

The dock body wraps layout/language/paper in
`Frame::NONE.fill(t.panel_darker).stroke(1.0 t.field_border).corner_radius(4).inner_margin(8)`
(`workspace.rs:1900-2100`), then puts `Label: [ComboBox]` rows inside, and the margin steppers
`T/R/B/L` in a 2×2 grid of bright `#2563EB` square `-`/`+` buttons (`:2179-2202`, measured fill
`#2563EB`). Reference panels have no bordered boxes: `widgets::section` + 1.5 px `rule`, and numeric
values are `widgets::spin_measure` fields you can type into. Three accent hues now compete inside one
dock: blue steppers, cyan links (`#38bdf8`), green status (`#4ade80`).

Fix: `section`/`divider` rhythm, `spin_measure` for margins (typed value + steppers), one accent.

### F16 — Small copy/state defects visible on the Worksheet

- `Source:: csv/COV.csv` — doubled colon (`"Source: {source_label}"` where the label already ends in `:`,
  `workspace.rs:1827`); same on `Source:: md/POINT.md`, `Source:: md/U1.md`.
- Status bar is present but entirely blank in Worksheet too (17 px `t.panel` band, measured at y≈883-900),
  while the centre panel already prints `● Clean (QA Fit 100%)` in `#4ADE80` (`:2988`) — the same fact the
  reference puts in the status bar, rendered in a different place in a different colour.
- Card kind pills ("Sampul", "Halaman Baru") are 10 px white-on-`t.well` and clip their border at the
  card's inner margin; reference chips use `widgets::swatch_chip`-style metrics.

## 5. What already matches (do not regress)

- Surfaces, borders, text: `t.app_bar`, `t.panel`, `t.panel_darker`, `t.pasteboard`, `t.tab_strip`,
  `t.text`, `t.text_dim`, `t.text_strong` — verified in pixels and in code, on both screens.
- Worksheet centre column uses `t.pasteboard` (`#5E5E5E`, measured), i.e. it reads as the same "canvas"
  surface as a DesignCraft page background; cards sit on it in `t.panel` (`#535353`) with a 1 px border.
- Host tab band background is `t.tab_strip` (`#424242`, measured) — the same colour as the reference's
  document tab strip, so the *palette* is right even though the height/weight/placement are not.
- Dock form fields are `widgets::FIELD_H` tall (~22 px) with `t.input` / `t.field_border`; text entry looks
  like DesignCraft's.
- Right dock: 27 px strip, token colours, `icons::paint` for panel glyphs, 1 px borders.
- Mode pills sit in the same 30 px app bar as the reference menus; the shell does not jump between modes.
- Everything user-visible still goes through `app.run(...)`, so this is a rendering problem, not an
  architecture problem.

## 6. Suggested plan

Order matters: tokens first, because most later fixes then become find-and-replace.

**Step 1 — Tokens (no visual change intended, then a visible one).**
Add to `theme::Tokens` (all five `Brightness` values): `success`, `success_text`, `warning`,
`warning_text`, `danger`, `danger_text`, `info`, `brand` (the Product blue), and `radius_pill`. Derive
`success/warning/danger` from the existing palette so the InDesign-derived look is preserved
(e.g. success `#00c853` already exists as `smart_guide`; danger from `caption_close #c42b1c`).
Acceptance: `grep -c "Color32::from_rgb" crates/ui-egui/src/panels/pubgen/workspace.rs` → 0 except the
paper-mock arms of `paint_card_thumbnail` and the live-preview page mock, which get a dedicated
`Tokens::page_mock_*` group or move to `panels/pubgen/thumbnails.rs`.

**Step 2 — Legibility.**
Replace the four `Color32::WHITE` galley paints with the galley's own format colour; add a regression test
mirroring `dock_extended::selected_dock_tab_has_readable_rendered_contrast_in_every_theme` over
`tab_button`, host tabs, tag pills and status text, asserted for every `Brightness`.
Acceptance: contrast ≥ 4.5:1 in all five themes for tab label, host tab, tag pill, status text.

**Step 3 — Delete dead chrome and fix branding.**
Remove `apps/pubgen-studio/src/chrome/top_bar.rs` (and its re-export), `workspaces/orchestrator/mod.rs`,
`workspaces/designcraft/mod.rs`; correct the `app.rs` comment. Add `product_name(mode)` used by
`app_bar.rs:436-439`; pill label "Designcraft" → "DesignCraft".

**Step 4 — One type and shape scale, and a dense Home.**
`LABEL_SIZE` for body, `HEADING_SIZE` for panel headings. Radius: `Tokens::radius` for controls,
`radius_pill` only for mode pills. Delete the 9.0/9.5/10.0/10.5/12.5 sizes; map 10.5 → 11.0, 12.5 → 12.0
or 13.0. Because the Home screen is a **dense tool surface** (§7.1), the 24 px heading / 32 px radius-16
pills / 44 px hero block are replaced by a compact start page: 11.5 px section headers, a toolbar row of
`widgets::outline_button`-sized actions, and a plain profile/recent list.
Acceptance: `rg "semibold\((9|10|10\.5|12\.5)"` returns nothing in `workspace.rs`; no `corner_radius(16.0)`
and no `proportional(22.0)`-and-larger glyph left on Home; and a Home screenshot or test still shows all
five actions, three profile cards and the recent list (§7.5 — density is restyling, not deletion).

**Step 4a — Worksheet geometry.**
Measure the wrapped title when sizing a card (F13) and add a test with a three-line title that asserts
`Pg. N` and the edit link stay inside the card. Give the right dock the reference resizer and persist its
width (F14). Replace the boxed settings form with `widgets::section` + `divider` and the margin steppers
with `widgets::spin_measure` (F15).
Acceptance: no clipped metadata row in a 3-line-title fixture; dock width survives a restart; margin row
matches a Properties-panel screenshot.

**Step 5 — Icons.**
Replace emoji prefixes with `icons::paint`; add missing names to `icons.rs`; use the icon for `×` and `«`
instead of text glyphs. Flag glyphs in the language selector stay painted (they already are) but move to a
shared helper so the View menu can use the same asset instead of 🇮🇩/🇬🇧.
Acceptance: no emoji in `workspace.rs` / `pubgen_bar.rs` user-visible strings.

**Step 6 — Tabs.**
Extract the dock tab renderer; use it for host tabs; render host tabs in the 35 px chrome tab band the
reference uses; drop the 2 px accent top line; adopt the reference collapse bar (18 px, full height,
labelled). Fold the centre segmented control (Page Structure / Visual Live Preview / Execution Log) onto
the same component so the screen has one tab treatment instead of three. Then honour the two-step role
split (§7.6): tab 1 = publication information from the `.pubgen`, tab 2 = generation into DesignCraft
pages; tab 1 opens by default (already the case); **keep the existing labels**; move generation status
("● Clean (QA Fit 100%)", Generate buttons) out of tab 1 and keep it in tab 2 unless Step 8 puts it in the
status bar.
Acceptance: one tab-rendering function; `rg "tab_bar_height|ORCH_TAB_MAX_W"` collapses to one constant;
opening the sample `.pubgen` lands on the information tab; generation controls appear only in tab 2.

**Step 7 — Accessibility, menus, mode pills.**
`widget_info` + `Role` on mode pills, tabs, dock chevrons, steppers, cards, pills. Convert menu rows to
`Item::Cmd`; running-job dot becomes item status, checkmark from `menus::state::checked`. Mode pills become
first-class chrome (§7.2): one component in `chrome/`, tokenised active colours (kill `pubgen_bar.rs:86-88`
and `:97-99` literals), same metrics in both modes, label "DesignCraft".
Acceptance: accessibility audit P1-3/P1-5/P1-6 closed; `ui.click` by widget name works on both modes;
`rg "Color32::from_rgb" crates/ui-egui/src/menus/pubgen_bar.rs` returns 0.

**Step 8 — Full EN, and Orchestrator status semantics.**
EN must ship complete (§7.3): every user-visible string gets an `en` key — the F8 list first (QA log, card
creation, studio panel, cover form, guard dialog, map header), then the menu rows that are English literals
today. Add a test that walks the i18n table and fails on any key missing an `en` value, so "mixed-language
frame" cannot regress. Decide the Orchestrator status bar: hide it, or populate it with workspace/profile/
job facts (the `● Clean (QA Fit 100%)` fact currently floats in the centre column).
Acceptance: no ID literal reachable while `lang == "en"`; status-bar decision implemented and screenshotted.

## 7. Decisions (product owner, 2026-11-07)

These were open questions; they are now settled and the plan in §6 assumes them.

1. **Dense tool surface, not a landing page.** Orchestrator must read like DesignCraft's panels: dense,
   small type, one accent. Consequence: the Home screen's 24 px heading, 32 px/radius-16 pills and 44 px
   hero must be reshaped into a compact start page (toolbar-sized actions, list rows), and "Visual Live
   Preview" styling follows the same rule. §4 F5 and Step 4 apply to Home too — there is no exempt
   "marketing" zone.
2. **Mode pills are first-class chrome.** They stay in the app bar in both modes, always visible, and must
   be tokenised (today their active colours are literals — `pubgen_bar.rs:86-88`, `:97-99`). They are the
   one sanctioned bespoke chrome element; the View-menu duplicates can stay as a fallback.
3. **Indonesian is the primary language, English must ship fully.** EN is not a partial switch: every
   user-visible string needs an `en` key before the EN selector is offered. F8's list is therefore a
   release blocker, not a nice-to-have, and the "remove the EN switch" alternative is off the table.
4. **Two columns is the design.** Centre card area + right panel dock; the documents already describe this
   (`README-PUBGEN.md:25-26`, `:88-92`, `:164` — right dock with tabs "Sumber & Konten" / "Profil &
   Eksekusi"). The "3-column" wording came from the `350930b8e` commit message and plan notes, not from
   the shipped UI. No three-pane work; the review's Q4 concern is closed.
5. **Home keeps every object it has today.** All five actions ("Open file (.pubgen)", "Import Manifest
   (XLSX)", "Sample BRS", "Sample DDA", "Open Active Studio"), the three profile cards, the recent-documents
   list and the hero icon/title stay. Styling and semantics only — no feature, shortcut or affordance is
   dropped in the name of density. Dense means smaller type, token radii and toolbar metrics, not fewer
   things.
6. **Right dock tab roles are fixed, and they are a two-step flow.** Tab 1 is *information*: the state read
   out of the `.pubgen` package (sources, embedded content, metadata/document settings, content tree), i.e.
   what the publication *is*. Tab 2 is *processing*: profile + generation controls that turn the Worksheet
   data into DesignCraft pages (`generate_from_workspace`, `compileProject`, QA/preflight status), i.e. what
   to *do* with it. Consequences: tab 1 is the default when a workspace opens (already true —
   `right_dock_tab: OrchestratorRightTab::SumberKonten`, `workspace.rs:518`); **labels stay as they are**
   ("Sources & Content" / "Profile & Execution", i.e. "Sumber & Konten" / "Profil & Eksekusi"); generation
   controls and status must not leak into tab 1 (green QA status, Generate buttons) and file facts must not
   be repeated inside tab 2.

Still genuinely open (not decided above): whether the `● Clean (QA Fit 100%)` fact lives in tab 2 or in the
status bar (§6 Step 8 — the status-bar decision is still pending).

## 8. Implementation log

**2026-11-07 — F1 (legibility) fixed; part of F2.** Branch `studio`, uncommitted.

- `theme.rs`: added two tokens, `success` and `danger` — *text* colours, not badge fills, chosen to reach
  ≥ 4.5:1 on `panel` in all five brightnesses (Dark `#6ee7a8`/`#fca5a5`, Medium Dark `#86efac`/`#fecaca`,
  Medium Light `#063f1e`/`#6b1a10`, Light `#0b6b32`/`#b3261e`, High Contrast `#4ade80`/`#f87171`).
- `workspace.rs`: the three `painter.galley(..., Color32::WHITE)` calls that overrode an already
  token-coloured galley (`host tab` ~`:1157`, `tab_button` ~`:2343`, `tag pill` ~`:2779`) now pass
  `Color32::PLACEHOLDER`; `measured_card_height` lays out with `PLACEHOLDER` (metrics only).
- `workspace.rs`: the eleven Tailwind status/label colours now use `t.success` / `t.danger` (QA status,
  log lines, "● Terintegrasi", slot OK/Over, delete-confirm labels); the green map-section heading is now
  `t.text_strong`. Content thumbnails keep their own palette on purpose (§7.1 decision).
- Test: `theme::readout_tests::status_and_label_text_is_readable_on_every_surface` asserts success/danger
  ≥ 4.5:1 on `panel`, `text_strong` on `well` and `text_dim` on `tab_strip` for every `Brightness::ALL`.
  `cargo test -p designcraft-ui-egui --features pubgen --lib theme::` → 12 passed.
- Also repaired a broken import left by in-flight work: `workspace.rs` called `super::load_card2_csv`
  (private import) → `super::csv::load_card2_csv`; without it the crate did not compile.

**2026-11-07 — F2 sweep done (chrome colours tokenised).** Branch `studio`, uncommitted (F1 above was
swallowed into commit `4707294b` by in-flight DDA work; F2 continues on top).

- `workspace.rs`: every remaining *chrome* `from_rgb` now reads Tokens — hero block → `t.accent_strong`;
  profile-card accents DDA `#3b82f6`→`t.accent`, BRS `#f59e0b`→`t.live_corner`, General `#9ca3af`→`t.text_dim`;
  both `btn_fill = #2563eb` sites (margin stepper, add-page/object) → `t.accent_strong`; cyan links
  (`View Latest PDF`, `Click to edit`) → `t.accent`; Generate `#0e7490` and Publish `#166534` fills →
  `t.accent_strong`; destructive fills (`Yes, Delete`, guard `⚠️ Timpa Tata Letak`) → `t.caption_close` +
  `t.caption_close_text`; `Save & Close` and guard `💾 Simpan Versi Baru` → `t.accent_strong`. The only
  `from_rgb` left in `workspace.rs` are content previews (sample-document mockups `:1318-1357`,
  `paint_card_thumbnail` `:2357-2458`, visual live preview `:2812-2861`) — sanctioned by §7.1.
- `pubgen_bar.rs`: mode pills tokenised — `mode_tab_button` now `(ui, label, active, tip)`; active pill =
  `t.row_selected` fill + 1 px `t.accent` rim + `t.text_strong` (one treatment for both modes; ponytail: add
  a token per mode if they ever need distinct identities); inactive `t.text_dim`/hover `t.hover` unchanged;
  added `widget_info(Role::Button)` (screen-reader name); label `"Designcraft"` → `"DesignCraft"` (F10).
- Test fix: `pubgen_bar_keeps_the_host_menus_and_adds_the_pubgen_ones` now matches menu names by *last*
  occurrence — the renamed pill shares its text with the later "DesignCraft" menu and pills render first.
- Verified: `cargo test -p designcraft-ui-egui --features pubgen --lib` → **1338 passed, 0 failed**;
  `cargo fmt` + `cargo clippy -p designcraft-ui-egui --features pubgen --all-targets` clean.
  Screenshot `.ui-shots/worksheet-after-f2.png` (MediumDark, sample BRS package) confirms no layout
  regression: pills/steppers/links/CTAs now token blue, status dots `t.success`, dock strip unchanged.
  Light-theme screenshot still owed (no control channel; theme needs a UI click).
- Hygiene note: the ui-egui test run rewrites three committed zip assets
  (`assets/templates/{data_tables,dividers,front_matter}/001_*.designcraft`) nondeterministically — restored
  with `git checkout`; worth a separate fix (tests must not dirty tracked binaries).

Not done yet: F3 (app-bar centre title says "DesignCraft" in Orchestrator), F4 (emoji vs icon font),
F5 (type/radius scale), F6 (three tab treatments), F7 (accessibility widget_info sweep), F8 (i18n EN/ID
completeness), F9 (menu rows bypass `Item::Cmd`), F10 rest, F11 (dead chrome modules), F12 (status-bar
semantics), F14 (fixed 320 px dock), F15 (blank status bar).

**2026-11-07 — F13 + F16 fixed (card footer clipping, `Source::`).** Branch `studio`.

- `workspace.rs` `measured_card_height`: the budget assumed bare text heights (15/16 px rows), but the pill
  and footer rows are `Ui::horizontal` layouts whose minimum is the style's `interact_size.y` (18 px by
  default) — every card was under-measured by ~10-25 px and wrapped titles pushed the footer past the card
  edge. The measurement now takes `row_min = ctx.global_style().spacing.interact_size.y` and budgets
  `PAD_Y + 3 gaps + max(galley, row_min)` for each of the four rows. Regression test
  `measured_card_height_budgets_row_minimums_and_wrapped_titles` asserts the floor (≥ 12 + 3×2 + 4×18) and
  that a wrapped 3-line title grows the budget.
- `Source:: csv/COV.csv` doubled colon: the i18n key itself was `"Source:"` while the format string added
  `": "` again. Added a colon-less key `("Source", "Sumber")` in `i18n/id.rs` and switched the two formatted
  sites to `tr(lang, "Source")`; the standalone caption sites keep the `"Source:"` key.
- Verified: `cargo test -p designcraft-ui-egui --features pubgen --lib` → **1339 passed, 0 failed**; clippy
  clean; screenshot `.ui-shots/worksheet-after-f13.png` shows every card footer ("Pg. N ✎ Click to edit")
  fully inside its card and single-colon `Source:` lines.