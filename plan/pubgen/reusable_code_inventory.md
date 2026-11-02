# Publisher Studio (pubgen) — Reusable Code Inventory & Mapping

Dokumen ini memetakan seluruh modul dan fitur dalam [`plan/pubgen/new_method_plan.md`](./new_method_plan.md) ke fungsi, struct, dan logika yang sudah ada di codebase saat ini (`designcraft-pubgen-0.7.X`). 

Karena Publisher Studio akan dikembangkan sebagai aplikasi desktop standalone di direktori `pubgen/` (tanpa memodifikasi core DesignCraft), kode-kode berikut dapat disalin (porting) atau dijadikan pustaka internal pada workspace `pubgen/`.

---

## 1. Content Ingestion & Normalization (`ingest.rs` & `metadata_form.rs`)

Tujuan: Mengonversi input mentah (DOCX, RTF, XLSX) menjadi format internal yang dinormalisasi (Markdown, CSV, JSON).

### A. Konversi Dokumen Teks (DOCX / RTF → Markdown)
* **Sumber:** `crates/textimport/`
* **File Kunci:**
  * [`crates/textimport/src/docx.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/textimport/src/docx.rs)
  * [`crates/textimport/src/rtf.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/textimport/src/rtf.rs)
  * [`crates/textimport/src/export.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/textimport/src/export.rs)
* **Fungsi / Struct yang di-reuse:**
  * `docx::import(bytes: &[u8]) -> Result<Imported, ImportError>`: Membaca dokumen Word via XML parsing, mengekstrak style paragraf, formatting (bold/italic/underline), bullet/numbering list (`docx_numbering.rs`), dan tabel.
  * `rtf::import(bytes: &[u8]) -> Result<Imported, ImportError>`: Membaca dokumen RTF, mempertahankan atribut teks dan paragraf.
  * `export::plain_text(story: &Story) -> String`: Mengekstrak teks murni dari `Story`.
* **Adaptasi untuk `ingest.rs`:**
  * Tambahkan serializer `export_markdown(story: &Story) -> String` yang memanfaatkan heading level, formatting bold (`**`), italic (`*`), dan daftar (`- `) dari objek `Imported`/`Story`.

### B. Konversi Tabel (XLSX → CSV)
* **Sumber:** `crates/ui-egui/src/panels/pubgen/excel.rs` & `crates/textimport/src/xlsx.rs`
* **File Kunci:**
  * [`crates/ui-egui/src/panels/pubgen/excel.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/excel.rs)
  * [`crates/textimport/src/xlsx.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/textimport/src/xlsx.rs)
* **Fungsi / Struct yang di-reuse:**
  * `parse_xlsx_sheets(path: &str) -> Result<Vec<ExcelSheetData>, String>`: Membaca sheet workbook menggunakan `zip` + `quick-xml` (ringan, tanpa dependensi C/Python).
  * `convert_excel_to_csv_data(sheets: &[ExcelSheetData], base_stem: &str) -> ExcelConversionResult`:
    * Memecah setiap sheet menjadi file tabel CSV individual.
    * Mengonversi header bertingkat / sel merger ke notasi stacked header `--`.
    * Memformat nomor tabel dan sanitasi nama file (`format_table_number`, `sanitize_filename`).
  * `parse_cell_ref(r: &str)` & `parse_range(ref_str: &str)`: Utilitas pemetaan sel Excel (`A1` → `(0, 0)`).
  * `escape_csv_field(val: &str) -> String`: Menjaga integritas data saat diekspor ke CSV.

### C. Utilitas CSV Parsing & Heuristik
* **Sumber:** `crates/ui-egui/src/panels/pubgen/csv.rs`
* **File Kunci:**
  * [`crates/ui-egui/src/panels/pubgen/csv.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/csv.rs)
