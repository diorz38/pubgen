# Publication Orchestrator Studio (POS) — System Architecture & Technical Plan

> **Current Status (2026-11-02):** 
> - **Phases 1a, 1b, 1c, Phase 2, Phase 3, and Phase 4 (F1-F5) are implemented and verified** (46/46 unit tests pass).
> - **Native Features:**
>   1. **Bilingual GUI (English & Indonesian)** — Indonesian as default, with an instant language switcher in the Top Bar (`i18n.rs`).
>   2. **Adaptive Theme Support (Dark / Light / System Mode)** — System Mode as default with instant runtime switching (`main_ui.rs`, `panels.rs`).
>   3. **Multi-Publication Profiles (BPS DDA & BPS BRS)** — parameterizes geometry, numbering, and structure (`profiles.rs`).
>   4. **Flexible Content Scope ("Halaman Baru" / "NewPage" vs "Objek Baru" / "NewObject")** — allows fine-grained page break vs flowing vertical insertion per content entry (`metadata_form.rs`).
>   5. **Smart Excel Sheet Extraction Rules** — automatically parses BRS sheets (`COV`, `POINT`, `U1` -> NewPage; other `UX`, `CX`, `TX`, `GX` -> NewObject) vs DDA sheets (all -> NewPage) (`ingest.rs`).
>   6. **Hybrid Metadata Ingestion** — auto-fill from Excel `COVER` sheet + manual in-app overrides (`metadata_form.rs`).
>   7. **Publication Cover Studio & AI Art Brief** — SVG cover generator (Horizon, Modern, Tricolor, Minimal) + Cloudflare/SD prompt synthesis (`covers.rs`).
>   8. **Table of Contents (ToC) Engine** — Two-stage extraction with bilingual tab-leader dot formatting (`toc.rs`).
>   9. **Vector Data Visualization** — Native standalone SVG generator for Charts (Bar, Line, Donut) and Choropleth GeoJSON Maps (`vector.rs`).
>   10. **Profile-Driven Header/Footer Styles** — DDA bilingual footer & BRS running header (`header_footer.rs`).

---

## Positioning & Workflow

Publication Orchestrator Studio is a **standalone Rust desktop app** that lives entirely in `pubgen/` (or `standalone/`, its own cargo workspace). The DesignCraft repo is **reference only and is never modified**: no fork, no upstream changes required, no workspace coupling.

The boundary between the two apps is DesignCraft's **documented external interfaces**:

* **JSON control protocol** over TCP `127.0.0.1:7979` (`docs/control-protocol.md` in the DesignCraft repo) — live/interactive mode. Key methods used: `engine.execute` (slot fills, `paste.svg`, `preflight.pubgen`), `document.inspect` (pages, items, oversets), `ui.render` (thumbnails), `ui.paste` (SVG markup → editable vector paths).
* **`designcraft-cli` headless** — batch mode (`designcraft-cli run --cmd '...' --export ...` or `designcraft-cli script ...`).
* **`designcraft-cli mcp`** — optional alternative channel, same command surface.

### End-to-End User Workflow

```
┌────────────────────────────────────────────────────────────────────────┐
│                        USER INPUT WORKFLOW                             │
│                                                                        │
│   0. UI Language Selection:                                            │
│      [ Bahasa Indonesia (Default)  |  English ]                        │
│                                                                        │
│   1. Select / Auto-detect Publication Profile:                         │
│      • BPS DDA (Daerah Dalam Angka)                                    │
│      • BPS BRS (Berita Resmi Statistik)                                │
│      • Custom / Corporate                                              │
│                                                                        │
│   2. Hybrid Metadata Ingestion:                                        │
│      [ Excel Workbook .xlsx ]                                          │
│                │                                                       │
│                ▼ (Auto-extract COVER sheet)                            │
│      ┌─────────────────────────┐                                       │
│      │ In-App Metadata Form    │◀── User Manual Edits / Overrides      │
│      │ (title, issue, dates,   │    (review, complement, or full input)│
│      │  release no, logo, etc.)│                                       │
│      └───────────┬─────────────┘                                       │
│                  ▼                                                     │
│            metadata.json                                               │
│                                                                        │
│   3. Content Ingestion & Normalization:                                │
│      • DOCX / RTF ──▶ Markdown (*.md)                                  │
│      • XLSX sheets ─▶ CSV (*.csv) with stacked header '--'             │
│                                                                        │
│   4. AI Translation & Copyfit (with BPS Glossary)                      │
│                                                                        │
│   5. Profile-Driven Header/Footer & Slot Fill Generation               │
│      • DDA Footer: Bilingual title + Year + Outer Folios               │
│      • BRS Header: BRS No. + Release Date + Title & Month              │
│                                                                        │
│   6. Layout Interpretation & Export inside DesignCraft (PDF/X-4)       │
└────────────────────────────────────────────────────────────────────────┘
```

