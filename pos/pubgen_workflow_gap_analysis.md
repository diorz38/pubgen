# Analisis Pembanding: Alur Generating POS (standalone) vs DesignCraft Pubgen

Tanggal riset: 2026-11-04 · Branch: `pubgen-0.8.X`
Objek: `standalone/` (Publication Orchestrator Studio / POS, iced) dibandingkan dengan panel pubgen aplikasi induk `crates/ui-egui/src/panels/pubgen/` (egui) + `crates/engine/`.

Semua baris kode di bawah sudah diverifikasi manual (bukan hanya kesimpulan agen riset).

---

## A. Perbandingan workflow generating

### A.1 Aplikasi induk (DesignCraft) — referensi

Dua jalur, keduanya memanggil engine **langsung** lewat `app.run(command_id, json)`:

| Jalur | Lokasi | Cara kerja |
|---|---|---|
| Panel interaktif | `crates/ui-egui/src/panels/pubgen/widgets.rs` | `card1_body_ui:537`, `card2_body_ui:658` (tabel), `card_cover_body_ui:812` (sampul), `card_toc_ui:1424`, `card_chart_body_ui:1584` |
| Kompilator manifest | `crates/ui-egui/src/panels/pubgen/compile.rs` | `ProjectManifest:49` → `CompilePlan` → `CompileJob` (frame-stepped) → `CompileStep:256`: NewDocument, MasterSetup, FrontCover, InsideCover, TocPlaceholders, BackCover, TocFinalize, FrontMatter/Chapter. Dispatch di `CompileJob::run_one:~758` |

Perintah engine terverifikasi: `toc.generate` / `toc.update` / `toc.entries` (`crates/engine/src/cmd/toc.rs:24/33/53`, param `autoflow` default true), `autoflow` (`cmd/place_text.rs:283`), `text.autoflow` & `text.thread` (`cmd/thread.rs:44/114`), `frame.create`, `file.place`, `object.fit`, `object.fittingOptions`, `type.char`, `type.para`, `style.paragraph.create/edit/apply`, `preflight.pubgen` (`cmd/preflight_pubgen.rs:14`).

### A.2 POS (standalone) — orkestrator

POS **tidak** memanggil engine langsung; ia mengompilasi daftar kartu menjadi skrip JSONL lalu mengeksekusinya:

- `actions.rs::build_batch_script:688-1050` — urutan: `variables.define` → `layout.headerFooter.apply` (preset `hf-brs` / `hf-dda`) → loop `metadata.pages` → regenerasi TOC (2-pass) → isi slot.
- Per `PageKind` (`actions.rs:800-991`): Cover → `template.apply`/`template.page.apply` + `object.place` + `story.setText`; Table → `table.insert`; Chart → `paste.svg`; Paragraph → `story.setText`; Image → `frame.create {content:"image", area}`; Toc → `layout.pages.insert {count: span}` + `toc.generate`.
- Eksekusi `actions.rs::batch_fill:1052-1126` → `designcraft-cli script fill.jsonl --in template.designcraft --save out.designcraft --export out.pdf`.
- Jalur live: `control_client.rs` TCP 7979 (`document.inspect`, `engine.commands`), `slots.rs` untuk discovery/isi slot.
- Paginasi direplikasi di POS: `toc.rs::simulate_pagination:236-263`, `toc_span_pages:230`, `estimate_toc_pages:270` (26 entri/halaman Standard, 22 TablePage); sisa overflow diserahkan ke `autoflow`.

### A.3 Kesimpulan struktur alur

**Kerangka alur SUDAH SAMA** (profil → metadata → normalisasi konten → daftar kartu → skrip perintah → TOC dua-pass → preflight `preflight.pubgen`). Yang **berbeda**: aplikasi induk membangun plan lalu mengeksekusi lewat engine di proses yang sama, POS membangun teks skrip JSONL dan mendelegasikannya ke `designcraft-cli`. Perbedaan ini sah sesuai desain clean-room POS, tapi jadi sumber bug kontrak perintah (lihat bagian C).

