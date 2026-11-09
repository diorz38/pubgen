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

## 3. Langkah Demi Langkah: Membuat Publikasi DDA

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

---

## 5. Pertanyaan yang Sering Diajukan (FAQ)

### Q: Mengapa teks ulasan bahasa Inggris saya belum muncul di halaman?
> **Jawab:** Pastikan Anda sudah menjalankan tombol **Translate with AI** atau memastikan file `file_en` di Excel mengarah ke file bahasa Inggris yang berbeda dari file bahasa Indonesia.

### Q: Apakah saya harus terhubung ke internet saat menggunakan PubGen?
> **Jawab:** PubGen dapat berjalan secara **offline** untuk impor Excel, penyusunan tabel, tata letak halaman, dan ekspor. Koneksi internet hanya dibutuhkan jika Anda menekan tombol *Translate with AI* untuk menerjemahkan teks baru yang belum ada di memori.

### Q: Bagaimana jika ada revisi angka pada data tabel dari dinas terkait?
> **Jawab:** Cukup perbarui angka pada file Excel Anda, klik **Re-import / Update from Excel**, lalu klik **Generate Pages**. Seluruh halaman buku akan diperbarui secara otomatis tanpa merusak tata letak yang sudah ada.

