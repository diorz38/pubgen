# Panduan Penggunaan PubGen (Publikasi Generator BPS)
**Panduan Praktis Langkah demi Langkah untuk Penyusunan Publikasi DDA / BRS**

---

## 1. Mengenal PubGen

**PubGen** adalah aplikasi untuk membantu rekan-rekan BPS menyusun publikasi resmi (seperti **Kecamatan Dalam Angka / DDA** dan **Berita Resmi Statistik / BRS**) secara otomatis, cepat, dan rapi.

Dengan PubGen:
* **Tidak perlu lagi** copy-paste tabel dan teks satu per satu ke Word atau InDesign.
* **Teks bilingual (Indonesia & Inggris)** otomatis tersusun berdampingan dalam dua kolom.
* **Daftar Isi, Daftar Tabel, dan Daftar Grafik** otomatis tersusun dengan nomor halaman yang tepat.
* **Memori Terjemahan (TM):** Kata-kata standar statistik yang sudah pernah diterjemahkan akan tersimpan, sehingga tahun berikutnya tidak perlu menerjemahkan ulang.

---

## 2. Persiapan Sebelum Membuka Aplikasi

Untuk membuat publikasi, kita hanya membutuhkan **satu folder kerja** yang berisi:
1. **File Excel Publikasi** (misalnya: `BAB_01_2025_ai_translate.xlsx` dari folder sample).
2. **Folder Gambar (`images/`)** yang berisi:
   - `city_skyline.jpg` (foto sampul atas).
   - `city_landscape.jpg` (foto sampul bawah).
   - `bps_logo.png` (logo resmi BPS).
3. **Folder Teks Narasi (`rtf/`)** (opsional):
   - Berisi file naskah ulasan bab (misalnya `R01.rtf`, `B01.rtf`).

> [!TIP]
> **Contoh Lengkap Sudah Tersedia:**
> Anda bisa melihat contoh struktur file yang sudah siap pakai di folder:
> `plan/pubgen/samples/DUAL_LANG/`

---

## 3. Aturan Baku Penamaan Sheet Excel (*Sheet Naming Rules*)

Agar aplikasi PubGen dapat mengenali, mengelompokkan, dan menyusun halaman buku dengan benar, **struktur dan penamaan sheet di file Excel wajib mengikuti aturan berikut**:

### 1. Dilarang Mengubah Pola Nama Sheet Standar
Aplikasi membaca jenis konten berdasarkan nama sheet. Jangan mengubah format nama sheet standar:

| Pola Nama Sheet | Jenis Halaman / Konten | Keterangan & Contoh dari Sample XLSX |
|---|---|---|
| `COVER` | **Halaman Sampul Depan** | Berisi metadata buku (judul, tahun, katalog, ISSN, instansi, file foto). |
| `A02` *(atau `AXX`)* | **Halaman Hak Cipta / Redaksi** | Informasi tim penyusun, penerbit, dan legalitas publikasi. |
| `B01` *(atau `BXX`)* | **Halaman Kata Pengantar / Preface** | Naskah Kata Pengantar BPS dan tanda tangan pimpinan. |
| `R01`, `R02`, `R03`, `R04` | **Ulasan Deskriptif Bab (*Review Chapter*)** | Ulasan narasi bab dalam format teks dwibahasa (Indonesia - Inggris). |
| `G00`, `G01`, `G02` | **Halaman Grafik Statistik (*Charts*)** | Halaman infografis atau grafik visual data statistik. |
| `M03_02` | **Halaman Peta Tematik (*Maps*)** | Halaman visualisasi peta wilayah/tematik kecamatan. |
| `01_01`, `02_01_01`, dst. | **Tabel Data Statistik** | Tabel data lengkap dengan header baris/kolom, catatan, dan sumber. |
| `note` | **Catatan Teknis / Penjelasan Data** | Lembar penjelasan istilah statistik atau catatan teknis tambahan. |

### 2. Aturan Menambahkan Sheet Ulasan Bab (*Review Chapter*)
Jika Anda ingin menambahkan ulasan narasi bab baru:
* Nama sheet **wajib** diawali huruf kapital **`R`** dan diikuti **2 digit nomor bab** dengan *leading zero*.
* **Pola:** `RXX` (di mana `XX` adalah nomor bab `01` sampai `99`).
* **Contoh Benar:**
  - `R01` $\rightarrow$ Ulasan Bab 1 (Geografi dan Iklim)
  - `R02` $\rightarrow$ Ulasan Bab 2 (Pemerintahan)
  - `R03` $\rightarrow$ Ulasan Bab 3 (Kependudukan)
  - `R04` $\rightarrow$ Ulasan Bab 4 (Sosial)
  - `R05` $\rightarrow$ Ulasan Bab 5 (Pertanian)