---

## B. Tujuh poin acuan — status kesesuaian

Ringkasan: **0 dari 7 terpenuhi penuh**. Yang paling mendekati: TOC (poin 6) dan Chart (poin 2).

### B.1 Card type Cover — menu publication cover ⚠️ belum sesuai

| | POS | Aplikasi induk (referensi) |
|---|---|---|
| Bentuk form | 14 input teks polos (`CoverField`, `metadata_form.rs:94-110`) di `ui/metadata.rs:1015-1282` | `card_cover_body_ui` (`widgets.rs:812-1391`) — form lengkap |
| Pilihan gaya | Combo `CoverStyle::ALL` isi **4** preset SVG (Horizon/Modern/Tricolor/Minimal, `covers.rs:5-30`) | Combo **13 gaya prosedural** + preset AI (`compile.rs::COVER_STYLES:1376`) |
| Tema/warna | Preset di dialog (`dialogs/cover.rs:58-96`) tapi hanya 4 dirender (`break` idx==4) | Combo tema 8 (`COVER_THEMES:1393`) + hex Primary/Accent/Dark |
| Cover dalam/belakang | Checkbox di dialog Cover Studio saja | Checkbox "Add Inside cover and Back Cover" di kartu |
| Peta wilayah | Radio Watermark/Accent Hero/Outlines (dialog) | "Add Map to Cover" stock + Custom GeoJSON + mode yang sama |
| Slot gambar | Tidak ada pemilih gambar per gaya | Dynamic cover images per gaya, Logo + Logo Text, Back-Cover Event Theme + Office Address, "Replace existing cover" |
| ISSN/ISBN, penerbit | Hanya lewat field teks `ExcelCoverData` | Field Code (ISSN/ISBN) eksplisit |

Gap konkret: form Cover di kartu POS tidak punya pemilih gaya, tema, warna, gambar per-gaya, inside/back cover, atau peta — semuanya tersembunyi di dialog Cover Studio yang datanya **tidak mengalir** ke `PageEntry.cover`. Perlu penyatuan sumber data + peniripan `card_cover_body_ui`.

### B.2 Card type Chart — menu chart options ⚠️ sebagian

POS punya dialog chart (`ui/dialogs/chart.rs:16-248`) tetapi form kartu hanya satu combo `ChartType` (`ui/metadata.rs:1173-1187`).

| Aspek | POS | Aplikasi induk |
|---|---|---|
| Jumlah tipe grafik | **5**: BarClustered, HorizontalBar, Line, Donut, Map (`vector.rs:47-90`) | **9**: + BarStacked, Area, PopulationPyramid, Gauge (`charts/spec.rs:40`) |
| Penempatan | Tidak ada | `ChartPlacement {SelectedPage, AfterTable, CurrentPage}` + `insert_on_new_page` |
| Format angka | Tidak ada | `ValueFormat {Number, Percent, CurrencyIdr}` |
| Detail | Series 1/2/3, label nilai, legenda, gridir, palet peta, CSV editor sudah ada | + PopulationPyramid Male/Female, Donut ring kedua, `csv_grid_preview` |
| Alur data | Dialog memakai `DialogState.toc_*`/chart state sendiri — hasil tidak tersimpan per kartu | `apply_chart` → `emitter::emit_chart` langsung ke dokumen |

### B.3 Card type Gambar — form option image ❌ belum ada

POS (`ui/metadata.rs:1188-1202`) hanya combo `ImageArea` = **FullPage / MarginArea** (`metadata_form.rs:206-211`) + sumber file. Tidak ada fitting, scaling, frame, border, crop.