**DesignCraft is the layout interpreter.** All composition (Knuth–Plass), hyphenation, thread reflow, overflow/overset detection, rendering and PDF export happen inside a DesignCraft instance. Publication Orchestrator Studio never parses, patches or re-serializes `.designcraft` file internals.

---

## Publication Profiles & Header-Footer Architecture

Publication Orchestrator Studio parameterizes layout rules, document geometry, nomenclature, and header/footer styling into declarative **Publication Profiles** (adopting the architecture from [`crates/ui-egui/src/panels/pubgen/profiles.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/profiles.rs) and [`header_footer.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/header_footer.rs)).

### Supported Profiles

| Profil | Sektor | Facing Spreads | Format Kertas & Margin | Aturan Penomoran | Gaya Header / Footer |
|---|---|---|---|---|---|
| **BPS DDA** *(Daerah Dalam Angka)* | BPS Official | Ya (Spreads, facing pages) | A4 (Top: 20.2mm, Bottom: 20.0mm, Inside: 19.0mm, Outside: 15.0mm) | Front matter: Angka Romawi kecil (`i, ii, iii...`); Body: Angka Arab (`1, 2, 3...`) | **BPS DDA Bilingual Footer**: <br>• Baris 1: Judul Indo (`KECAMATAN KARAWANG TIMUR DALAM ANGKA`)<br>• Baris 2: Judul English + Right-Tab + Tahun (`EAST KARAWANG DISTRICT IN FIGURES\t2026`)<br>• Nomor halaman (folio) di margin luar. |
| **BPS BRS** *(Berita Resmi Statistik)* | BPS Official | Opsional / Single page flow | A4 / Custom (Top: 18.0mm, Bottom: 16.0mm, Left: 18.0mm, Right: 15.0mm) | Tidak ada front matter romawi terpisah; penomoran langsung angka Arab | **BPS BRS Running Header**: <br>• Header Baris 1: `BRS No. [no_rilis], [tgl_rilis]` (contoh: `BRS No. 61/10/32/Th. XXVIII, 1 Oktober 2026`)<br>• Header Baris 2: `[Judul Publikasi], [Bulan Rilis]`<br>• Folio simpel di bagian bawah. |
| **Custom / Corporate** | Umum | Bebas dipilih | A4 / Letter / A5 | Konfigurasi fleksibel | Running header klasik / polos |

### Profile Data Schema (`profiles.rs`)

```rust
pub struct PubgenProfile {
    pub id: String,                     // "bps_dda", "bps_brs", "corporate"
    pub name: String,                   // "BPS Daerah Dalam Angka"
    pub description: String,
    pub sector: ProfileSector,          // BpsOfficial, Corporate, Custom
    pub geometry: ProfileGeometry,      // page_size, facing_pages, margins_mm
    pub structure: ProfileStructure,    // front_matter, roman_lower, restart_body, dividers
    pub nomenclature: ProfileNomenclature, // chapter_prefix ("BAB"), table_prefix ("Tabel")
    pub sheet_routing: ProfileSheetRouting,// regex matching sheet names (RXX, CXX, tabel, cover)
    pub table_styling: ProfileTableStyling,// column_number_row (1)(2)(3), footer summary row
    pub header_footer: ProfileHeaderFooterSpec, // DDA footer vs BRS header preset
}
```

### Auto-Detection & Manual Selection
* **Auto-Detection (`detect_profile_from_sheets`):** Saat workbook Excel diimpor, nama-nama sheet dianalisis:
  - Jika terdapat sheet berawalan `R` (ulasan), `Tabel`, `TOC`, atau `DDA` → default ke **BPS DDA**.
  - Jika terdapat keyword `BRS`, `Indikator`, atau nomor rilis berita → default ke **BPS BRS**.
* **Manual Override:** Pengguna dapat mengganti profile kapan saja melalui dropdown di panel pengaturan (Right Panel). Mengganti profile langsung memperbarui margin default, format nomor halaman, dan template header/footer yang aktif.

---

## Document Model — Page Kinds

A pubgen publication is an **ordered list of page kinds**. Each kind = pubgen generator + DesignCraft template + slot binding; a new feature is a new kind or a new slot filler — never an architecture change:

| Kind | Input | Output path | Reusable Engine Component |
|---|---|---|---|
| cover | metadata (Excel/manual), cover template, AI art | slot fills | `crates/ui-egui/src/panels/pubgen/covers.rs`, `ai_cover.rs`, `crates/imagegen` |
| content | MD sections | `slot:body_flow` | `crates/textimport/src/docx.rs`, `rtf.rs` |
| table | CSV | styled table frame | `crates/ui-egui/src/panels/pubgen/excel.rs` (stacked `--` headers), `tables.rs` |
| chart | CSV | pubgen-rendered SVG → `paste.svg` | `crates/ui-egui/src/panels/pubgen/charts/` (`spec.rs`, `layout.rs`, `palette.rs`) |
| map | SVG map + XLSX data | colorized SVG → `paste.svg` | `charts/geojson.rs`, `charts/stock_maps.rs` |
| toc | content tree + page mapping (scoped) | `slot:toc_entries` | `crates/ui-egui/src/panels/pubgen/toc.rs` (two-pass inspection) |
| divider | section metadata | slot fills | `crates/ui-egui/src/panels/pubgen/divider_art.rs` |

