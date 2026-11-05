# Publication Orchestrator Studio (POS) — Standalone Edition

Publication Orchestrator Studio (`POS`) adalah aplikasi desktop mandiri berbasis Rust dan `iced` yang dirancang untuk otomasi penyusunan dan penerbitan publikasi statistik (seperti publikasi resmi BPS: **Daerah Dalam Angka / DDA** dan **Berita Resmi Statistik / BRS**).

---

## 📌 Hubungan dengan Aplikasi DesignCraft

Aplikasi ini beroperasi dengan prinsip **clean-room architecture** dan terpisah secara decoupling penuh dari repositori DesignCraft:

1. **DesignCraft sebagai Layout Interpreter:**
   - Seluruh pemrosesan tata letak tingkat lanjut (Knuth–Plass paragraph composition, hyphenation, optical margin alignment, deteksi overflow/overset teks, serta ekspor standar industri ke PDF/X-4) dilakukan sepenuhnya oleh instance DesignCraft.
   - Publication Orchestrator Studio **tidak pernah** membongkar, mem-patch, atau mengompresi ulang file `.designcraft` secara langsung (*Direct File Synthesis ditiadakan*).
2. **Komunikasi Melalui Antarmuka Eksternal Resmi:**
   - **Mode Interaktif / Live:** Komunikasi via protokol JSON Lines melalui koneksi TCP loopback `127.0.0.1:7979` (`engine.execute`, `document.inspect`, `ui.paste`, `ui.render`).
   - **Mode Batch / Headless:** Eksekusi otomatis tanpa antarmuka menggunakan binary `designcraft-cli` (misalnya via `designcraft-cli script <batch_commands.jsonl>`).
3. **Format Dokumen Mandiri (`.pubgen`):**
   - Publication Orchestrator Studio memiliki format proyek sendiri berupa container ZIP berakhiran `.pubgen` (contoh: `DDA2026.pubgen`) yang menyimpan manifest (`pubgen.json`), metadata (`metadata.json`), konten yang dinormalisasi (`content/*.md`, `content/*.csv`), aset gambar, kamus istilah (`glossary.json`), dan terjemahan.

---

## 🚀 Fitur Utama

- **GUI Dwibahasa (Bilingual UI):** Mendukung **Bahasa Indonesia (default)** dan **English** dengan tombol pengalih bahasa instan di Top Bar tanpa perlu me-restart aplikasi.
- **Dukungan Multi-Publication Profile (Langkah Pertama Wajib):**
  - **Langkah 1 Onboarding:** Saat memulai proyek, pengguna diwajibkan memilih Profil Publikasi (**BPS DDA**, **BPS BRS**, atau **Custom**). Memilih profil otomatis menginisialisasi seluruh konfigurasi metadata dan geometri bawaan aplikasi POS v0.8.3.
  - **BPS DDA (*Daerah Dalam Angka*):** Format spreads/halaman ganda berpasangan (`Double`), margin resmi (Top 20.2mm, Bottom 20.0mm, Inside 19.0mm, Outside 15.0mm), dwibahasa ID/EN, penomoran romawi kecil untuk bagian pengantar dan angka Arab untuk bab isi, serta **BPS DDA Bilingual Footer** (Judul ID + Judul EN + Tahun + Folio luar).
  - **BPS BRS (*Berita Resmi Statistik*):** Format buletin/halaman tunggal (`Single`), margin resmi (Top 18.0mm, Bottom 18.0mm, Left 20.0mm, Right 20.0mm), Bahasa Indonesia, dengan **BPS BRS Running Header** (Nomor rilis BRS + Tanggal + Judul + Bulan rilis).
  - **Custom (*Kustom*):** Parameter bebas disesuaikan tanpa penimpaan otomatis.