Aplikasi induk menerapkan: `frame.create` → `file.place {path, frame}` → `object.fit {fitProportionally | centerContent | fitFrameToContent | fitContentToFrame}` → `object.fittingOptions {fitting}` → `object.stroke {swatch:"[None]"}` (`page_utils.rs:260-304`, `pages.rs:254-260`) + `object.textFrameOptions {autoSize:"heightOnly", autoSizeReference:"topLeft"}` (`pages.rs:251`). Semua kosakata ini harus muncul sebagai form di kartu Gambar POS.

### B.4 Card type Table — form options table style ❌ belum ada

Di `card_form` (`ui/metadata.rs:1111-1279`), Table jatuh ke lengan `_ => {}` — **nol field gaya**. Styling dipaksakan dari profil (`profiles.rs::ProfileTableStyling:236-256`) saat generate (`actions.rs:910-924`: `first_row_header`, `col_numbers`, `table_footer`).

Referensi induk, `card2_body_ui` (`widgets.rs:658-809`):
- Checkbox: "First row is header", "Add column numbers (1),(2)…", "Table footer (last row)", "Condition rules (Jumlah, Tahun…)".
- Combo Style: `["Basic Grid","Corporate Navy","Minimalist","Horizontal Rules","BRS6"]` (`tables.rs:414`).
- Combo Theme: `TABLE_COLOR_THEMES` (`themes.rs:49-55`) — `#FFCB56-#FFE9A9`, `#F3D2AF-#F8EADC`, `#9E7EB5-#C4A4DA`, None, BRS6 `#F58838-#FBBF91`. Resolver `parse_table_color_theme:58`, struct `TableColorTheme {header_footer, row_number, header_text_color, data_odd_fill, data_even_fill, font_size_delta}`.
- Radio Place: "At cursor" / "New frame" / "At page".
- Implementasi border/banding/font di `tables.rs`: `border_weight` 2.0 (style 0) else 0.5; `border_overrides [true;4]`; Horizontal Rules = tanpa border vertikal, header top 1.0 / bottom 0.5, bottom tabel 1.0; BRS6 8.0pt; merged header max 3 stack (pisah di `"--"`).

### B.5 Card type Paragraph — form options paragraph style ❌ belum ada

Sama seperti Table, Paragraph jatuh ke `_ => {}`. Satu-satunya toggle adalah `has_title` (`metadata_form.rs:384-413`). Tipografi hanya dari `ProfileTypography` (`profiles.rs:258-273`) + named style saat generate.

Catatan penting: **di aplikasi induk pun tidak ada editor gaya per-kartu Paragraph** — kosakata atributnya ada di sisi engine dan dipakai programatik. Jadi acuan yang benar untuk POS adalah meniru kosakata ini sebagai form (belum ada di mana pun sebagai UI kartu):
- `type.char {fontFamily, fontStyle, size, leading:{kind:"points",value}, tracking, stroke, strokeBehindFill}` (contoh pemakaian `panels/pubgen/covers.rs:2408-2462`, `pages.rs:640` body Lato 12pt/leading 15pt).
- `type.para {attrs:{align}}` → left/right/center (`charts/layout.rs:104`).
- `style.paragraph.create/edit {chars{fontFamily,fontStyle,size,fill}, para{listType,listName,listLevel,numberExpression,restartNumbers,restartAfterLevel,spaceBefore,spaceAfter}}` (`pages.rs:551-624`, contoh Heading 1 13pt bold 12/4 bernomor `^#.^t`, Heading 2 11pt 8/3 `^1.^#.^t`, Title 14.5pt 0/6).
- Indent: `left_indent` / `first_line_indent` (`helpers.rs:344-349`; tes `page_utils.rs:609-619`: numbered 36/−18, nested bullet 72/−18).

### B.6 Scope option — harus punya opsi TOC + form TOC ❌ belum sesuai

Ini **gap paling eksplisit terhadap permintaan Anda**. `PageItemScope` POS hanya dua varian (`metadata_form.rs:132-137`):