* *Jangan gunakan nama seperti `Review01`, `R1`, atau `Ulasan Bab 1` karena aplikasi tidak akan mengenalinya sebagai kartu narasi dwibahasa.*

### 3. Aturan Menambahkan Sheet Tabel Data Statistik
Jika Anda ingin menambahkan sheet tabel baru ke dalam bab tertentu, ikuti rumus baku penamaan:
$$\mathbf{XX\_XX\_XXZZ}$$
* **`XX` Pertama:** 2 digit nomor Bab (*leading zero*, contoh: `01`, `02`, `03`, `04`).
* **`XX` Kedua:** 2 digit nomor Grup Data / Subbab (*leading zero*, contoh: `01`, `02`).
* **`XX` Ketiga:** 2 digit nomor Subgrup Data / Nomor Tabel (*leading zero*, contoh: `01`, `02`, `03`).
* **`ZZ` (Opsional):** Huruf penanda halaman lanjutan, contoh huruf **`L`** (*Lanjutan*) jika isi tabel tersebut terpotong dan berlanjut ke halaman berikutnya.

**Contoh Format Tabel:**
* `01_01` $\rightarrow$ Bab 1, Tabel 1 (format ringkas 2 level).
* `01_02` $\rightarrow$ Bab 1, Tabel 2.
* `02_01_01` $\rightarrow$ Bab 2, Subbab 1, Tabel 1.
* `02_02_01` $\rightarrow$ Bab 2, Subbab 2, Tabel 1.
* `02_02_02` $\rightarrow$ Bab 2, Subbab 2, Tabel 2.
* `03_01` $\rightarrow$ Bab 3, Tabel 1 (halaman pertama tabel).
* `03_01L` $\rightarrow$ **Lanjutan** dari tabel `03_01` (aplikasi otomatis memberi judul *"Lanjutan/Continued..."* pada header).
* `04_01_01` $\rightarrow$ Bab 4, Subbab 1, Tabel 1.
* `04_01_02` $\rightarrow$ Bab 4, Subbab 1, Tabel 2.
* `04_01_03` $\rightarrow$ Bab 4, Subbab 1, Tabel 3.

---

## 4. Langkah Demi Langkah: Membuat Publikasi DDA

### Langkah 1: Membuka Aplikasi dan Membuat Proyek Baru
1. Buka aplikasi **PubGen Studio**.
2. Pada panel atas / menu bar, klik menu **PubGen**.
3. Pilih **"Import from Excel..."** (atau klik tombol **Browse** pada kartu awal).
4. Arahkan dan pilih file Excel publikasi Anda, misalnya:
   `plan/pubgen/samples/DUAL_LANG/BAB_01_2025_ai_translate.xlsx`
5. Aplikasi akan otomatis membaca seluruh lembar kerja (*sheet*) dan membaginya menjadi beberapa **Kartu (Cards)**:
   - Kartu Sampul (*COVER*)
   - Kartu Kata Pengantar (*B01*)
   - Kartu Ulasan Narasi (*R01, R02, R03, R04*)
   - Kartu Tabel Data (*01_01, 02_01_01, dst.*)
   - Kartu Grafik (*G01, G02*)

---

### Langkah 2: Memeriksa Data dan Judul Publikasi
1. Klik pada **Kartu Pertama (Cover / Sampul)** di daftar kartu sebelah kiri.
2. Di sebelah kanan, periksa:
   - **Judul Publikasi (Indonesia):** contoh: `KECAMATAN KARAWANG TIMUR DALAM ANGKA`
   - **Judul Publikasi (Inggris):** contoh: `EAST KARAWANG DISTRICT IN FIGURES`
   - **Tahun Publikasi:** contoh: `2026`
   - **Nama Instansi / Kantor:** pastikan nama BPS Kabupaten/Kota sudah sesuai.
3. Anda bisa mengubah teks langsung jika terdapat salah ketik.

---

### Langkah 3: Mengenal Jenis Kartu & Aturan Format Nomor Halaman (*Page Number Style*)

Di PubGen, publikasi dibagi menjadi beberapa **jenis kartu** dengan aturan penomoran halaman otomatis:

#### A. Jenis-Jenis Kartu di PubGen:
1. **Kartu Sampul (*Cover*):**
   - Berisi informasi judul publikasi, foto sampul, dan logo instansi.
2. **Kartu Bagian Depan (*Front Matter* / Kata Pengantar `B01` & Daftar Isi):**
   - Berisi naskah *Kata Pengantar* (Indonesia), *Preface* (Inggris), serta *Daftar Isi*, *Daftar Tabel*, dan *Daftar Grafik*.