- **Daftar Konten Publikasi dengan Opsi Fleksibel ("Halaman Baru" vs "Objek Baru"):**
  - Setiap penambahan item konten mendukung 2 mode yang tersimpan langsung di metadata (`PageEntry.scope`):
    - `+ 📄 Tambah Halaman Baru` (`NewPage`): Memaksa page break dan menghasilkan halaman baru.
    - `+ 📦 Tambah Objek Baru` (`NewObject`): Mengalirkan konten baru (*flowing vertical stack*) pada halaman aktif jika ruang vertikal mencukupi.
  - Setiap baris item di daftar konten dapat diubah cakupannya kapan saja melalui pemilih ComboBox interaktif.
  - **Aturan Ekstraksi Sheet Excel Cerdas:**
    - **Profil BRS:** Sheet `COV` -> metadata + Halaman Baru (`NewPage`), Sheet `POINT` -> Halaman Baru (`NewPage`), Sheet `U1` -> Halaman Baru (`NewPage`), sedangkan Sheet `UX` (selain U1), `CX`, `TX`, `GX` otomatis masuk sebagai Objek Baru (`NewObject`) yang disisipkan secara mengalir berurutan.
    - **Profil DDA:** Seluruh sheet diekstrak sebagai Halaman Baru (`NewPage`).
- **Tema Tampilan Adaptif (Dark / Light / System Mode):**
  - Mendukung 3 mode tema yang dapat dialihkan instan lewat bilah atas: **System Mode** (default, mengikuti tema OS Windows/macOS/Linux), **Light Mode** (terang), dan **Dark Mode** (gelap).
- **Alur Kerja Metadata Hybrid:**
  - Ekstraksi otomatis dari sheet `COVER` file workbook Excel (`.xlsx`), sekaligus menyediakan formulir interaktif di aplikasi untuk peninjauan, penyesuaian, atau pengisian manual.
- **Normalisasi Konten di Latar Belakang:**
  - DOCX / RTF dikonversi otomatis menjadi Markdown standar (`.md`).
  - Spreadsheet XLSX diurai menjadi CSV dengan dukungan merged headers bertingkat (`--`) dan pemisahan catatan tabel/sumber.
- **AI Translation & Copyfit Loop:**
  - Integrasi penerjemah OpenAI/Groq compatible (dan offline mock translator).
  - Penegakan terminologi baku melalui BPS Glossary engine sebelum pengecekan batas karakter (*character budget*) dan inspeksi overset layout.
- **Slot Discovery:**
  - Deteksi elemen slot pada template (`slot:cover_image`, `slot:title`, `slot:body_flow`, dll.) secara dinamis melalui perintah `document.inspect` DesignCraft.
- **Vector Charts Generator (SVG Murni):**
  - Pembuatan grafik vektor adaptif (Bar Clustered, Horizontal Bar, Line Chart, Donut Chart) dari tabel data tanpa dependensi eksternal, dengan palet warna terkoordinasi dan ekspor SVG instan.
- **Choropleth Maps Generator:**
  - Parser GeoJSON poligon & multipoligon dengan komputasi bounding box, penyesuaian rasio aspek, pewarnaan tematik kontinu (*color ramp interpolation*), serta integrasi tabel nilai wilayah.
- **Publication Cover Studio & AI Art Brief:**
  - Preset sampul publikasi resmi (**Horizon**, **Modern**, **Tricolor**, **Minimal**) beresolusi tajam dalam format SVG.
  - Generator prompt AI otomatis berbasis metadata untuk pembuatan gambar latar sampul menggunakan Cloudflare flux atau Stable Diffusion.
- **Table of Contents Engine (Daftar Isi, Tabel, & Grafik):**
  - Ekstraksi entri dua-tahap dari dokumen aktif DesignCraft atau fallback halaman proyek dengan format tab-leader dot (`. . .`) dwibahasa siap pakai untuk story text.
  - **Kartu TOC di Daftar Konten Publikasi:** jenis kartu `Daftar Isi (TOC)` dengan form yang sama seperti form pubgen TOC DesignCraft referensi (Judul TOC, Gaya TOC Entry/Table/Chart/Paragraph, Tata Letak Standard/TablePage, indent gantung, sertakan halaman TOC sendiri, pasangan dwibahasa ID/EN). Skrip generate menghitung halaman absolut TOC dari struktur halaman (sampul facing = 2 halaman) dan memanggil `toc.generate` + `autoflow` DesignCraft sehingga jumlah halaman TOC yang dihasilkan dinamis mengikuti jumlah entri dan halaman total.

---

## 🛠️ Panduan Penggunaan & Menjalankan Aplikasi