```rust
pub enum PageItemScope { #[default] NewPage, NewObject }
```

TOC **bukan** opsi scope — ia varian `PageKind::Toc` (`metadata_form.rs:159-168`). Akibatnya combo "Cakupan/Scope" (`ui/metadata.rs:1026-1036`) dan tombol add-bar (`publication_cards:269-333`) tidak pernah menawarkan TOC.

Sisi baiknya, form TOC yang diminta **sudah ada** dan mendekati referensi:
- `TocData {title, style, layout, hanging_indent, include_self, pair_bilingual}` (`metadata_form.rs:285-341`), dirender inline di kartu (`ui/metadata.rs:1203-1276`).
- `TocStyle {Entry, Table, Chart, Paragraph}` (`metadata_form.rs:235-245`) = "Gaya TOC Entry/Table/Chart/Paragraph" yang Anda minta. ✔
- `TocLayout {Standard, TablePage}` (`metadata_form.rs:263-269`) = "Tata Letak Standard/TablePage". ✔
- Generate dua-pass + `toc.generate` + `autoflow` (`actions.rs:970-990`, `push_toc_generate:673-686`), entri via `toc_entries_json` (`toc.rs:169-201`). ✔
- Dialog TOC terpisah (`ui/dialogs/toc.rs:15-142`) dengan 3 ekstraksi (Daftar Isi/Tabel/Grafik) + preview story text.
- Referensi induk: `card_toc_ui` (`widgets.rs:1424`) — "TOC title" + checkbox "Insert on new page after selected page"; engine `toc.generate` param `rect`, `createBookmarks`, `autoflow`, `replace`.

Yang kurang: (1) `Toc` sebagai varian `PageItemScope`, (2) penyatuan dialog vs form kartu agar satu sumber kebenaran, (3) field `rect`/`createBookmarks` tidak diekspos di POS.

### B.7 Options Cover Template — thumbnail ❌ belum ada

POS **tidak menampilkan thumbnail** sama sekali:
- Combo `CoverStyle::ALL` hanya label teks (`ui/dialogs/cover.rs:20-30`).
- "Preview" yang ada adalah **wireframe skematik** hasil gambar canvas: `PageThumb`/`thumb_frame` (`ui/metadata.rs:502-708`) dan `CoverCanvas` di tab Preview (`ui/preview.rs:208-369`) — bukan render aset.
- Kenyataannya POS sudah punya **14 aset `.designcraft`** di `assets/templates/covers/` (`000_wave_curves` … `012_quarter_stack`, `100_ai_base`, plus `brs/`) yang dipilih saat generate via `cover_template_by_style()` (`actions.rs:600-644`, fallback `007_modern_horizon`) — tetapi combo UI hanya menawarkan 4 nama Horizon/Modern/Tricolor/Minimal, jadi 9+ template tak pernah bisa dipilih dari UI. Ada juga direktori `assets/templates/new_covers/` yang tidak dirujuk.

Referensi induk — sistem thumbnail lengkap di `crates/ui-egui/src/panels/pubgen/thumbnails.rs`:
- `show_style_thumbnail_tooltip:831-872` (ikoner 👁 di `card_cover_body_ui`), PNG 150×≤220, fallback vector `paint_fallback_cover_thumb:219-829` (gaya 0-12 + AI 100-104) plus string deskripsi per gaya.
- Pencarian berprioritas `thumb_dirs():35-59`: env `DESIGNCRAFT_PUBGEN_THUMBS` → `config_dir()/PubgenThumbs` → `plan/pubgen/samples/thumb`, `samples/thumb`, `outputs`.
- Pencocokan nama→gaya: `style_aliases():100-116`, `slug():62`, `style_thumb_match():120-153` (skor: prefix numerik `0_`/`style7` > alias persis > starts-with > contains), `thumb_path_in():165-183` (folder pertama dengan skor terbaik menang, `3_my_cover.png` meng-override sampel).
- Rendering/cache: `style_thumbnail_texture():185-201` key = (style, path, file len, mtime); decode via `designcraft_render::decode_pixmap_page` lalu `fit_texture(ci, 300)`.