---

## App Skeleton

```
pubgen/
  Cargo.toml            # own workspace; current egui/eframe versions or pinned to DesignCraft fork revs
  app/
    src/
      main.rs           # eframe entry & CLI arguments (--smoke, --headless, etc.)
      main_ui.rs        # App state container and frame orchestration
      control_client.rs # TCP client for the control protocol (inverse of DesignCraft's control_server.rs)
      process.rs        # spawn/manage designcraft --control and designcraft-cli headless runs
      project.rs        # open/save .pubgen project containers (zip + atomic rename)
      i18n.rs           # GUI internationalization: Indonesian (default) & English dictionaries, tr(), switch widget
      profiles.rs       # publication profiles (BPS DDA, BPS BRS, schema, auto-detect)
      header_footer.rs  # profile-specific header & footer line resolvers and frame placement
      vector.rs         # SVG generation: charts (from CSV) and choropleth maps (SVG + XLSX)
      glossary.rs       # term base: prompt injection + post-translation enforcement
      covers.rs         # cover template grid + AI art/copy candidates
      toc.rs            # TOC entries, scope filters, two-pass page numbers
      ingest.rs         # background converters: DOCX/RTF → MD, XLSX → CSV
      metadata_form.rs  # interactive metadata form (hybrid: Excel pre-fill + manual input)
      slots.rs          # template slot discovery, char budgets, inspection traversal
      translate.rs      # LLM call + copyfit retry loop (Groq/OpenAI client with 429 backoff)
      panels.rs         # 3-column UI layout (Source & Content, Run & Verification, Slots & Run)
```

---

## Core Operational Modules & Code Reuse Mapping

### 1. GUI Internationalization & Bilingual Interface (`i18n.rs`)

Antarmuka pengguna (GUI) Publication Orchestrator Studio mendukung dua bahasa: **Bahasa Indonesia (default)** dan **English**, dengan opsi pergantian bahasa instan (*instant language switcher*) di bagian Top Bar.

* **Arsitektur `i18n.rs`:**
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
  pub enum UiLang {
      #[default]
      Id, // Bahasa Indonesia (Default)
      En, // English
  }

  impl UiLang {
      pub fn label(self) -> &'static str {
          match self {
              Self::Id => "Bahasa Indonesia",
              Self::En => "English",
          }
      }
      pub fn flag(self) -> &'static str {
          match self {
              Self::Id => "🇮🇩 ID",
              Self::En => "🇬🇧 EN",
          }
      }
  }

  pub fn tr(lang: UiLang, key: &str) -> &'static str { ... }
  ```
* **Cakupan Terjemahan:**
  - **Panel Headers:** `"Sumber & Konten"` / `"Source & Content"`, `"Eksekusi & Verifikasi"` / `"Run & Verification"`, `"Inspektor Slot & Eksekusi"` / `"Slot Inspector & Run"`.
  - **Form Labels:** `"Judul Publikasi"` / `"Publication Title"`, `"Tahun Rilis"` / `"Release Year"`, `"Profil Publikasi"` / `"Publication Profile"`, `"Format Kertas"` / `"Paper Size"`, `"Margin"` / `"Margins"`.
  - **Action Buttons:** `"Impor Dokumen"` / `"Import Documents"`, `"Simpan Proyek"` / `"Save Project"`, `"Ekspor PDF"` / `"Export PDF"`, `"Buka di DesignCraft"` / `"Open in DesignCraft"`.
  - **Status & Dialogs:** Pesan log eksekusi, notifikasi overset, peringatan konfirmasi simpan.
* **Reusable Source:**
  - Mengadopsi kosakata dari [`crates/ui-egui/src/panels/pubgen/locale.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/locale.rs) (`ID_WORDS`, `EN_WORDS`, `Locale::Id`, `Locale::En`).
  - Desain non-blocking: perubahan bahasa di Top Bar langsung memicu `request_repaint()` egui dan merender ulang seluruh teks seketika tanpa memerlukan restart aplikasi.

---

### 2. Hybrid Metadata Ingestion & Content Normalization (`ingest.rs`, `metadata_form.rs`)

Accepted inputs are normalized in the background to a small internal format set:

* **Hybrid Metadata Workflow (`metadata_form.rs`):**
  1. **Excel Auto-Extraction:** Pengguna dapat mengimpor file Excel (.xlsx). Jika file tersebut memiliki sheet `COVER` atau baris metadata kunci/nilai, fungsi `parse_cover_sheet` dari [`crates/ui-egui/src/panels/pubgen/excel.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/excel.rs) otomatis mengekstrak:
     - `title1`, `title2`, `title_en` (Judul dan terjemahan bahasa Inggris).
     - `year`, `volume`, `catalogue`, `code`, `release_no`, `release_date`, `release_month`.
     - `agency`, `pub_office`, `office_address`, `pic_name`, `pic_email`, dll.
  2. **Manual In-App Form Editing:** Seluruh data yang diekstrak ditampilkan dalam form egui interaktif (dengan label yang mengikuti bahasa aktif `UiLang`). Pengguna bebas mengedit, mengoreksi, atau menambahkan informasi yang belum ada di Excel.
  3. **Persistensi:** Data gabungan disimpan ke dalam `metadata.json` berstruktur standar di dalam container `.pubgen`.
* **Paragraph Content (DOCX / RTF → Markdown):**
  - **Reusable Source:** [`crates/textimport/src/docx.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/textimport/src/docx.rs) & [`rtf.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/textimport/src/rtf.rs).
  - Parser AST lengkap mengekstrak headings, numbering lists (`docx_numbering.rs`), bold, italic, dan paragraf menjadi Markdown standar.
* **Tables (XLSX → CSV):**
  - **Reusable Source:** [`crates/ui-egui/src/panels/pubgen/excel.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/excel.rs) (`parse_xlsx_sheets`, `convert_excel_to_csv_data`, `matches_table_note_keyword`).
  - Mendeteksi sel merger dan mengubah header bertingkat menjadi format stacked header `--`.
  - Memisahkan catatan tabel (`Sumber:`, `Catatan:`) secara otomatis sesuai profil yang aktif.
* **CSV Parsing & Heuristics:**
  - **Reusable Source:** [`crates/ui-egui/src/panels/pubgen/csv.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/csv.rs) (`detect_delimiter`, `parse_csv_content_with_delim`).

---

### 3. Header & Footer Engine (`header_footer.rs`)

Mengimplementasikan resolver teks dan pembuatan frame running header/footer sesuai publication profile yang dipilih:

* **BPS DDA Footer Resolver (`resolve_dda_footer_lines`):**
  - **Sumber:** [`crates/ui-egui/src/panels/pubgen/header_footer.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/header_footer.rs).
  - Menghasilkan dua baris teks footer:
    - Line 1: Judul Bahasa Indonesia (contoh: `KECAMATAN KARAWANG TIMUR DALAM ANGKA`).
    - Line 2: Judul Bahasa Inggris + right-aligned tab (`\u{e006}` atau `\t`) + Tahun publikasi (contoh: `EAST KARAWANG DISTRICT IN FIGURES\t2026`).
  - Folio halaman diletakkan di sudut luar spread (halaman ganjil di kanan luar, halaman genap di kiri luar).
  - Front matter menggunakan angka Romawi kecil (`i, ii, iii`), dan halaman bab utama mereset penomoran ke angka Arab (`1, 2, 3`).
* **BPS BRS Running Header Resolver (`resolve_brs_header_texts`):**
  - **Sumber:** [`crates/ui-egui/src/panels/pubgen/header_footer.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/header_footer.rs).
  - Menghasilkan dua baris teks running header:
    - Line 1: `BRS No. [no_rilis], [tgl_rilis]` (contoh: `BRS No. 61/10/32/Th. XXVIII, 1 Oktober 2026`).
    - Line 2: `[Judul Utama], [Bulan Rilis]` (contoh: `Perkembangan Nilai Tukar Petani, September 2026`).
  - Format tanggal dan bulan otomatis diformat dengan kaidah penulisan resmi Indonesia (`format_indonesian_date`, `format_indonesian_month_year`).
* **Injeksi ke Dokumen DesignCraft:**
  - Melalui perintah `engine.execute` (`frame.create` pada master spread atau target pages, `story.setText`, `object.rename` dengan nama slot `header:running` atau `footer:running`).

---

### 4. Project File — `.pubgen` (`project.rs`)

Project file adalah ZIP container mandiri berakhiran `.pubgen`:

```
DDA2026.pubgen (zip)
├── pubgen.json        # manifest: format_version, profile_id ("bps_dda" | "bps_brs"), ui_lang, slot mapping
├── metadata.json      # dari hybrid metadata (Excel + form manual)
├── content/           # normalized sources (*.md, *.csv)
├── assets/            # gambar, logo, template cover
├── glossary.json      # BPS terminology glossary
└── translations.json  # per-slot translated text + fit status
```

* **Reusable Source:**
  - [`crates/ui-egui/src/panels/pubgen/compile.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/compile.rs): Schema `ProjectManifest` (menyimpan `profile: Option<String>`), validasi atomik temp+rename save.
  - [`crates/ui-egui/src/panels/pubgen/excel.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/excel.rs): `create_csv_zip`.

---

### 5. Template & Slot Convention (`slots.rs`)

Templates adalah file native `.designcraft`. Publication Orchestrator Studio mendeteksi slot melalui `document.inspect` tanpa membaca internal file secara langsung:
* Nama slot semantic: `slot:cover_image`, `slot:title`, `slot:subtitle`, `slot:issue_meta`, `slot:body_flow`, `slot:table_data`.
* **Reusable Source:** [`crates/ui-egui/src/panels/pubgen/covers.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/covers.rs) (`walk`, `scan` traversal pohon inspect JSON).

