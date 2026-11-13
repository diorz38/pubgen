## 14. Tutorial Praktis Langkah demi Langkah

Tutorial ini dimulai dari pekerjaan yang paling sering menahan orang: menyiapkan isi publikasi di Excel. Setelah bahan siap, alurnya berlanjut ke impor ke Orchestrator, pemeriksaan kartu, penerbitan halaman, dan penyimpanan paket **`.pubgen`**.

---

### Tutorial 1: Menyiapkan Konten Publikasi di Excel

Orchestrator tidak mengetik isi publikasi; ia membaca satu buku kerja **`.xlsx`**. Satu impor workbook menjadi satu publikasi, dan hampir setiap sheet menjadi satu kartu halaman. Karena itu, rapikan dulu isi Excel supaya impor berjalan sekali jadi.

#### 1. Siapkan satu folder kerja

Letakkan buku kerja beserta semua berkas pendukungnya dalam satu folder, misalnya:

```
KCDA_KARAWANG_TIMUR_2026/
├── KCDA_KARAWANG_TIMUR_2026.xlsx
├── images/        city_skyline.jpg, bps_logo.png, infografik_01.png …
└── rtf/           R01.rtf, B01.rtf, A02.rtf …
```

- Nama berkas workbook bebas. Yang menentukan isi adalah **nama sheet di dalamnya**.
- Semua path di dalam sel (foto, logo, RTF, GeoJSON) ditulis relatif terhadap folder workbook, misalnya `images/bps_logo.png`. Pencarian berkas juga mencoba folder workbook itu sendiri; naskah teks juga dicari di `rtf/`, `docx/`, dan `md/`, sedangkan gambar dan GeoJSON juga di `images/`, `assets/`, dan `img/`.
- Hindari path absolut seperti `F:\GitHub\...`: path itu hanya berlaku di komputer tersebut dan membuat workbook tidak portabel. Buku kerja contoh di repositori (`plan/pubgen/samples/DUAL_LANG/KCDA_KARTIM_2026.xlsx`) masih memuat beberapa path absolut dari komputer penyusunnya.

#### 2. Tentukan sheet apa saja yang dipakai

Nama sheet adalah kontrak antara Excel dan Orchestrator. Gunakan pola berikut:

| Nama sheet | Menjadi kartu | Isi yang ditulis |
|---|---|---|
| `COVER`, `COV`, `SAMPUL` | **Sampul** | Pasangan kunci–nilai: judul, tahun, katalog, logo, foto. |
| `A02` (juga `A01`–`A99`) | **Paragraf** catatan publikasi | `title` dan `file` RTF (`Keterangan Publikasi`, redaksi, atau hak cipta). |
| `B01`, `B02`, … | **Paragraf** bagian awal | `title`, `title_en`, `file`, `file_en`, dan gambar `image1`/`image2`. |
| `G00`, `G01`, … | **Paragraf** narasi | `file` RTF (Kata Pengantar, Penjelasan Teknis, dan sejenisnya). |
| `R01`–`R99` | **Paragraf** ulasan bab | `title`, `file` (Indonesia), `file_en` (Inggris), opsional pembatas bab. |
| `XX_XX`, `XX_XX_XX` | **Tabel** | Judul tabel, baris header, data, baris `Sumber`/`Catatan`. |
| `CXX_XX` | **Grafik** | Blok `chart_type` dan data grafik (§8). |
| `MXX_XX` | **Peta** | `type: choropleth`, `map` atau `geojson`, dan `data`. |
| `DIV` | **Pembatas** bab | `title`, `image`/`file`, dan `template`. |
| `TCP`, `TCT`, `TCF` | **Daftar isi** | Paragraf, tabel, atau grafik; lihat langkah 7. |
| `NOTE`, `CATATAN` | *(diabaikan)* | Lembar bantu/legenda; tidak menjadi kartu. |