* **Fungsi yang di-reuse:**
  * `detect_delimiter(text: &str) -> char`: Heuristik frekuensi dan konsistensi baris untuk mendeteksi otomatis pemisah (`,` `;` `\t` `|`).
  * `parse_csv_content_with_delim(content: &str, delim: char) -> Vec<Vec<String>>`: Quote-aware CSV matrix parser.
  * `delimiter_name(delim: char) -> &'static str`: Label nama delimiter untuk UI.

### D. Ekstraksi Metadata Form (`metadata_form.rs` → `metadata.json`)
* **Sumber:** `crates/ui-egui/src/panels/pubgen/excel.rs` & `compile.rs`
* **Fungsi / Struct yang di-reuse:**
  * `struct ExcelCoverData`: Struktur data lengkap metadata publikasi (title, subtitle, year, volume, catalogue, code, agency, pub_office, logo, address, dll.).
  * `parse_cover_sheet(sheet: &ExcelSheetData) -> ExcelCoverData`: Parsing otomatis dari sheet `COVER` jika ada.
  * `extract_field_for_keys(row: &[String], expected_keys: &[&str]) -> Option<String>`: Pencocokan field metadata yang toleran terhadap spasi / variasi nama kolom.

---

## 2. Project File Container — `.pubgen` (`project.rs`)

Tujuan: Manajemen container ZIP `.pubgen` yang menyimpan manifest, metadata, normalized content (`content/`), assets, dan translations.

* **Sumber:** `crates/ui-egui/src/panels/pubgen/excel.rs` & `compile.rs`
* **File Kunci:**
  * [`crates/ui-egui/src/panels/pubgen/compile.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/compile.rs)
  * [`crates/ui-egui/src/panels/pubgen/excel.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/excel.rs)
* **Fungsi / Struct yang di-reuse:**
  * `create_csv_zip(files: &[(&str, &str)]) -> Result<Vec<u8>, String>`: Menggunakan `zip::ZipWriter` dengan opsi kompresi Deflate. Logika ini dapat diperluas untuk menulis struktur direktori `.pubgen`.
  * `struct ProjectManifest`: Schema JSON manifest lengkap (versioning, info publikasi, output, profil).
  * `ProjectManifest::load(path: &Path)`: Deserialisasi manifest berbasis `serde_json`.
  * `validate_lists`, `validate_numbering`: Validasi integritas bab dan penomoran sebelum proyek disimpan/dijalankan.
  * Pola penyimpanan atomik: Tulis ke file temporary (`.pubgen.tmp`), kemudian `std::fs::rename` untuk mencegah file corrupt jika proses terputus.

---

## 3. Slot Binding & Template Discovery (`slots.rs`)

Tujuan: Menginspeksi template `.designcraft` melalui DesignCraft (bukan unzipping langsung) dan menghitung alokasi budget karakter.

* **Sumber:** `crates/ui-egui/src/panels/pubgen/covers.rs` & `crates/ui-egui/src/control.rs`
* **File Kunci:**
  * [`crates/ui-egui/src/panels/pubgen/covers.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/covers.rs)
  * [`crates/ui-egui/src/control.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/control.rs)
* **Fungsi / Logika yang di-reuse:**
  * `walk(v: &Value, id: u64) -> Option<[f64; 4]>` & `scan(...)`: Logika traversal respons pohon JSON `document.inspect` untuk menemukan item berdasarkan ID atau nama berawalan `slot:*`.
  * Perhitungan budget karakter: Membaca dimensi bounding box slot `[x0, y0, x1, y1]` dikombinasikan dengan estimasi ukuran font template untuk menentukan batas aman `max_chars`.

---

## 4. AI Translation & Copyfit Pipeline (`translate.rs` & `glossary.rs`)

Tujuan: Penerjemahan terstruktur berbasis LLM, pembatasan karakter (budgeting), glossary enforcement, dan copyfit retry loop.