3. **Kartu Pembatas Bab (*Chapter Divider*):**
   - Menandai awal bab baru (misalnya *BAB 01 Geografi dan Iklim*). Halaman ini menampilkan nomor bab, judul bab dua bahasa, dan daftar subbab.
4. **Kartu Ulasan Narasi (*Overview / Narrative* `R01, R02, dst.`):**
   - Ulasan deskriptif bab yang disajikan dalam **dua kolom berdampingan** (kiri: Indonesia, kanan: Inggris).
5. **Kartu Tabel Data (`01_01, 02_01_01, dst.`):**
   - Menyajikan tabel statistik lengkap dengan judul bilingual, kepala kolom, baris data, catatan kaki (*Notes*), dan sumber (*Source*).
6. **Kartu Grafik (`G01, G02, dst.`):**
   - Berisi visualisasi grafik statistik beserta judul dan sumber data.

#### B. Aturan Penomoran Halaman (*Page Numbering Rules*):
PubGen secara otomatis menerapkan standar buku publikasi resmi:
* **Bagian Awal (*Front Matter*): Angka Romawi Kecil (`i, ii, iii, iv, v, ...`)**
  - **Sampul (`Cover`):** Halaman `i` (nomor disembunyikan/tidak dicetak di sampul).
  - **Kata Pengantar:** Halaman `ii`.
  - **Preface (Inggris):** Halaman `iii` (otomatis disisipkan setelah Kata Pengantar).
  - **Daftar Isi & Daftar Tabel:** Halaman `iv, v, ...` seterusnya.
* **Bagian Isi (*Body* / Bab & Tabel): Angka Arab (`1, 2, 3, 4, ...`)**
  - Dimulai kembali dari **halaman 1** pada lembar pertama **Bab 1** (*Chapter Divider* atau *Ulasan*).
  - Berlanjut terus sampai halaman terakhir publikasi.
* **Aturan Posisi Nomor Halaman (Header / Footer):**
  - **Halaman Ganjil (Kanan):** Nomor halaman berada di pojok kanan luar.
  - **Halaman Genap (Kiri):** Nomor halaman berada di pojok kiri luar.
  - Di sebelah nomor halaman tercantum *Running Title* (judul bab / judul publikasi) secara bilingual.

---

### Langkah 4: Menerjemahkan ke Bahasa Inggris (Dual-Language)

PubGen menyediakan fitur terjemahan otomatis berbasis AI dan Memori Terjemahan:

1. Di bilah menu atas atau panel PubGen, klik tombol **"Terjemahkan dengan AI"** (*Translate with AI*).
2. **Pemberitahuan Aman:** Jika publikasi Anda sudah memiliki terjemahan sebelumnya, aplikasi akan memunculkan dialog konfirmasi agar data terjemahan Anda tidak tertimpa tanpa sengaja.
3. Aplikasi akan mengisi:
   - Judul-judul tabel ke bahasa Inggris (*List of Tables*).
   - Kepala kolom tabel (*Table Headers*).
   - Ulasan narasi pada kolom kanan (*Overview*).
4. Anda dapat membaca dan mengoreksi hasil terjemahan langsung di kartu terkait. Kata yang telah Anda ubah akan otomatis tersimpan sebagai terjemahan resmi (*Reviewed*).

---

### Langkah 5: Menyimpan Proyek (*Save Project*)
Sebelum mencetak halaman:
1. Klik tombol **"Save Project"** (atau tekan `Ctrl + S`).
2. Beri nama file, misalnya: `KECAMATAN KARAWANG TIMUR DALAM ANGKA.pubgen`.
3. **Penting:** Semua gambar, tabel, teks narasi, dan riwayat terjemahan akan tersimpan utuh di dalam satu file `.pubgen` ini. File ini bisa Anda kirimkan ke rekan kerja lain.

---

### Langkah 6: Menghasilkan Halaman Publikasi (*Generate Pages*)

Ini adalah langkah utama yang menyusun seluruh buku publikasi:

1. Klik tombol **"Generate Pages"** (atau **Compile Publication**).
2. Aplikasi akan menyusun halaman secara berurutan:
   - **Halaman Romawi (Front Matter):**
     - Halaman i: Sampul Depan (*Cover*).
     - Halaman ii: Kata Pengantar (*Indonesian*).
     - Halaman iii: *Preface* (*English*).
     - Halaman iv - v: Daftar Isi (*Table of Contents*), Daftar Tabel (*List of Tables*), dan Daftar Grafik (*List of Charts*).
   - **Halaman Isi (Bab dan Tabel):**
     - Halaman Pembatas Bab (*Chapter Divider*).
     - Halaman Narasi Ulasan Bilingual (Kolom kiri bahasa Indonesia, kolom kanan bahasa Inggris).
     - Halaman Tabel Data lengkap dengan catatan kaki dan sumber data.
