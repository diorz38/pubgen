# Pubgen Studio: Development Guide & Architecture Manual

Dokumen ini berisi panduan teknis, arsitektur, struktur kode, dan alur kerja pengembangan untuk **Pubgen Studio** (`apps/pubgen-studio`), aplikasi desktop dual-mode yang menggabungkan orkestrasi penerbitan otomatis (*Publication Orchestrator*) dengan kanvas tata letak interaktif (*DesignCraft Layout Canvas*).

---

## 1. Visi & Konsep Dasar

Pubgen Studio dirancang menggunakan **Workspaces / Perspectives Pattern** (seperti DaVinci Resolve atau Blender) untuk menyelesaikan pemisahan lama antara aplikasi POS (standalone) dan DesignCraft:

1. **Satu Jendela Terpadu:** Pengguna tidak lagi menjalankan dua aplikasi terpisah yang terhubung melalui port TCP loopback.
2. **Pemisahan Peran yang Jelas:**
   - **Recipe (`.pubgen`):** Menyimpan *apa* isi publikasi (metadata, sheet Excel, teks markdown, kamus istilah, pilihan sampul, dan aset).
   - **Artifact (`.designcraft`):** Menyimpan *bagaimana* dokumen ditata secara visual (koordinat frame, tipografi mikro, perataan optik, penyesuaian manual).
3. **Isolasi Penuh dari Upstream:** Seluruh kode Pubgen Studio berada di direktori `apps/pubgen-studio/`, sehingga pembaruan dari repository upstream `storytold/designcraft` dapat di-pull kapan saja **tanpa risiko konflik merge**.

---

## 2. Arsitektur Dual-Mode (`StudioMode`)