Pemetaan di atas berlaku untuk profil **BPS Daerah Dalam Angka** (profil bawaan). Profil lain merutekan sheet menurut aturannya sendiri (§12); BRS dipilih dari nama sheet yang diawali `brs`, memuat `ringkasan`/`utama`, atau bernama `POINT`. Periksa profil aktif sebelum menamai sheet.

Aturan penamaan yang perlu diikuti:

- Gunakan angka dua digit dan huruf besar: `01_01`, `02_03`, `R01`. Nama seperti `Tabel 1` atau `Review01` tidak dibaca sebagai halaman yang dimaksud dan bisa jatuh menjadi kartu Tabel.
- `03_01L` adalah **lanjutan** tabel `03_01`. PGS mengenalinya sebagai lanjutan, sehingga tidak didaftarkan lagi di Daftar Tabel; tulis sendiri kata `Lanjutan` pada baris judulnya bila ingin tercetak.
- Sheet `COVER` selalu menjadi kartu pertama; kartu lainnya mengikuti urutan sheet di workbook.

#### 3. Isi sheet `COVER`

Tulis pasangan kunci–nilai di kolom A dan B, satu pasangan per baris:

| Kunci | Contoh nilai |
|---|---|
| `style` | `HONEYCOMB HEXAGON` (nama gaya sampul, §3) |
| `title1` | `KECAMATAN KARAWANG TIMUR` |
| `title2` | `DALAM ANGKA` |
| `year` | `2026` |
| `volume` | `13 2026` |
| `no_catalogue` | `110200.10547` |
| `ISSN` atau `ISBN` | `2828-7436` |
| `office_name` | `BADAN PUSAT STATISTIK\nKABUPATEN KARAWANG` |
| `image_1` | `images/city_skyline.jpg` |
| `image_lanscape` | `images/city_landscape.jpg` |
| `office_logo` | `images/bps_logo.png` |
| `event_theme` | `images/theme1.png` (opsional) |
| `palette` | `Corporate Blue` (opsional, untuk sampul AI Base) |
| `title_publication` | `Kecamatan Karawang Timur Dalam Angka 2026` (opsional, dipakai footer profil BPS General Report) |

`title1` wajib diisi. Kalau sheet ini tidak ada sama sekali, publikasi tetap bisa dibangun, tetapi seluruh isian sampul memakai nilai bawaan.

#### 4. Isi sheet narasi (`A02`, `B01`, `G00`, `R01`)

Naskah panjang tidak ditulis di sel; naskah disimpan sebagai berkas RTF (atau DOCX, yang diubah menjadi RTF saat impor), lalu sel hanya menunjuk berkasnya:

| Baris | Isi |
|---|---|
| `title` | Judul halaman, mis. `Bab 01 Geografi` atau `Keterangan Publikasi`. |
| `title_en` | Judul versi Inggris (opsional). |
| `file` | `rtf/R01.rtf` — badan naskah bahasa Indonesia. |
| `file_en` | `rtf/R01_en.rtf` — badan naskah bahasa Inggris (opsional). |

- Foto pada halaman bagian awal (`image1`, `image2`) ditempatkan oleh alur **Master Excel** di mode DesignCraft (§7); impor Orchestrator membaca judul dan berkas naskahnya saja.
- Untuk `RXX`, tambahkan baris `divider`/`pembatas` (mis. `Batik Infographic`) dan `image_div` bila bab itu perlu halaman pembatas; PGS menyisipkan kartu **Pembatas** tepat sebelum ulasan bab.
- Sheet `A02` yang hanya berisi baris `file` tetap menjadi halaman catatan, tetapi tanpa judul halaman. Tambahkan baris `title` agar judulnya tercetak.
- Teks yang ditulis langsung di sel (bukan RTF) tetap dipakai sebagai paragraf cadangan, tetapi kehilangan perataan, indentasi, dan spasi paragraf dari Word.

#### 5. Isi sheet tabel (`XX_XX`)