Rencana port untuk POS: generate thumbnail PNG per template sekali (`designcraft-cli run --sample`/`ui.render` atau `decode_pixmap_page`), simpan di `assets/templates/covers/thumb/<nnn>_<slug>.png`, lalu tampilkan sebagai `iced` `image::view` dalam grid pilihan template (bukan combo teks), dengan cache key path+len+mtime.

---

## C. Temuan kritis tambahan: kontrak perintah POS vs engine induk

Ini menyebabkan generate **gagal di runtime**, bukan hanya beda UI. diverifikasi langsung terhadap engine saat ini.

| Perintah yang dikirim POS | Status di engine | Bukti |
|---|---|---|
| `paste.svg` (`actions.rs:928`) | ❌ tidak terdaftar. Yang benar: **`edit.pasteSvg`** | `crates/engine/src/cmd/paste_svg.rs:15,277` |
| `object.place` (`actions.rs:866,877,899`) | ❌ tidak ada. Yang benar: **`place.drop`** / `place.load` / `file.place` | `cmd/file/registry.rs:130`, `cmd/file/place.rs:255+` |
| `template.apply` (`actions.rs:809`, jalur BRS) | ❌ tidak ada. Yang benar: **`template.page.apply`**, `template.catalog.list`, `template.preview.get` | `cmd/template/mod.rs:26` |
| `frame.create {content:"image", area:"full_page"\|"margin"}` tanpa `rect` (`actions.rs:959-966`) | ❌ **error pasti**: `rect` wajib (`bad("frame.create","missing rect")`), dan `content` hanya `graphic\|text\|unassigned` — `"image"` tidak valid; `area`/`full_page`/`margin` tidak dibaca engine mana pun (grep: 0 konsumen) | `cmd/object.rs:32-39,996` |
| `story.setText`, `story.replaceRange`, `text.select`, `table.insert`, `toc.generate`, `layout.pages.insert`, `layout.headerFooter.apply`, `variables.define`, `style.paragraph.apply`, `object.rename`, `preflight.pubgen`, `document.inspect` | ✅ ada & benar namanya | `cmd/linked.rs`, `cmd/object.rs`, `cmd/anchored.rs`, `cmd/editorial.rs`, `cmd/toc.rs`, `cmd/index.rs`, `cmd/header_footer.rs`, `cmd/layout/pages.rs`, `cmd/lists.rs`, `cmd/preflight_pubgen.rs` |

Masalah protokol jalur live (`control_client.rs:71-73`): `execute()` mengirim **id perintah sebagai field `method`** JSON-RPC. Server kontrol hanya mendispatch allowlist tetap (`engine.execute`, `ui.menu.invoke`, `command`, `engine.commands`, `document.inspect`, `ui.*`) dan mengembalikan `unknown method` selain itu — `crates/ui-egui/src/control.rs:239-250,460`. Jadi panggilan live `story.setText`, `object.rename`, `preflight.pubgen` dari POS akan ditolak. Yang benar: `{"method":"engine.execute","params":{"command": <id>, "params": {…}}}`. Jalur CLI `designcraft-cli script` **sudah benar** karena membungkus tiap baris sebagai `engine.execute` (`apps/designcraft-cli/src/main.rs:573`).

Catatan tambahan: `check_params` di sisi server menolak param tak dikenal pada jalur kontrol (`control.rs:244`), sedangkan jalur CLI umumnya mengabaikan param tak dikenal (`main.rs:69`) — jadi bug `area`/`content:"image"` mungkin diam di satu jalur dan error di jalur lain.

---

## D. Skoring akhir per acuan