### A. Klien LLM Translation (Groq / OpenAI-compatible)
* **Sumber:** `crates/translate/src/online.rs` & `crates/translate/src/lib.rs`
* **File Kunci:**
  * [`crates/translate/src/online.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/translate/src/online.rs)
  * [`crates/translate/src/lib.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/translate/src/lib.rs)
* **Fungsi / Struct yang di-reuse:**
  * `OnlineTranslator::new(key: &str, model: &str) -> Self`
  * `OnlineTranslator::translate_batch_with_glossary(...)`: Mengirim batch teks dengan terminologi glossary di system prompt, validasi jumlah respons agar tidak terjadi desinkronisasi index.
  * `remaining_requests() -> Option<u32>`: Melacak rate-limit sisa harian dari header respons API.
  * Penanganan HTTP 429 backoff otomatis (`MAX_429_RETRIES = 3`, parsing header `retry-after`).

### B. Glossary Terminology Engine (`glossary.rs`)
* **Sumber:** `crates/ui-egui/src/panels/pubgen/ai.rs` & `vocab.rs`
* **File Kunci:**
  * [`crates/ui-egui/src/panels/pubgen/ai.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/ai.rs)
* **Fungsi / Struct yang di-reuse:**
  * `struct GlossaryTerm` & `struct BpsGlossary`: Model penyimpanan pasangan kata (Indonesian / English / Official / AI draft).
  * `BpsGlossary::load(user_path: &str) -> Self` & `BpsGlossary::save(&self, path: &Path) -> Result<(), String>`.
  * `terms_from_csv(content: &str) -> Vec<(String, String)>`: Parsing glossary dari file CSV eksternal.
  * `merge_terms(target: &mut Vec<GlossaryTerm>, imported: Vec<(String, String)>) -> (usize, usize)`: Penggabungan terminologi tanpa duplikasi.
  * `BpsGlossary::lookup(&self, cell: &str) -> Option<&str>`: Pencocokan cepat istilah saat pra/pasca-terjemahan.
  * `translatable_cell(cell: &str, glossary: &BpsGlossary) -> bool`: Filter penentu apakah suatu sel/string perlu diterjemahkan atau merupakan angka/simbol murni.
  * `alias_en<'a>(cell: &str, glossary: &'a BpsGlossary) -> Option<&'a str>`: Lookup terjemahan baku.

### C. Ekstraksi Teks & Background Job Worker
* **Sumber:** `crates/ui-egui/src/panels/pubgen/ai.rs`
* **Fungsi / Struct yang di-reuse:**
  * `collect_translation_units(st: &PubgenState) -> Vec<(String, String)>`: Mengumpulkan unit teks dari narasi, tabel, dan metadata.
  * `table_cells(st: &PubgenState, file_val: &str) -> Vec<String>`: Ekstraksi sel-sel teks dari tabel CSV.
  * `rtf_paragraphs(st: &PubgenState, file_val: &str) -> Vec<String>`: Ekstraksi paragraf naratif.
  * `AiJob::spawn(...)`, `AiJob::poll(...)`, `AiJob::cancel()`: Pola threading background worker non-blocking untuk UI egui.

---

## 5. DesignCraft Connector (`process.rs` & `control_client.rs`)

Tujuan: Menjalankan DesignCraft headless (`designcraft-cli`) atau live via TCP control protocol (`127.0.0.1:7979`).

* **Sumber:** `apps/designcraft/src/control_server.rs` & `crates/ui-egui/src/control.rs`
* **File Kunci:**
  * [`apps/designcraft/src/control_server.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/apps/designcraft/src/control_server.rs)
  * [`crates/ui-egui/src/control.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/control.rs)