Susun satu tabel per sheet dengan urutan berikut:

1. **(Opsional) baris metadata** di paling atas: `title` (judul bab), `title_en`, `subtitle` (judul subbab), `subtitle_en`, `template`, dan `theme`. Baris ini menjadi kepala bab/subbab, bukan isi tabel.
2. **Baris judul tabel**, mis. `Tabel 01.01 Luas Daerah Menurut Desa/Kelurahan di Kecamatan Karawang Timur, 2025`. Baris ini menjadi judul kartu dan judul halaman; nomornya juga dipakai di Daftar Tabel.
3. **Baris header kolom**. Header bertingkat boleh memakai sel gabungan (*merge*), atau ditulis langsung dengan pemisah `--`:
   `Jenis Kelamin--Laki-Laki`, `Jenis Kelamin--Perempuan`, `Jumlah`.
4. **Baris data**.
5. **Baris sumber dan catatan** di bawah data: tulis label di kolom A, titik dua `:` di kolom B, dan isinya di kolom C — mis. `Sumber` | `:` | `Badan Pusat Statistik, Pendataan Potensi Desa (Podes)`. Baris `Sumber`, `Catatan`, `Keterangan`, `Note`, dan `Source` dikeluarkan dari kisi tabel dan dicetak di bawah tabel dengan gaya yang sesuai.

Beberapa hal yang menentukan hasil:

- Baris kosong dilewati; kolom kosong di ujung kanan tidak dihitung.
- Gaya garis, baris nomor kolom `(1), (2), …`, dan penanda baris total mengikuti profil aktif (§12) — tidak perlu digambar manual.
- Untuk tabel yang terpotong, buat sheet lanjutan `03_01L` dengan header dan kolom yang sama.

#### 6. Isi sheet grafik `CXX_XX` dan peta `MXX_XX`

Sheet grafik adalah blok kunci–nilai. Hanya `chart_type` yang wajib, dan `data` menunjuk sheet tabel yang menjadi sumber angkanya:

```
chart_type        bar_clustered
title             Pertumbuhan Ekonomi Menurut Sektor
title_en          Economic Growth by Sector
data              03_01L
value_format      percent
show_value_labels true
```

- Daftar arketipe (`bar_clustered`, `horizontal_bar`, `line`, `donut`, `population_pyramid`, `gauge`, …) dan kunci lain ada di §8. Tanpa `chart_type`, sheet ditempatkan sebagai gambar biasa.
- Bila `data` menunjuk sheet tabel, kolom kategori dan seri dideteksi otomatis dari tabel itu; setel ulang di tab **Opsi Grafik** lewat **Click to edit** bila urutannya tidak sesuai. Bila grafik memuat datanya sendiri di sheet `CXX_XX` (tanpa `data`), tulis juga `category_col`, `series_1_col`, `data_start_row`, dan `data_end_row` seperti contoh di §8.

Sheet peta menulis `type: choropleth` (atau `chart_type: map`) dan wajib menunjuk peta lewat `map` (peta bawaan, mis. `KARAWANG TIMUR`) atau `geojson` (berkas sendiri), ditambah `data` untuk nilai per wilayah (mis. `maps/penduduk.csv`).

#### 7. Isi sheet daftar isi dan pembatas

- `TCP` → **Daftar Isi** (paragraf), `TCT` → **Daftar Tabel**, `TCF` → **Daftar Grafik**.
- Baris yang dikenali: `content_type` | `Table of Contents`, `toc_style` | `TOC Paragraph`/`TOC Table`/`TOC Chart`, `column` | `1 column`/`2 column`, `include_self` | `yes`/`no`, dan `layout` | mis. `TablePage`.
- Sheet `DIV` menjadi halaman pembatas bab. Isi `title` (mis. `BAB 02 PEMERINTAHAN`), `image` bila memakai foto, dan `template` bila memakai templat pembatas tertentu.

#### 8. Periksa sebelum impor