---

### 6. AI Translation & Copyfit Engine (`translate.rs`, `glossary.rs`)

* **Reusable Source (API Client):** [`crates/translate/src/online.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/translate/src/online.rs) (`OnlineTranslator` dengan batching, 429 retry-after backoff, rate-limit header parsing, fence stripping).
* **Reusable Source (Glossary Engine):** [`crates/ui-egui/src/panels/pubgen/ai.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/panels/pubgen/ai.rs) (`BpsGlossary`, `GlossaryTerm`, `terms_from_csv`, `merge_terms`, `translatable_cell`).
* **Copyfit Loop Order:**
  1. Translate batch (strict JSON output).
  2. Glossary enforcement (word-boundary replacement, case-preserving).
  3. Character budget check (`max_chars`).
  4. If over-budget → LLM rephrasing prompt.
  5. Placement to DesignCraft → Engine overset check (`document.inspect` story overset flags) → 1 tighter retry → flag for human edit.

---

### 7. DesignCraft Connector (`process.rs`, `control_client.rs`)

* **Batch:** Spawn `designcraft-cli` headless per run via `designcraft-cli script <file>` atau direct `--cmd`.
* **Live/Interactive:** Launch `designcraft --control 7979` di background, komunikasi via loopback TCP client (`control_client.rs`).
* **Reusable Source:** [`apps/designcraft/src/control_server.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/apps/designcraft/src/control_server.rs) & [`crates/ui-egui/src/control.rs`](file:///e:/Github/designcraft-pubgen-0.7.X/crates/ui-egui/src/control.rs).

---

## Desktop GUI Layout (Bilingual ID/EN, Profiles, & Header/Footer)

```
+----------------------------------------------------------------------------------------------------+
│  [Logo] Publication Orchestrator Studio    │ Proyek: DDA_Karawang.pubgen  │ [🇮🇩 ID | 🇬🇧 EN] │ (● DesignCraft: OK)  │
+----------------------------------------------------------------------------------------------------+
│ KIRI: Sumber & Metadata   │ TENGAH: Eksekusi & Verifikasi           │ KANAN: Profil, Slot & Eksekusi│
│                           │                                         │                               │
│ [ + Impor Konten ]        │ ┌── Log Eksekusi ─────────────────────┐ │ ┌── Profil Publikasi ────────┐│
│ [ Impor Metadata Excel ]  │ │ Profil aktif: BPS DDA.              │ │ │ [ BPS DDA (Dalam Angka) ▼] ││
│                           │ │ 48 halaman dibuat dalam 420ms.      │ │ │ Margin: 20.2/20/19/15 mm   ││
│ ┌── Formulir Metadata ───┐│ │ DesignCraft: 0 oversets.            │ │ │ Spreads Berpasangan: Ya    ││
│ │ Judul: Karawang DDA    ││ ┌── Pratinjau Header / Footer ────────┐ │ └────────────────────────────┘│
│ │ Tahun: 2026            ││ │ Footer Baris 1: KECAMATAN...        │ │ ┌── Pengaturan Templat ─────┐│
│ │ Vol: 12  Kat: 1102001  ││ │ Footer Baris 2: KARAWANG... 2026    │ │ │ Templat: Modern_Cover_A    ││
│ │ [Edit semua bidang...] ││ └─────────────────────────────────────┘ │ │ Bahasa: ID - EN (Bilingual)││
│ └────────────────────────┘│ ┌── Kesesuaian Slot (Fit) ────────────┐ │ └────────────────────────────┘│
│ ┌── Pohon Konten ────────┐│ │ slot:title      42/50 kar   OK      │ │ ┌── Terdeteksi Slot (8) ────┐│
│ │ • artikel_tekno.docx→md││ │ slot:body_flow   88/90 kar  OK      │ │ │ ☑ slot:cover_image        ││
│ │ • tabel_01.xlsx→csv    ││ └─────────────────────────────────────┘ │ │ ☑ slot:title              ││
│ └────────────────────────┘│         [ Ekspor PDF ]  [ Buka PDF ]    │ │ ☑ footer:running (DDA)    ││
│ ┌── Komparasi Terjemahan ┐│         (buka viewer PDF sistem)        │ │ └──────────────────────────┘│
│ │ EN: "East Karawang"    ││                                         │ [ Buat Seluruh Dokumen ]      │
│ │ ID: "Karawang Timur"   ││ Log: PDF siap di target/DDA2026.pdf     │ [ Ekspor PDF/X-4 ] [Buka GUI] │
│ └────────────────────────┘│                                         │                               │
+----------------------------------------------------------------------------------------------------+
│ Profil: BPS DDA | Metadata: 18 bidang terisi (Excel + Manual)      │ Pekerja: Menganggur (Idle)     │
+----------------------------------------------------------------------------------------------------+
```

---

## Addendum: Phase 1 & 2 Audit & Reusability Maximization

Berdasarkan audit implementasi di folder `standalone/app/src/`, seluruh target milestone **Phase 1 (1a, 1b, 1c) dan Phase 2 telah terimplementasi dan lulus verifikasi (23/23 tests pass)**.

Berikut adalah evaluasi perbandingan antara implementasi saat ini dengan peningkatan fungsionalitas Publication Profile, Hybrid Metadata, dan Bilingual GUI:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        PHASE 1 & 2 IMPLEMENTATION & UPGRADE AUDIT                      │
├─────────────────────┬───────────────────────────────┬──────────────────────────────────┤
│ Modul               │ Status Implementasi Saat Ini  │ Peningkatan / Upgrade yang Ada   │
├─────────────────────┼───────────────────────────────┼──────────────────────────────────┤
│ Phase 1a: App Shell │ SELESAI (PASS, 741 cmd smoke) │ Sudah optimal.                   │
│ & Live Pipe         │ - TCP Client loopback 7979    │ - Tambah modul i18n.rs untuk     │
│                     │ - process.rs lifecycle spawn  │   UI dwibahasa (ID default).     │
│                     │                               │ - Status bar & Top bar dwibahasa.│
├─────────────────────┼───────────────────────────────┼──────────────────────────────────┤
│ Phase 1b: Ingestion │ SELESAI (PASS, unit-tested)   │ Upgrade Hybrid Metadata & Excel: │
│ & Normalization     │ - Minimal quick_xml docx/rtf  │ - Tambah parse_cover_sheet dari  │
│                     │ - Calamine XLSX sederhana     │   excel.rs untuk auto-fill form. │
│                     │ - MetadataForm interactive    │ - Integrasikan PubgenProfile     │
│                     │ - .pubgen zip save/load       │   (BPS DDA & BRS) ke metadata.   │
├─────────────────────┼───────────────────────────────┼──────────────────────────────────┤
│ Phase 1c: Slot Fill │ SELESAI (PASS, verified)      │ Upgrade Header/Footer Styles:    │
│ Batch               │ - inspect slot discovery      │ - Porting header_footer.rs untuk │
│                     │ - designcraft-cli script      │   resolve_dda_footer_lines dan   │
│                     │ - PDF rendering (76 glyphs)   │   resolve_brs_header_texts.      │
├─────────────────────┼───────────────────────────────┼──────────────────────────────────┤
│ Phase 2: AI &       │ SELESAI (PASS, unit-tested)   │ Upgrade LLM & Glossary:          │
│ Copyfitting         │ - Trait Translator (Llm/Mock) │ - Adopsi translate/online.rs     │
│                     │ - Strict JSON & 3x retry      │   (429 backoff & rate-limit).    │
│                     │ - Glossary word boundary      │ - Adopsi preset vocab BPS resmi. │
└─────────────────────┴───────────────────────────────┴──────────────────────────────────┘
```

---

## Phased Implementation Roadmap

### Phase 1a — App Shell + Live Pipe (Done)
- [x] Workspace `pubgen/` dengan eframe shell (window, status bar, log panel).
- [x] `process.rs`: launch `designcraft --control 7979`, health check.
- [x] `control_client.rs`: `engine.execute` round-trip. (`--smoke` mode: 741 commands verified, SMOKE PASS).
- [x] Clean-room: repository DesignCraft tetap tidak dimodifikasi.

### Phase 1b — Ingestion + Project Container + Hybrid Metadata (Done / In Progress Upgrade)
- [x] `ingest.rs`: konverter DOCX/RTF → MD dan XLSX → CSV.
- [x] `metadata_form.rs`: form metadata publikasi interaktif → `metadata.json`.
- [x] `project.rs`: simpan/buka `.pubgen` (ZIP container + versioned manifest), atomic temp+rename save.
- [x] **Hybrid Metadata Ingestion:** Auto-extract metadata dari sheet `COVER` file Excel (`parse_cover_sheet`) dengan kemampuan manual edit/override oleh pengguna.
- [x] **Publication Profiles Model:** Integrasi `profiles.rs` dengan profil bawaan **BPS DDA** dan **BPS BRS** serta deteksi otomatis profil dari daftar sheet (`detect_profile_from_sheets`).

### Phase 1c — Slot Fill Batch + Header/Footer Styles (Done / In Progress Upgrade)
- [x] `slots.rs`: slot discovery via `document.inspect` tanpa membaca file `.designcraft` internal.
- [x] Batch slot filling via `designcraft-cli script` headless → export PDF.
- [x] **Header/Footer Style Integration:** Penerapan otomatis styling **BPS DDA Footer** (bilingual title + year + folio luar) dan **BPS BRS Header** (nomor rilis + tanggal + judul) ke template DesignCraft.

### Phase 2: Translation & Copyfitting Pipeline (Done)
- [x] `translate.rs`: Trait `Translator` (klien OpenAI-compatible `LlmTranslator` + `MockTranslator` offline).
- [x] Strict JSON mapping per-slot dengan retry otomatis (maks 3x).
- [x] Copyfit rephrase loop: deteksi over-budget → rephrase → overset inspection check.
- [x] `glossary.rs`: Term base, do-not-translate protection, case-preserve, validasi word-boundary.
- [x] Penerjemahan tabel sel-per-sel dengan perlindungan angka dan simbol metrik.

---

### Phase 3: Desktop Interface Polish (Next Focus)

Mengembangkan 3-column GUI interaktif dengan Publication Profile switcher, Header/Footer preview, dan Bilingual UI (ID/EN):
1. **Top Bar Chrome (`top_bar`):**
   - Nama aplikasi, nama proyek aktif, status koneksi DesignCraft.
   - **Language Switcher Widget:** Toggle `[🇮🇩 ID | 🇬🇧 EN]` atau dropdown pemilihan bahasa UI (Indonesian default). Mengganti bahasa langsung merender ulang antarmuka tanpa lag.
2. **Left Panel (Content & Hybrid Metadata):**
   - Pohon konten sumber dinormalisasi (`.md`, `.csv`).
   - Tombol `Impor Metadata Excel` / `Import Excel Metadata` untuk mengisi form otomatis, dilengkapi form interaktif untuk review/edit manual.
   - Translation diff editor.
3. **Center Panel (Run & Verification & Header/Footer Preview):**
   - Live preview baris teks running header/footer yang di-resolve sesuai profil aktif.
   - Slot fit status (OK / Overset).
   - Preflight QA checklist via `preflight.pubgen`.
   - Action buttons: `Ekspor PDF` / `Export PDF`, `Buka di Viewer Sistem` / `Open in System Viewer`.
4. **Right Panel (Profile & Slot Inspector):**
   - Dropdown **Publication Profile** (`BPS DDA`, `BPS BRS`, `Custom`).
   - Slot inspector dengan toggle per-slot dan pengaturan gaya terjemahan.
   - Action buttons: `Buat Seluruh Dokumen` / `Generate Publication`, `Ekspor PDF/X-4` / `Export PDF/X-4`, `Buka di DesignCraft` / `Open in DesignCraft`.
5. **Process Lifecycle Hardening:**
   - Auto-reconnect TCP loopback, pembersihan child process saat Publication Orchestrator Studio ditutup.

**Kriteria Penerimaan (Phase 3 Acceptance):**
- [x] Antarmuka GUI mendukung dua bahasa (**Bahasa Indonesia default** dan **English**) dengan toggle bahasa di Top Bar.
- [x] Penggantian bahasa UI seketika mengubah semua label panel, header, tombol, tooltip, dan pesan log tanpa me-restart aplikasi.
- [x] Pengguna dapat memilih profil publikasi (BPS DDA vs BPS BRS) dan melihat perubahan parameter geometri serta preview header/footer secara instan.
- [x] Pengguna dapat mengisi metadata via file Excel (auto-fill) lalu mengedit/melengkapi nilainya di UI form.
- [x] Hasil PDF/X-4 yang dihasilkan menampilkan header/footer yang sesuai dengan profil terpilih (DDA Bilingual Footer atau BRS Running Header).
- [x] Audit QA dari perintah `preflight.pubgen` tampil di panel verifikasi.
- [x] Teardown bersih tanpa proses DesignCraft tertinggal di background.

---

## Phase 4: Feature Backlog (F1 s/d F5) (Completed)

Semua fitur backlog F1 s/d F5 telah terimplementasi murni di Rust (`standalone/app/src/`):
- [x] **F1 — Vector Charts (`vector.rs`):** Pure SVG vector chart generation dari data tabular CSV (Clustered Bar, Horizontal Bar, Line Chart, Donut Chart) dengan layout adaptif, grid lines, label numerik, dan palet warna modern.
- [x] **F2 — Choropleth Maps (`vector.rs`):** Parser GeoJSON poligon/multipoligon dengan komputasi bounding box, aspect ratio normalizer, continuous color ramp interpolation (`lerp_color`), serta ekspor SVG mandiri.
- [x] **F3 — Cover Studio & AI Art Brief (`covers.rs`):** Preset gaya sampul resmi (`Horizon`, `Modern`, `Tricolor`, `Minimal`) yang menghasilkan SVG siap pakai dengan logo, judul dwibahasa, dan elemen grafis. Terintegrasi dengan generator AI Art Brief / Prompt untuk Cloudflare flux & SDXL.
- [x] **F4 — Glossary of Terminology (`glossary.rs`):** Perlindungan DNT (Do Not Translate), penggantian istilah dengan case-preserving, dan terminologi resmi BPS.
- [x] **F5 — Table of Contents Engine (`toc.rs`):** Ekstraksi entri TOC dua-tahap (Bab, Tabel, Grafik) dari inspeksi dokumen aktif atau fallback daftar halaman proyek, dilengkapi formatter tab-leader dot (`. . .`) dwibahasa untuk story text DesignCraft.

**Kriteria Penerimaan (Phase 4 Acceptance):**
- [x] Unit test untuk `vector.rs` (Bar, Line, Donut, GeoJSON Map) lulus 100%.
- [x] Unit test untuk `covers.rs` (Horizon, Modern, AI Prompt Generation) lulus 100%.
- [x] Unit test untuk `toc.rs` (Inspect parser, Tab-leader formatter, TocKind filter) lulus 100%.
- [x] Seluruh modul backlog terintegrasi ke UI Panel samping kanan (`panels.rs`), mendukung ekspor SVG langsung dan salin clipboard satu-klik.
- [x] Lulus quality gate clippy (`-D warnings`) dan uji regresi keseluruhan (43/43 tests pass).

---

## Decisions & Progress Log

* **2026-11-02:** DesignCraft repo berstatus *reference-only*. Publication Orchestrator Studio beroperasi sebagai aplikasi mandiri di `pubgen/` (atau `standalone/`).
* **2026-11-02:** GUI standalone app ditetapkan **Bilingual (Bahasa Indonesia sebagai default, dan English)**, dilengkapi widget switcher di Top Bar dengan re-rendering reaktif seketika.
* **2026-11-02:** Alur kerja metadata ditetapkan sebagai **Hybrid**: otomatis diekstrak dari sheet `COVER` Excel, namun tetap dapat ditinjau dan diedit bebas melalui form interaktif.
* **2026-11-02:** Dukungan resmi untuk 2 profil publikasi utama BPS: **BPS DDA** (Daerah Dalam Angka) dan **BPS BRS** (Berita Resmi Statistik), masing-masing dengan geometri, margin, dan gaya header/footer khasnya (`resolve_dda_footer_lines` dan `resolve_brs_header_texts`).
* **2026-11-02 (Phase 1a Shipped):** App shell, process manager, dan TCP control client loopback (`127.0.0.1:7979`) selesai. Lulus uji 741 command smoke test terhadap instance DesignCraft.
* **2026-11-02 (Phase 1b Shipped):** Ingestion converter DOCX/RTF/XLSX, MetadataForm, dan container `.pubgen` berbasis ZIP dengan manifest terverifikasi.
* **2026-11-02 (Phase 1c Shipped):** Slot discovery melalui `document.inspect` dan batch headless script fill terbukti menghasilkan PDF yang valid.
* **2026-11-02 (Phase 2 Shipped):** Trait Translator, pipeline copyfit, glossary enforcement, dan penanganan tabel sel-per-sel selesai dan lulus 23/23 unit tests.
* **2026-11-02 (Phase 3 Shipped):** Desktop Interface Polish & Engine Wiring selesai: integrasi audit preflight QA (`preflight.pubgen`), auto-reconnect TCP client, live slot character fit tracker, tombol Buka di DesignCraft GUI, ekspor PDF/X-4, translation diff viewer, dan script builder running header/footer per profil publikasi. Lulus 33 unit tests dengan 0 warning.
* **2026-11-02 (Phase 4 Shipped):** Feature Backlog F1-F5 selesai: Modul `vector.rs` (Vector Charts & GeoJSON Choropleth Maps), `covers.rs` (Cover Presets & AI Image Prompt Briefs), dan `toc.rs` (Dua-tahap TOC engine). Seluruh 43 unit tests lulus dengan 0 warnings di clippy. Sub-repo Git disinkronisasi ke origin `https://github.com/3215appLabs/pubgen.git`.
* **2026-11-02 (Publication Profile & Page/Object Scope Upgrade):** Langkah 1 pemilihan Profil Publikasi (`BPS DDA`, `BPS BRS`, atau `Kustom`) diwajibkan di awal alur kerja antarmuka. Memilih profil langsung menginisialisasi parameter metadata sesuai arsitektur v0.8.3 (DDA: Facing Spreads, Margin 20.2/15/20/19mm, Dwibahasa ID/EN, Footer DDA; BRS: Single-sided, Margin 18/20/18/20mm, Bahasa Indonesia, Header BRS & No Rilis). Form Daftar Halaman/Konten kini mendukung 2 mode penambahan: `+ 📄 Tambah Halaman Baru` (`NewPage`) dan `+ 📦 Tambah Objek Baru` (`NewObject`), keduanya tersimpan di metadata `PageEntry.scope`. 44/44 unit tests lulus.
* **2026-11-02 (Excel Ingestion Rule BRS vs DDA):** Implementasi fungsi ekstraksi konten sheet Excel cerdas (`extract_workbook_content_entries`) sesuai aturan: untuk BRS (COV -> NewPage, POINT -> NewPage, U1 -> NewPage, UX/CX/TX/GX -> NewObject mengalir berurutan), sedangkan untuk DDA seluruh sheet diekstrak sebagai NewPage. Group UI disesuaikan menjadi "Daftar Konten Publikasi". 45/45 unit tests lulus.
