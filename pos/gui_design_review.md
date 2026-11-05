# Laporan Audit Kualitas GUI & UX/UI Review
**Publication Orchestrator Studio (POS) — Standalone Edition**  
*Evaluasi Antarmuka Berdasarkan Sudut Pandang Pengguna & 168 UX/UI Principles*

---

## 📌 1. Ringkasan Eksekutif & Skor Kesehatan Antarmuka

Audit ini mengevaluasi kualitas antarmuka grafis (**GUI**) aplikasi *Publication Orchestrator Studio (POS)* dari sudut pandang **Pengguna Nyata (*User Point of View*)**, yaitu Tim Penyusun Publikasi Statistik BPS (Statistisi, Editor Publikasi, dan Operator Diseminasi Data di tingkat Kabupaten/Kota hingga Pusat).

Evaluasi mengacu pada standar riset antarmuka internasional: **Nielsen’s 10 Usability Heuristics**, **Laws of UX (Jakob's Law, Fitts's Law, Hick's Law)**, serta pedoman aksesibilitas **WCAG 2.2 AA**.

### Matriks Skor Kualitas UX/UI

| Dimensi Evaluasi | Skor (1–10) | Status | Ringkasan Penilaian |
| :--- | :---: | :---: | :--- |
| **Kesesuaian Mental Model BPS (Heuristic 2)** | **9.6 / 10** | 🟢 Luar Biasa | Terminologi sangat akurat: DDA (Spreads/Facing Pages), BRS (Single Page), footer dwibahasa ID/EN, dan penomoran Romawi/Arab otomatis. Sampel bawaan 1-klik kini tersedia di halaman utama. |
| **Visibilitas Status Sistem (Heuristic 1)** | **9.5 / 10** | 🟢 Luar Biasa | Indikator IPC DesignCraft live, bar kesiapan 4-segmen (Ready/Partial/Empty), slot character budget live, serta indikator busy `⏳ Sedang menyusun dokumen...` di status bar. |
| **Konsistensi Desain & Kontras (Heuristic 4 / WCAG AA)** | **9.4 / 10** | 🟢 Luar Biasa | Desain token dan tema adaptif penuh. Kotak pratinjau Live Header/Footer kini menggunakan `POPOVER_BG_DARK` dan border adaptif pada mode gelap. |
| **Pencegahan Error & Keamanan Data (Heuristic 5 & 9)** | **9.0 / 10** | 🟢 Sangat Baik | Konfirmasi hapus 2-tahap (*armed delete*), penanganan tombol `Esc` global, audit preflight otomatis, serta tooltip/sub-teks penjelas cakupan `NewPage` vs `NewObject`. |
| **Efisiensi Alur & Beban Kognitif (Hick's / Fitts's Law)** | **9.2 / 10** | 🟢 Sangat Baik | Empty state interaktif bergaris putus-putus (*dropzone CTA*) di pohon konten, serta accordion toggle (*collapsible*) pada Header/Footer Preview dan Slot Character Fit. |

---

## 🧭 2. Evaluasi Berdasarkan Prinsip & Hukum UX (Laws of UX)

### A. Jakob’s Law (Kesesuaian dengan Ekspektasi Aplikasi Desktop Profesional)
* **Kekuatan:**
  * Pengguna yang terbiasa dengan perangkat lunak tata letak desktop profesional (Adobe InDesign, Figma, Affinity Publisher, Microsoft 365) langsung merasa familiar.
  * Aplikasi mengadopsi **Unified Application Bar (Frameless CSD)** di bilah atas: menggabungkan navigasi brand, menu drop-down desktop (`File`, `Publication`, `Tools`, `View`, `Help`), tombol kompilasi cepat, dan tombol caption window Windows (`—`, `□`, `✕`).
  * Panel kanan menyediakan kontrol kompilasi terdedikasi (`Compile PDF/X-4`, `Batch Generate`, `Simpan Proyek`, `Kelola Profil`).
  * Dukungan pintasan keyboard standar desktop: `Ctrl+S` untuk simpan, `Ctrl+Enter` untuk kompilasi, `F10` untuk fokus menu bar, dan `Esc` untuk menutup dialog.

### B. Hick’s Law (Beban Keputusan Pengguna / Cognitive Load)
* **Kekuatan:**
  * Alur pemilihan profil publikasi melalui **Modal Onboarding Wizard (A/B Variant B)** mengisolasi keputusan penting (Profil $\rightarrow$ Geometri $\rightarrow$ Metadata) ke dalam 3 langkah terpandu, mencegah *decision paralysis*.
  * **Collapsible Sections (Accordion)**: Grup *Live Header/Footer Preview* dan *Slot Character Fit* di panel tengah kini dapat dilipat/dibuka dengan satu klik. Pengguna dapat melipatnya untuk memaksimalkan ruang vertikal saat menyusun puluhan kartu publikasi.

### C. Fitts’s Law (Akurasi & Kecepatan Target Klik)
* **Kekuatan:**
  * Tombol aksi kompilasi (`Compile PDF/X-4`) diletakkan di sudut kanan atas dengan ukuran mencolok (*high visual prominence*) dan memiliki split-button drop-down yang mudah dijangkau.
  * Tombol aksi pada halaman Home (`Buka Berkas`, `Impor Manifest`, `Sampel BRS`, `Sampel DDA`) menggunakan ukuran pill button yang lapang dan mudah diakses.

---

## 🔍 3. Analisis Perjalanan Pengguna (User Journey Walkthrough)

### Tahap 1: Inisiasi Proyek & Onboarding (First Impression)
* **Pengalaman Pengguna:**
  * Halaman Home menyajikan kartu profil resmi BPS yang jelas (`DDA BPS`, `BRS BRS`, `Laporan Standar`).
  * **Fitur Baru (1-Click Sample Projects):** Tersedia tombol cepat **"📰 Sampel BRS"** dan **"📑 Sampel DDA"** langsung di hero action bar halaman Home. Pengguna pemula dapat langsung memuat dokumen sampel dan melihat hasil nyata tanpa harus menyiapkan file Excel terlebih dahulu.

### Tahap 2: Ingesti Berkas Data & Manifest Excel
* **Pengalaman Pengguna:**
  * Aturan ekstraksi otomatis sheet workbook Excel BRS (`COV`, `POINT`, `U1`, `CX`, `TX`, `GX`) sangat menghemat waktu editor.
  * **Fitur Baru (Actionable Empty State):** Ketika belum ada berkas sumber, pohon konten di panel kiri menampilkan tombol kotak interaktif bertema dashed: `[ 📂 Klik di sini untuk mengimpor berkas sumber data (.xlsx, .docx, .md) ]` yang langsung membuka dialog pemilihan berkas saat diklik.
  * **Fitur Baru (Kejelasan Scope):** Tombol tambah cepat dan formulir editor kartu kini dilengkapi tooltip serta sub-teks:
    * **Halaman Baru (`NewPage`):** *"Memaksa page-break dan memulai lembar baru."*
    * **Objek Baru (`NewObject`):** *"Mengalirkan objek ke sisa ruang halaman aktif."*

### Tahap 3: Penyuntingan Kartu Publikasi & Formulir Metadata
* **Pengalaman Pengguna:**
  * Grid dua kolom kartu ringkasan (*Summary Cards*) di panel tengah menjaga tampilan tetap rapi tanpa membuat formulir inline membengkak.
  * Membuka modal terfokus saat kartu diklik memberikan ruang yang cukup bagi pengguna untuk memeriksa metadata per halaman.
* **Mekanisme Keamanan:**
  * Sistem konfirmasi hapus kartu 2-tahap (*"Hapus kartu ini?"*) dengan pembatalan via tombol `Esc` sangat efektif mencegah terhapusnya data secara tidak sengaja (*error prevention*).

### Tahap 4: Preflight, Overset Checking, & Ekspor Akhir
* **Pengalaman Pengguna:**
  * **Bottom Readiness Bar (Sources | Pages | QA Fit | Status)**: Indikator kesiapan 4 segmen memberikan kepastian status secara instan.
  * **Fitur Baru (Busy State Feedback):** Saat kompilasi latar belakang atau proses AI berlangsung (`ui.busy = true`), segmen status otomatis menampilkan indikator aktif berwarna amber: `⏳ Sedang menyusun dokumen...` (atau `⏳ Processing document layout...` dalam mode EN).

---

## ⚠️ 4. Status Resolusi Temuan & UX Smells di Codebase

### 🟢 Temuan 1: Hardcoded Light Background pada Dark Theme (Bug Kontras) — [TERSELESAIKAN]
* **Tingkat Keparahan:** 🟡 Major (Visual Defect & Accessibility)
* **Lokasi File:** [`app/src/ui/panels/center.rs`](file:///app/src/ui/panels/center.rs)
* **Implementasi:**
  Mengganti hardcoded light color dengan token adaptif `theme::POPOVER_BG_DARK` dan border kontras tema gelap:
  ```rust
  background: Some(if dark {
      theme::POPOVER_BG_DARK
  } else {
      iced::Color::from_rgb8(250, 250, 252)
  }.into()),
  border: iced::Border {
      width: 1.0,
      color: if dark {
          iced::Color::from_rgb8(55, 60, 72)
      } else {
          iced::Color::from_rgb8(210, 210, 210)
      },
      radius: (4.0).into(),
      ..Default::default()
  },
  ```

---

### 🟢 Temuan 2: Empty State Pasif pada Panel Kiri (Dead-End State) — [TERSELESAIKAN]
* **Tingkat Keparahan:** 🟡 Moderate (Onboarding Flow)
* **Lokasi File:** [`app/src/ui/panels/left.rs`](file:///app/src/ui/panels/left.rs)
* **Implementasi:**
  Mengganti teks pasif redup dengan tombol interaktif bergaya dropzone CTA yang memicu `Message::Action("file_import_sources")`:
  ```rust
  let empty_btn = button(
      column![
          text(tr(lang, "no_sources")).size(12).color(theme::muted(dark)),
          text(tr(lang, "empty_sources_cta")).size(11).color(theme::ACCENT),
      ]
      .spacing(4)
      .align_x(iced::Alignment::Center),
  )
  .width(Length::Fill)
  .padding([12, 10])
  .on_press(Message::Action("file_import_sources"))
  ```

---

### 🟢 Temuan 3: Transparansi Status Proses Latar Belakang (Busy State Feedback) — [TERSELESAIKAN]
* **Tingkat Keparahan:** 🟡 Moderate (Visibility of System Status - Heuristic 1)
* **Lokasi File:** [`app/src/ui/panels/bottom.rs`](file:///app/src/ui/panels/bottom.rs)
* **Implementasi:**
  Mengintegrasikan status `ui.busy` langsung ke bilah status bawah (*bottom bar*) dengan status peringatan amber (*Partial readiness*):
  ```rust
  let (status, status_state) = if ui.busy {
      (
          format!("⏳ {}", tr(lang, "status_busy")),
          Readiness::Partial,
      )
  } else {
      (summary.status_text(lang), summary.status_state())
  };
  ```

---

### 🟢 Temuan 4: Kejelasan Konsep "Halaman Baru" vs "Objek Baru" — [TERSELESAIKAN]
* **Tingkat Keparahan:** 🟢 Minor / UX Copy Polish
* **Lokasi File:** [`app/src/ui/metadata.rs`](file:///app/src/ui/metadata.rs)
* **Implementasi:**
  Menambahkan tooltip pada tombol quick add (`NewPage` / `NewObject`) serta tooltip dan sub-keterangan dinamis di bawah ComboBox cakupan modal:
  * **Halaman Baru (`NewPage`):** `"Memaksa page-break dan memulai lembar baru."`
  * **Objek Baru (`NewObject`):** `"Mengalirkan objek ke sisa ruang halaman aktif."`

---

## 🛠️ 5. Log Eksekusi Rencana Aksi (Actionable Roadmap)

### Fase 1: Perbaikan Cepat (Quick Wins) — [SELESAI 100%]
- [x] **Perbaiki Kontras Preview Card Mode Gelap:** Container `preview_card` di `center.rs` adaptif terhadap mode gelap (`theme::POPOVER_BG_DARK`).
- [x] **Actionable Empty State di Panel Kiri:** Tombol CTA bergaris putus-putus interaktif di `left.rs` untuk langsung memicu impor berkas data.
- [x] **Indikator Busy Aktif di Bottom Bar:** Teks `⏳ Sedang menyusun dokumen...` di segmen status bottom bar saat `ui.busy = true`.

### Fase 2: Peningkatan Ergonomi Lanjutan — [SELESAI 100%]
- [x] **Collapsible Accordion Sections di Center Panel:** Grup *Live Header/Footer Preview* dan *Slot Character Fit* kini dapat dilipat/dibuka secara independen melalui `widgets::collapsible` (`Message::PanelToggled`).
- [x] **Sampel Bawaan 1-Klik di Home Page:** Tombol `📰 Sampel BRS` dan `📑 Sampel DDA` terintegrasi langsung pada bilah aksi utama di `home.rs`, otomatis mendeteksi keberadaan file sampel (`samples/brs/BRS_NTO_202610.pubgen` dan `samples/DUAL_LANG/BAB_01_2025_test.xlsx`).
- [x] **Verifikasi Internasionalisasi (i18n):** Seluruh label, tooltip, dan teks baru didukung dwibahasa penuh (ID & EN) dan lolos uji `all_keys_have_both_languages` (127 passing unit tests).