- Buku kerja disimpan sebagai **`.xlsx`** (atau `.xlsm`), bukan `.xls` lama.
- Ada satu sheet `COVER` dengan `title1`.
- Semua nama sheet mengikuti pola di langkah 2; sheet bantu tak terpakai dinamai `NOTE`/`CATATAN` atau dihapus.
- Semua berkas RTF, DOCX, gambar, dan GeoJSON benar-benar ada di folder kerja, dan path di sel cocok dengan lokasinya. Berkas yang tidak ditemukan tidak menggagalkan impor: halaman narasi memakai teks cadangan, dan gambar yang hilang tidak ditempatkan.
- Setiap tabel punya baris judul dan header; baris `Sumber`/`Catatan` memakai format kolom A–B–C di langkah 5.
- Setiap sheet grafik punya `chart_type` (dan `data` bila angkanya diambil dari tabel); setiap peta punya `map`/`geojson` dan `data`.
- Tidak ada path absolut yang menunjuk folder pribadi.

---

### Tutorial 2: Mengimpor Excel ke Orchestrator dan Menyimpan Paket `.pubgen`

Setelah workbook siap, seluruh alur berikut berjalan dari mode Orchestrator:

1. **Buka aplikasi**: jalankan **`pgs.exe`**. Beranda **Orchestrator** terbuka.
2. **Impor workbook**:
   - Klik **Import Manifest (XLSX)** di beranda, atau pilih **File ▸ Import Excel Workbook (.xlsx)…**.
   - Pilih berkas `.xlsx`/`.xlsm` hasil Tutorial 1. PGS membuka **tab workspace baru**; publikasi yang sedang terbuka tidak tertimpa.
   - PGS membaca setiap sheet, menyalin tabel, naskah RTF, dan aset gambar ke workspace, lalu menyusun **kartu halaman**. Hasil impor dan struktur kartu tercatat di tab **Execution Log**.
3. **Periksa profil**: PGS memilih profil dari nama sheet — BRS, laporan korporat, monograf akademik, atau BPS DDA sebagai bawaan. Profil bisa diganti di beranda atau di kombo **Publication Profile** pada dock **Profile & Execution** (§12).
4. **Periksa kartu**:
   - Tab **Page Structure** menampilkan **Publication Content List** — Sampul, Paragraf, Grafik, Peta, Tabel, Gambar, Daftar Isi, Pembatas.
   - Klik **Click to edit** untuk membetulkan judul, sumber, tata letak seksi, atau header & footer satu kartu. Tab **Visual Live Preview** menampilkan pratinjau halaman.
5. **Terbitkan**:
   - Di dock **Profile & Execution**, tekan **Generate Publication** (membangun halaman) atau **Generate & Publish PDF** (membangun halaman sekaligus mengekspor PDF rilis).
   - Bila kanvas sudah pernah diedit manual, **Compilation Safety Guard** muncul lebih dulu. Pilih **Save as New Version (v2)** untuk menyimpan tata letak saat ini sebagai versi baru, **Overwrite Layout** untuk membangun ulang, atau **Cancel**.
6. **Simpan paket `.pubgen`**:
   - Pilih **File ▸ Save Publication (.pubgen)** (`Ctrl+Shift+S`) atau **Save Publication As (.pubgen)…**.
   - Penyimpanan pertama meminta lokasi berkas. Paket adalah satu berkas ZIP portabel yang memuat `metadata.json`, `pubgen.json`, `glossary.json`, memori terjemahan, tabel (`csv/`), teks (`text/rtf/`), aset (`assets/`), dan templat bila ada. Sumber Excel aslinya tidak ikut diubah.
   - Simpan berikutnya menimpa paket yang sama, sehingga proyek bisa ditutup dan dibuka kembali persis di titik terakhir.