Pastikan Rust toolchain (Rust 2024 edition / Rust 1.85+) telah terinstal di komputer Anda.

### 1. Menjalankan di Mode Development
Untuk menjalankan Publication Orchestrator Studio dalam mode debug (cepat saat kompilasi):
```powershell
cargo run --manifest-path app/Cargo.toml
```
> UI menggunakan `iced` dan tidak memerlukan feature flag tambahan. Seluruh dependensi UI sudah menjadi dependensi default.

### 2. Menjalankan di Mode Release
Untuk menjalankan dalam mode teroptimasi performa tinggi:
```powershell
cargo run --release
```

### 3. Smoke Test Integrasi dengan DesignCraft
Publication Orchestrator Studio menyediakan mode verifikasi otomatis (`--smoke`) yang meluncurkan DesignCraft di latar belakang, membuka port TCP control `7979`, memverifikasi koneksi, dan menjalankan uji coba panggilan perintah `engine.commands`:
```powershell
cargo run -- --smoke
```
> **Catatan:** Secara default, aplikasi akan mencari binary `designcraft.exe` di folder `target/debug/` atau `target/release/`. Anda juga dapat menentukan lokasi binary secara eksplisit via environment variable:
> ```powershell
> $env:PUBGEN_DESIGNCRAFT_EXE = "E:\Github\designcraft-pubgen-0.7.X\target\debug\designcraft.exe"
> cargo run -- --smoke
> ```

---

## 🧪 Menjalankan Pengujian (Testing)

Proyek ini dilengkapi rangkaian unit tests untuk memvalidasi parser DOCX/RTF, ekstrasi XLSX/CSV, validasi metadata hybrid, resolver header/footer, profile switcher, serialisasi proyek `.pubgen`, dan alur copyfit AI:

### Menjalankan Seluruh Unit Tests
```powershell
cargo test
```

### Menjalankan Pemeriksaan Kode (Clippy Linter)
```powershell
cargo clippy --all-targets
```

---

## 📂 Struktur Direktori `standalone/`

```
standalone/
├── Cargo.toml               # Workspace manifest mandiri
├── Cargo.lock               # Pinned dependencies
├── README.md                # Dokumentasi proyek (file ini)
├── init_plan.md             # Dokumen arsitektur teknis dan roadmap implementasi
├── reusable_code_inventory.md # Inventaris pemetaan fungsi dari repositori DesignCraft
└── app/
    ├── Cargo.toml           # Manifest package bin pos
    ├── build.rs             # Menanamkan ikon & VERSIONINFO ke pos.exe (Windows)
    └── src/
        ├── main.rs          # Entry point & argumen CLI (--smoke)
        ├── main_ui.rs       # State utama aplikasi (App, tema, recent projects)
        ├── actions.rs       # Aksi murni (import, translate, batch fill, dll.)
        ├── branding.rs      # Ikon jendela runtime
        ├── ui/              # Shell iced: chrome, menu, home, panel, preview, metadata
        ├── i18n.rs          # Sistem dwibahasa (Bahasa Indonesia & English)
        ├── profiles.rs      # Engine profil publikasi (BPS DDA & BPS BRS)
        ├── header_footer.rs # Resolver baris header dan footer per profil
        ├── metadata_form.rs # Model metadata hybrid (Excel pre-fill + input manual)
        ├── ingest.rs        # Konverter background DOCX/RTF -> MD & XLSX -> CSV
        ├── project.rs       # Penanganan container file .pubgen (ZIP atomik)
        ├── control_client.rs# Klien TCP protokol JSON lines DesignCraft
        ├── process.rs       # Pengelola proses child DesignCraft / designcraft-cli
        ├── slots.rs         # Penemuan slot & budgeting karakter
        ├── audit.rs         # Preflight QA checklist audit parser & runner
        ├── translate.rs     # Pipeline penerjemahan AI & copyfit retry loop
        ├── glossary.rs      # Penyimpanan dan penegakan terminologi kamus istilah
        ├── covers.rs        # Preset sampul SVG & generator prompt AI art
        ├── vector.rs        # Engine grafik vektor SVG & choropleth maps GeoJSON
        └── toc.rs           # Mesin daftar isi, daftar tabel, dan grafik (two-pass)
```