3. Di bagian bawah setiap halaman genap dan ganjil, *running footer* akan menampilkan judul resmi dalam dua bahasa secara otomatis:
   ```text
   KECAMATAN KARAWANG TIMUR DALAM ANGKA
   EAST KARAWANG DISTRICT IN FIGURES
   ```

---

### Langkah 7: Mengekspor ke PDF Siap Cetak
1. Di menu utama, pilih **File** $\rightarrow$ **Export** $\rightarrow$ **PDF**.
2. Pilih folder tujuan dan klik **Save**.
3. File PDF siap dicetak atau diunggah ke website resmi BPS.

---

## 4. Penjelasan Istilah & Tombol di Aplikasi

| Tombol / Bagian | Fungsi Sederhana |
|---|---|
| **Cards (Daftar Kartu)** | Bagian-bagian buku (sampul, bab, tabel, grafik) yang disusun seperti kartu. Anda bisa mengklik kartu mana saja untuk melihat isinya. |
| **Translate with AI** | Menerjemahkan judul, narasi, dan kepala tabel ke bahasa Inggris secara otomatis. |
| **Save Project (.pubgen)** | Menyimpan seluruh pekerjaan Anda dalam satu file paket lengkap. |
| **Generate Pages** | Tombol ajaib yang menyusun seluruh teks, tabel, dan gambar menjadi halaman buku siap cetak. |
| **Running Header / Footer** | Teks di bagian atas dan bawah halaman yang menunjukkan nomor halaman dan judul publikasi. |

### Kelompok Peran Terjemahan (*Translation Unit Roles*)
Saat meninjau hasil terjemahan di panel sebelah kanan, unit terjemahan dikelompokkan ke dalam 4 kategori utama:

| Kelompok | Jenis / Peran Unit | Keterangan / Sumber Teks |
|---|---|---|
| **1. Page Title** | `cover_title` | Judul utama dan subjudul pada halaman sampul (`COVER`) |
| | `table_title` | Judul dan keterangan tabel statistik |
| | `chart_title` | Judul grafik statistik |
| | `map_title` | Judul dan keterangan peta tematik |
| | `image_caption` | Teks keterangan gambar / infografis |
| | `divider_title` | Judul halaman pembatas bab (*Divider*) |
| | `card_title` | Judul umum pada kartu publikasi |
| **2. Paragraph Content** | `narrative_title` | Judul bab dan subbab naskah ulasan |
| | `narrative_body` | Paragraf teks isi ulasan narasi bab |
| **3. Table Content** | `table_header` | Kepala kolom dan sub-kolom tabel data |
| | `table_row_label` | Baris pertama / label kategori baris pada tabel data |
| | `table_note` | Catatan kaki tabel dan sumber data (`Sumber: ...`) |
| **4. Other** | `chart_series` | Seri data dan label legenda grafik |
| | `toc_title` | Label judul Daftar Isi, Daftar Tabel, dan Daftar Grafik |
| | `header_footer` | Teks kepala dan kaki halaman (*Running Header/Footer*) |

---

## 5. Pertanyaan yang Sering Diajukan (FAQ)

### Q: Bagaimana jika file naskah ulasan bahasa Inggris (file_en) belum tersedia atau sama dengan file bahasa Indonesia?
> **Jawab:** Tidak masalah! Jika file `file_en` di Excel menunjuk ke file yang sama dengan `file` (seperti naskah BAB 03 & BAB 04), PubGen secara cerdas akan merekonstruksi naskah bahasa Inggris dari file bahasa Indonesia tersebut:
> 1. Judul bab otomatis menjadi `"REVIEW"`.
> 2. Paragraf dan heading yang sudah ada di memori terjemahan / AI akan diganti dengan bahasa Inggris.
> 3. Teks yang belum diterjemahkan tetap dipertahankan dengan format aslinya agar kolom kanan halaman tidak kosong.
> 4. Subbab di Daftar Isi (TOC) otomatis dipadukan (`Subbab Indonesia / English Subheading`).

### Q: Apakah saya harus terhubung ke internet saat menggunakan PubGen?
> **Jawab:** PubGen dapat berjalan secara **offline** untuk impor Excel, penyusunan tabel, tata letak halaman, dan ekspor. Koneksi internet hanya dibutuhkan jika Anda menekan tombol *Translate with AI* untuk menerjemahkan teks baru yang belum ada di memori.

### Q: Bagaimana jika ada revisi angka pada data tabel dari dinas terkait?
> **Jawab:** Cukup perbarui angka pada file Excel Anda, klik **Re-import / Update from Excel**, lalu klik **Generate Pages**. Seluruh halaman buku akan diperbarui secara otomatis tanpa merusak tata letak yang sudah ada.