7. **Buka lagi nanti**: dari beranda klik **Open file (.pubgen)** atau pilih **File ▸ Open Publication (.pubgen)…**. Paket muncul juga di **Recent Opened Documents**.
8. **Bolak-balik data dengan Excel**: **File ▸ Export Excel Workbook (.xlsx)…** menulis workspace kembali menjadi workbook. Sunting di Excel, lalu impor ulang seperti langkah 2.

---

### Tutorial 3: Menyusun Dokumen dari Nol (Mode DesignCraft)

Alur manual ini berguna bila Anda ingin membangun dokumen kosong lalu memakai dialog generator satu per satu:

1. **Membuat Dokumen Baru**:
   - Tekan tombol pintas `Ctrl+N` (**File ▸ New Document** di mode DesignCraft). Pilih ukuran `A4`, centang *Facing Pages*.
2. **Membuat Sampul Depan**:
   - Buka **Publication ▸ Publication Cover…**.
   - Pilih gaya **`⬡ Hexagon`** (atau arahkan kursor ke pil gaya untuk melihat pratinjau).
   - Isi judul: `KARAWANG TIMUR` dan tahun: `2026`.
   - Klik **Generate Cover Page**. Sampul langsung jadi di Halaman 1!
3. **Membuat Halaman Judul Bab**:
   - Buka **Publication Tools ▸ Manual Ingest & Tables ▸ Page from CSV…**.
   - Klik **Browse...** lalu pilih berkas contoh `samples/KBDA2026_chapters.csv`.
   - Pilih preset: `1 Row · 3 Columns`.
   - Klik **Generate Pages**. Sebanyak 28 halaman judul bab otomatis terbentuk!
4. **Memasukkan Tabel Data**:
   - Pindah ke Halaman 2 dokumen.
   - Buka **Publication Tools ▸ Manual Ingest & Tables ▸ Table from CSV…**.
   - Klik **Browse...** lalu pilih berkas `samples/kbda_2005.csv`.
   - Pilih gaya: `Horizontal Rules (Booktabs)`. Penempatan: `At page (below title)`.
   - Centang opsi `[x] Condition rules (Jumlah, Tahun, Kab, Kec...)`.
   - Klik **Insert Table**. Tabel statistik langsung terpasang rapi di bawah judul!
5. **Membuat Daftar Isi**:
   - Di **Publication Tools ▸ Page & Layout Utilities**, buka kartu utilitas halaman lalu klik **Add Blank Page**.
   - Buka **Publication ▸ Table of Contents…**, lalu pilih TOC style: `TOC Table` (kolom Judul otomatis terisi `Daftar Table`), pilih
     Layout: `TablePage`.
   - Klik **Add TOC**. Daftar isi langsung tercipta lengkap dengan nomor halamannya! Untuk daftar grafik, pindah ke
     halaman berikutnya lalu pilih TOC style: `TOC Chart` dan klik **Add TOC** lagi.
6. **Mengekspor ke PDF**:
   - Buka **File ▸ Output ▸ Export PDF…** (mode Orchestrator) atau **File ▸ Export PDF…** (mode DesignCraft). Publikasi Anda siap untuk dicetak atau disebarluaskan secara digital!

---

### Tutorial 4: Mendesain Sampul Kustom dengan Foto & Warna Sendiri

Ingin menyesuaikan sampul dengan identitas visual daerah Anda?

1. Pada dialog **Publication Cover**:
   - Pilih gaya sampul, misalnya **`❖ Medallion`** atau **`⬡ Hexagon`**.
2. **Menyesuaikan Warna**:
   - Masukkan kode warna resmi daerah Anda, misalnya:
     - Primary: `#1e3a8a` (Biru Laut)
     - Accent: `#10b981` (Hijau Zamrud)
     - Dark: `#0f172a` (Biru Gelap Arang)
3. **Memilih Foto Sendiri**:
   - Pada bagian **Cover Images**, klik tombol **Browse...** di samping *Landscape* atau *Honeycomb*.
   - Pilih foto pemandangan terbaik dari komputer Anda (format `.jpg` atau `.png`).