```text
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ [Pubgen Studio]    [ 📊 Orchestrator (Active) ]  [ 🎨 DesignCraft Layout (🔒/Active) ]   │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Menu Bar Dinamis:                                                                       │
│ • Orchestrator Mode: File | Profiles | Ingest | Glossary | Covers | Preflight           │
│ • DesignCraft Mode:  File | Edit | Layout | Type | Object | View | Window               │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### A. `StudioMode::Orchestrator` (Default saat aplikasi dibuka)
* **Fokus:** Struktur data dan resep publikasi tingkat tinggi.
* **Fitur:** Pemilihan profil (BPS DDA / BPS BRS / Custom), impor workbook Excel, pemetaan sheet, formulir metadata, studio sampul, generator daftar isi (TOC), dan audit preflight.
* **Bebas Distraksi:** Pengguna tidak terganggu oleh 50+ tool DTP saat sedang menyiapkan data.
* **Tombol Utama:** `[ 🚀 Compile & Open in Layout ]`.

### B. `StudioMode::DesignCraft`
* **Fokus:** Tata letak dan penyempurnaan grafis interaktif.
* **Aktif:** Terbuka otomatis setelah proses kompilasi halaman selesai, atau saat pengguna membuka file `.designcraft`.
* **Fitur:** Kanvas interaktif, seleksi objek, penyesuaian teks, swatch warna, layer, panel tipografi, dan ekspor standar industri ke PDF/X-4.
* **Navigasi Bebas:** Pengguna dapat berpindah kembali ke Orchestrator kapan saja melalui pill switcher di Top Bar.

---

## 3. Struktur Folder & Modul

```text
pubgen/
├── apps/
│   ├── designcraft/              # App desktop bawaan upstream (tidak dimodifikasi)
│   ├── designcraft-cli/          # CLI bawaan upstream
│   ├── pubgen-cli/               # CLI compiler headless
│   └── pubgen-studio/            # 🌟 Executable Studio Dual-Mode
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # Entry point, file sniffer (.pubgen vs .designcraft)
│           ├── app.rs            # Root PubgenStudioApp struct (implementasi eframe::App)
│           ├── mode.rs           # StudioMode enum (Orchestrator vs DesignCraft)
│           ├── chrome/
│           │   ├── mod.rs
│           │   └── top_bar.rs    # Mode switcher pills, status badge, project name
│           ├── project/
│           │   ├── mod.rs
│           │   └── container.rs  # ZIP container manager (.pubgen) & atomic writer
│           ├── domain/           # Domain logic yang di-harvest dari POS
│           │   ├── mod.rs
│           │   ├── brs.rs        # Parser buletin BPS BRS & estimasi tinggi teks
│           │   ├── maps.rs       # Generator peta tematik koroplet SVG murni (GeoJSON)
│           │   ├── glossary.rs   # Penegakan kamus istilah resmi BPS & case matching
│           │   └── ingest.rs     # Parser DOCX/XML ke format Markdown bersih
│           └── workspaces/
│               ├── mod.rs
│               ├── orchestrator/
│               │   ├── mod.rs    # Mount panel pubgen dari crates/ui-egui
│               │   └── guard_dialog.rs # Dialog proteksi timpa layout (3-way modal)
│               └── designcraft/
│                   └── mod.rs    # Mount DocumentCanvas & panel dock DesignCraft
```

---

## 4. Cara Menjalankan dalam Mode Development (Dev Mode)

Pastikan Anda berada di direktori `e:\Github\pubgen`:

### A. Menjalankan Mode Standar (Orchestrator Kosong)
```powershell
cargo run -p pubgen-studio
```

### B. Menjalankan dan Membuka Proyek `.pubgen`
Aplikasi akan langsung membuka proyek dan masuk ke **📊 Orchestrator Mode**:
```powershell
cargo run -p pubgen-studio -- "path/ke/proyek.pubgen"
```

### C. Menjalankan dan Membuka Dokumen Layout `.designcraft`
Aplikasi akan langsung memuat dokumen ke kanvas dan masuk ke **🎨 DesignCraft Mode**:
```powershell
cargo run -p pubgen-studio -- "path/ke/dokumen.designcraft"
```

### D. Menjalankan dengan Logging Debug Aktif
Membantu melihat pesan error internal, trace rendering wgpu, dan proses compiler:
```powershell
$env:RUST_LOG="info"
cargo run -p pubgen-studio
```

### E. Menjalankan Unit Test
```powershell
cargo test -p pubgen-studio
```

### F. Build Binary Release
```powershell
cargo build --release -p pubgen-studio
```
Binary hasil kompilasi akan berada di `target/release/pubgen-studio.exe`.

---

## 5. Fitur Kunci & Mekanisme Pengamanan

### A. Proteksi Penimpaan Layout (`RecompileGuardDialog`)
Jika pengguna telah melakukan penyesuaian layout manual di kanvas DesignCraft, menekan tombol kompilasi ulang di Orchestrator tidak akan menghapus pekerjaan mereka secara diam-diam. Sistem memunculkan dialog 3 opsi:
1. **💾 Save as Version (v2):** Mempertahankan file layout lama dan mengompilasi hasil baru ke versi berikutnya (`publication_v2.designcraft`).
2. **⚠️ Overwrite Existing:** Menimpa layout yang ada dengan kompilasi segar dari data mentah.
3. **Cancel:** Membatalkan proses kompilasi tanpa mengubah apa pun.

### B. Format Kontainer Atomik (`PubgenContainer`)
Format `.pubgen` adalah kontainer ZIP yang menyimpan:
* `pubgen.json` (Manifest versi proyek)
* `metadata.json` (Judul, pengarang, profil BPS, parameter geometri)
* `glossary.json` (Kamus istilah dwibahasa)
* `content/*.md` & `content/*.csv` (Konten ternormalisasi)
* `output/publication.designcraft` (Cache hasil layout yang sudah terkompilasi)

Penyimpanan dilakukan secara **atomik**: file ditulis ke `.tmp` terlebih dahulu, lalu di-rename ke `.pubgen` asli guna mencegah kerusakan file jika daya mati atau proses terhenti di tengah jalan.

### C. Generator Peta Vektor Murni (`domain/maps.rs`)
Menghasilkan peta koroplet beresolusi tinggi langsung dalam format string SVG tanpa dependensi Python atau pustaka C eksternal:
* Parser poligon & multipoligon GeoJSON.
* Koreksi rasio aspek dengan proyeksi kosinus lintang Mercator (`cos_lat`).
* Interpolasi gradasi warna tematik kontinu (*color ramp interpolation*).

---

## 6. Sinkronisasi dengan Upstream DesignCraft

Untuk mengambil update fitur atau perbaikan bug terbaru dari repository induk (`storytold/designcraft`):

```powershell
# 1. Pastikan working tree bersih
git status

# 2. Ambil update dari upstream
git fetch upstream main

# 3. Gabungkan ke branch kerja Anda
git merge upstream/main
```

Karena `apps/pubgen-studio/` berada di namespace terpisah dan tidak memodifikasi file bawaan `apps/designcraft/`, proses merge akan berjalan lancar tanpa konflik teks pada antarmuka Studio.