* **Logika / Protokol yang di-reuse:**
  * Protokol JSON Lines:
    * Request: `{"id": u64, "method": String, "params": Value}\n`
    * Response: `{"id": u64, "result": Value, "error": Option<String>}\n`
  * Metode DesignCraft yang ditargetkan:
    * `engine.execute`: Menjalankan perintah engine (`paste.svg`, `swatch.create`, `preflight.pubgen`).
    * `document.inspect`: Memeriksa pohon halaman, item, dan status overset frame.
    * `ui.render`: Mengekspor thumbnail halaman menjadi PNG jika diperlukan.
    * `app.export`: Mengekspor hasil dokumen akhir ke PDF/X-4 atau PDF standar.

---

## 6. Feature Backlog (F1 s/d F5)

### F1 — Vector Charts & F2 — Choropleth Maps (`vector.rs`)
* **Sumber:** `crates/ui-egui/src/panels/pubgen/charts/`
* **File Kunci:**
  * [`crates/ui-egui/src/panels/pubgen/charts/spec.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/charts/spec.rs)
  * [`crates/ui-egui/src/panels/pubgen/charts/layout.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/charts/layout.rs)
  * [`crates/ui-egui/src/panels/pubgen/charts/palette.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/charts/palette.rs)
  * [`crates/ui-egui/src/panels/pubgen/charts/geojson.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/charts/geojson.rs)
  * [`crates/ui-egui/src/panels/pubgen/charts/stock_maps.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/charts/stock_maps.rs)
* **Fungsi / Struct yang di-reuse:**
  * `parse_chart_spec(sheet: &ExcelSheetData) -> Option<ChartSpec>`
  * `extract_chart_data(sheet: &ExcelSheetData, spec: &ChartSpec) -> (Vec<String>, Vec<Vec<f64>>)`
  * `compute_layout(spec, categories, series_values) -> Result<ChartLayout, String>`:
    * Menghasilkan struktur geometri murni: `BarRect`, `DonutSlice`, `GridLine`, `LegendItem`, `XTickLabel`.
  * `BPS_CHART_PALETTE`, `series_color(series_idx: usize) -> &'static str`: Skema warna grafik statistik standar.
  * `GeoMap`, `GeoFeature`, `ProjectedFeature`, `load_geo_map(...)`, `list_stock_map_charts()`.
* **Catatan Adaptasi (Emitter → SVG):**
  * Di implementasi lama, `charts/emitter.rs` langsung memanggil engine command internal DesignCraft (`frame.create`).
  * Di implementasi baru, data `ChartLayout` dan polygon `GeoMap` langsung ditulis menjadi format teks **SVG standar** (`<svg ...><rect .../><path .../></svg>`), kemudian diinjeksikan ke DesignCraft menggunakan perintah `paste.svg` / `ui.paste`.

---

### F3 — Cover Options & Generative AI Art (`covers.rs`)
* **Sumber:** `crates/ui-egui/src/panels/pubgen/covers.rs`, `ai_cover.rs`, dan `crates/imagegen/`
* **File Kunci:**
  * [`crates/ui-egui/src/panels/pubgen/covers.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/covers.rs)
  * [`crates/ui-egui/src/panels/pubgen/ai_cover.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/ai_cover.rs)
  * [`crates/imagegen/src/cloudflare.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/imagegen/src/cloudflare.rs)
  * [`crates/imagegen/src/lib.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/imagegen/src/lib.rs)
* **Fungsi / Struct yang di-reuse:**
  * `HORIZON_STYLE`, `BUSINESS_STYLE`, `URBAN_STYLE`, `TRICOLOR_STYLE`, `PRISM_STYLE`, `QUARTER_STYLE`: Preset index dan style cover statistik.
  * `apply_report_palette(...)` & `apply_urban_palette(...)`: Mapping warna tema sampul.
  * `AI_PRESETS`, `AiPalette`, `AI_PALETTES`: Preset prompt generatif berbasis tema publikasi.
  * `cloudflare::generate_image(...)`: Klien Cloudflare Workers AI (`flux-1-schnell`) untuk menghasilkan latar belakang cover artistik.
  * `Aspect::dimensions()`: Perhitungan rasio dimensi aman (ISO A4, Folio/F4, Letter, Wraparound spread).
  * `quota.rs`: Pelacakan konsumsi neuron kuota harian.

---

### F4 — Glossary Enforcement
* Seluruh implementasi dapat digunakan langsung dari `crates/ui-egui/src/panels/pubgen/ai.rs` (lihat bagian 4.B di atas).

---

### F5 — Table of Contents Layouts (`toc.rs`)
* **Sumber:** `crates/ui-egui/src/panels/pubgen/toc.rs`
* **File Kunci:**
  * [`crates/ui-egui/src/panels/pubgen/toc.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/toc.rs)
* **Fungsi / Konstanta yang di-reuse:**
  * `TOC_KINDS`: Tipe TOC ("TOC Entry", "TOC Table", "TOC Chart", "TOC Paragraph").
  * `toc_kind_words(kind)`, `toc_kind_title(kind, bilingual)`.
  * `strip_tablepage_title_prefix(title, num, typ) -> &str`.
  * Logika two-pass numbering: Mengidentifikasi judul dan pemetaan nomor halaman aktual via traversal inspect hasil pass 1.

---

### Table Formatting & Preflight QA
* **Tabel Formatting:**
  * [`crates/ui-egui/src/panels/pubgen/tables.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/tables.rs): `table_cell_text(cell: &str) -> String` mengonversi baris ganti dalam sel Excel menjadi Unicode Forced Line Break (`\u{2028}`), mencegah pemecahan baris tabel CSV yang tidak diinginkan.
* **Preflight QA Engine:**
  * Perintah engine `preflight.pubgen` sudah tertanam di [`crates/engine/src/cmd/preflight_pubgen.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/engine/src/cmd/preflight_pubgen.rs).
  * Publisher Studio tinggal mengeksekusinya via control client:
    ```json
    {
      "method": "engine.execute",
      "params": {
        "command": "preflight.pubgen",
        "params": { "profile": "print" }
      }
    }
    ```
  * Model penanganan isu preflight di sisi UI dapat mengadopsi [`crates/ui-egui/src/panels/pubgen/audit.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/audit.rs) (`struct AuditIssue`, `enum AuditProfile`).

---

## Ringkasan Matriks Porting

| Modul Baru di `pubgen/` | Status Kode Eksisting | Tindakan |
|---|---|---|
| `ingest.rs` | Siap pakai (di `textimport` & `excel.rs`) | Ekstraksi ke fungsi murni, tambah output Markdown. |
| `metadata_form.rs` | Siap pakai (di `excel.rs` & `state.rs`) | Hubungkan model `ExcelCoverData` ke input widget egui. |
| `project.rs` | Siap pakai (di `compile.rs` & `excel.rs`) | Adopsi struct manifest & pembungkus ZIP atomic rename. |
| `slots.rs` | Siap pakai (di `covers.rs`) | Query slot frame via TCP `document.inspect`. |
| `translate.rs` | Siap pakai (di `translate/online.rs` & `ai.rs`) | Salin modul `OnlineTranslator` & loop copyfit. |
| `glossary.rs` | Siap pakai (di `ai.rs`) | Salin `BpsGlossary` & fungsi validasi/merge kata. |
| `control_client.rs` | Belum ada client TCP terpisah | Buat inverse TCP client dari `control_server.rs`. |
| `process.rs` | Baru | Spawn child process `designcraft` / `designcraft-cli`. |
| `vector.rs` | Siap pakai (di `charts/`) | Ganti emitter engine dengan renderer teks SVG. |
| `covers.rs` | Siap pakai (di `covers.rs`, `ai_cover.rs`, `imagegen`) | Salin preset style cover dan integrasi AI art. |
| `toc.rs` | Siap pakai (di `toc.rs`) | Salin logika filtering judul dan algoritma 2-pass TOC. |