4. **Memasang Lambang Daerah / Logo**:
   - Klik **Browse...** pada kotak *Logo* lalu pilih berkas lambang daerah berlatar transparan.
   - Ketikkan identitas lembaga pada isian *Logo Text*.
5. Klik **Generate Cover Page**. Sampul eksklusif Anda langsung selesai!

---

### Tutorial 5: Memformat Tabel Statistik Kompleks dari Excel

Jika Anda memiliki tabel dengan kelompok kolom bersusun (misalnya: Jumlah Penduduk menurut Kelompok Umur dan Jenis Kelamin):

1. Pada lembar kerja Excel, beri nama kepala kolom menggunakan tanda hubung ganda (`--`):
   | Kecamatan | Usia 0-14--Laki-Laki | Usia 0-14--Perempuan | Usia 15-64--Laki-Laki | Usia 15-64--Perempuan | Total |
2. Simpan lembar kerja tersebut sebagai berkas `.xlsx` atau `.csv`.
3. Di DesignCraft, buka dialog **Table from CSV / Excel**:
   - Klik **Browse...** lalu pilih berkas Anda.
   - Centang `[x] First row is header`.
   - Centang `[x] Add column numbers (1), (2)...`.
   - Centang `[x] Table footer (last row)`.
   - Centang `[x] Condition rules (Jumlah, Tahun, Kab, Kec...)`.
   - Pilih gaya `Horizontal Rules (Booktabs)`.
4. Klik **Insert Table**.
5. Hasilnya: Tabel bertingkat rapi dengan penggabungan sel otomatis, baris penomoran kolom resmi `(1), (2), (3)...`, serta garis pembatas khusus pada baris total!

---

### Tutorial 6: Mengompilasi Seluruh Publikasi dari Satu Manifest

Manifest/compiler adalah jalur multi-bab. Di Orchestrator, padanannya adalah satu paket **`.pubgen`** dengan satu daftar kartu per bab; gunakan tutorial ini bila Anda ingin mendeskripsikan banyak buku kerja bab sekaligus.

Untuk buku tahunan dengan delapan sampai dua belas buku kerja bab, compiler menggantikan perulangan manual:

1. **Kumpulkan buku kerja** dalam satu folder, misalnya `workbooks/BAB_01_2025.xlsx` … `workbooks/BAB_10_2025.xlsx`, dengan sheet `COVER` di bab 1 atau di buku kerja tersendiri.
2. **Pilih profil** di beranda atau combo **Publication Profile** pada dock **Profile & Execution** — `BPS Daerah Dalam Angka` untuk buku tahunan KDA/KCDA (§12). Nilai `"profile"` di manifest menang bila diisi.
3. **Buat manifest**: di kartu **Publication Compiler** (mode DesignCraft) tekan **New…**, isi judul, daftar bab, margin, dan apakah akan mengekspor PDF, lalu tentukan lokasi penyimpanan `pubgen_project.json`. Skema lengkapnya ada di §10.
4. **Compile**: tekan **Compile** atau **Publication ▸ Compile Publication** (`Ctrl+Shift+B`). Bilah progres menyebut setiap langkah — dokumen baru, halaman induk, sampul depan, sampul dalam, bagian awal, placeholder daftar, tiap bab, sampul belakang, daftar isi, penomoran halaman, running header, simpan, PDF. **Cancel** berhenti dengan rapi dan tidak menyentuh hasil build sebelumnya.
5. **Periksa**: baca baris status (halaman, bab, peringatan), lalu jalankan **Audit Publication** (§11) dan klik baris yang ditandai untuk langsung melompat ke sana.
6. **Ekspor**: dengan `"exportPdf": true` PDF ditulis di samping dokumen; bila tidak, gunakan **File ▸ Output ▸ Export PDF…** (mode Orchestrator) atau **File ▸ Export PDF…** (mode DesignCraft).