| # | Acuan | Status | Skor |
|---|---|---|---|
| 1 | Cover → menu publication cover | ❌ form kartu minim, data dialog tak terhubung | 25% |
| 2 | Chart → menu chart options | ⚠️ dialog ada, 5/9 tipe, tak tersimpan per kartu | 50% |
| 3 | Gambar → form option image | ❌ hanya area FullPage/Margin; perintah generate rusak | 10% |
| 4 | Table → form options table style | ❌ tidak ada sama sekali (profil-only) | 5% |
| 5 | Paragraph → form options paragraph style | ❌ tidak ada (induk juga tak punya UI kartu, tapi kosakata engine siap) | 5% |
| 6 | Scope → opsi TOC + form TOC | ⚠️ form TOC lengkap, tapi TOC bukan opsi scope | 60% |
| 7 | Cover Template → thumbnail | ❌ tidak ada thumbnail; 9/14 template tak terjangkau UI | 10% |

---

## E. Rencana perbaikan (urut prioritas)

**P0 — perbaiki kontrak perintah dulu (generate saat ini rusak)**
1. Ganti id: `paste.svg`→`edit.pasteSvg`, `object.place`→`place.drop`/`file.place`, `template.apply`→`template.page.apply`.
2. Perbaiki kartu Image: `frame.create` dengan `rect` nyata (hitung dari margin profil + `ImageArea`) lalu `file.place` + `object.fit`/`object.fittingOptions`.
3. Perbaiki `ControlClient::execute` agar membungkus `engine.execute {command, params}`.
4. Tambahkan tes "command registry contract": assert tiap id yang dikirim `build_batch_script` ada di `engine.commands`.

**P1 — acuan 6 (scope TOC)**: tambahkan `PageItemScope::Toc`, sinkronkan dialog dan form kartu ke satu `TocData`, ekspos `rect`/`createBookmarks`.

**P2 — acuan 7 (thumbnail)**: pipeline thumbnail PNG + grid pilihan template dengan 13+1 gaya (bukan combo teks 4 item), cache (path,len,mtime).

**P3 — acuan 1 & 2**: satukan state dialog→`PageEntry`, tiru field `card_cover_body_ui` dan `charts/dialog.rs` (tambah BarStacked, Area, PopulationPyramid, Gauge, `ValueFormat`, `ChartPlacement`).

**P4 — acuan 3, 4, 5**: form Image (fit/scale/frame/stroke), form Table (Style 5 opsi + Theme 5 opsi + checkbox + Place), form Paragraph (`type.char`/`type.para`/`style.paragraph.*` + indent). Untuk Table/Paragraph, petakan hasil form ke param `table.insert` / `style.paragraph.create` alih-alih hardcode profil.

Setiap perubahan diikuti: `cargo test` di `standalone/`, lalu smoke integrasi `cargo run -- --smoke` dengan DesignCraft hidup di 7979, dan verifikasi hasil dengan `ui.screenshot` (sesuai aturan "look at the result" di `../AGENTS.md`).

---

## F. Referensi file

POS: `standalone/app/src/{metadata_form.rs, actions.rs, toc.rs, covers.rs, vector.rs, profiles.rs, control_client.rs, slots.rs, audit.rs}`, `standalone/app/src/ui/{metadata.rs, preview.rs, update.rs, panels/center.rs}`, `standalone/app/src/ui/dialogs/{cover.rs, chart.rs, toc.rs}`.
Induk: `crates/ui-egui/src/panels/pubgen/{widgets.rs, tables.rs, themes.rs, thumbnails.rs, compile.rs, pages.rs, page_utils.rs, covers.rs, toc.rs, charts/{dialog.rs, spec.rs}}`, `crates/ui-egui/src/control.rs`, `crates/engine/src/cmd/{toc.rs, object.rs, paste_svg.rs, preflight_pubgen.rs, template/mod.rs, file/{place.rs, registry.rs}}`, `apps/designcraft-cli/src/main.rs`.
