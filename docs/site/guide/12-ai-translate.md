## 13. Terjemahan AI & Memori Terminologi

Pubgen menerjemahkan publikasi berbahasa Indonesia ke bahasa Inggris dengan model chat yang kompatibel dengan OpenAI pilihan Anda, dan menyimpan setiap keputusan di dalam paket proyek. Terjemahan disiapkan **sebelum** kompilasi: tahap **Translate Worksheet** di Orchestrator mengisi teks Inggris dan Anda mengulasnya. Halaman yang dihasilkan dari lembar kerja Orchestrator (**Generate Publication**) lalu merender teks Inggris itu — judul, keterangan, catatan, sel tabel yang berpasangan, dan badan narasi Inggris. Jalur kompilasi manifest/`pgs-cli compile` tidak membaca memori terjemahan (lihat batasan). Tindakan per halaman di dalam dokumen tetap tersedia untuk membetulkan satu halaman secara manual.

Semua di bawah ini memerlukan build daring (`--features pubgen,ai-translate-online`; aplikasi Studio mengaktifkannya) dan penyedia yang sudah dikonfigurasi. Model NLLB luring tidak dapat berjalan di build ini — baris **Offline on this computer** di dialog penyiapan menyatakannya.

### Penyiapan

Buka **Publication Tools ▸ AI Translate ▸ Setup…**, atau tekan **AI Translator** di header panel Pubgen. Dialog **Translate with AI — setup** punya dua tab, **Setup** dan **Terminology**. Pilih salah satu dari empat baris, isi kolomnya, lalu tekan **Use AI**: satu permintaan kecil yang benar-benar berjalan membuktikan endpoint dan id model, setelah itu pengaturan disimpan dan penyedia tersebut siap dipakai.

- **Online — Groq (free account)** — kunci gratis dari [console.groq.com/keys](https://console.groq.com/keys). **Refresh models** mengisi daftar dari akun Anda (endpoint speech-to-text, text-to-speech, embedding, dan safety disembunyikan); bawaannya `llama-3.3-70b-versatile`. Teks katalog dikirim ke cloud Groq, dan batas paket gratis membuat pekerjaan besar berjalan lebih lama, bukan gagal — 16 unit per permintaan, hingga tiga percobaan ulang setelah `429`.
- **Custom API (OpenAI-compatible)** — endpoint chat apa pun: base URL (skema `http://` atau `https://` wajib dan tidak pernah ditebak), kunci opsional, dan model. **Refresh models** membaca `/models` endpoint tersebut.
- **Local AI (Ollama, LM Studio…)** — server `/v1` lokal, bawaannya `http://localhost:11434/v1` (Ollama); LM Studio dan llama.cpp bekerja dengan cara yang sama. Tanpa kunci, dan tidak ada yang keluar dari komputer.
- **Offline on this computer** — hanya keterangan: berkas NLLB-200 belum disambungkan ke build ini, jadi baris ini tidak bisa menerjemahkan. Gunakan salah satu dari tiga yang di atas.

Tempat pengaturannya: di DesignCraft berupa preferensi penyedia di `prefs.json` di folder pengaturan (`DesignCraftData` untuk instalasi portabel). PubGen Studio menyimpan salinannya sendiri di `settings.json` di bawah `%APPDATA%\pubgen-studio` (Windows), `$XDG_CONFIG_HOME/pubgen-studio` atau `~/.config/pubgen-studio`, dan memuatnya saat mulai. Kunci tersimpan apa adanya di berkas itu; kunci tidak pernah ditulis ke dokumen, paket `.pubgen`, memori terjemahan, baris log, atau pesan kesalahan.

### Lembar Kerja Terjemahan (sebelum kompilasi)

Di dock kanan Orchestrator, tab **Profil & Eksekusi** (antarmuka Inggris: **Profile & Execution**), ada tombol **Translate Worksheet** di bawah kotak profil publikasi. Tombol ini diredupkan sampai ada penyedia yang dikonfigurasi dan lembar kerja berisi minimal satu kartu, tetapi tetap bisa ditekan: bila ada kartu dan belum ada penyedia, tahapnya tetap menyelesaikan kecocokan memori dan glosarium secara lokal, dan sisa yang belum dikenal melaporkan bahwa belum ada penyedia yang dikonfigurasi. Setelah penyedia dikonfigurasi, satu tekan memulai pekerjaan latar belakang:

1. Kumpulkan unit yang dapat diterjemahkan dari setiap kartu (tabel di bawah).
2. Selesaikan apa yang bisa diselesaikan secara lokal: entri memori terjemahan (asal mana pun), atau istilah glosarium resmi yang cocok utuh. Unit-unit itu dihitung sebagai digunakan ulang (*reused*) dan tidak memakan permintaan; kecocokan glosarium juga masuk ke memori sebagai *Imported*.
3. Samarkan sisanya — istilah glosarium dan angka menjadi placeholder — lalu kirim hanya yang belum dikenal ke penyedia, 16 unit per permintaan.
4. Pulihkan placeholder, terapkan glosarium resmi, dan jalankan sensus angka (di bawah). Unit yang gagal dihitung sebagai gagal, dan tidak ada yang ditulis untuknya.
5. Simpan setiap target yang diterima di memori sebagai *AI draft*, dan tulis bahasa Inggris bertipe judul ke kartunya sebagai **Judul EN**.
6. Catat satu baris di LogEksekusi: translated / reused / skipped / failed.

Tombol menampilkan progres (`Menerjemahkan dengan AI… 12/84`, berbahasa Indonesia di kedua bahasa antarmuka) dan bilah status mengulang totalnya. Bila semua unit sudah dikenal, tidak ada permintaan yang keluar dari mesin. Tekan **Generate Publication** (atau **Generate & Publish PDF**) sesudahnya: generate dari lembar kerja merender teks Inggris yang tersimpan pada halaman baru.

### Yang diterjemahkan (per jenis kartu)

| Kartu | Dikirim ke model | Bahasa Inggris disimpan di | Dirender oleh Generate |
|---|---|---|---|
| **Cover** | judul publikasi dan baris kedua sampul | **Judul EN** kartu (data sampul) | baris judul Inggris sampul |
| **Narrative** | judul, dan setiap paragraf tidak kosong dari badan RTF yang disiapkan | **Judul EN**; satu entri memori per paragraf | judul bab Inggris; badannya dibangun ulang dari memori (atau RTF `file_en` bila kartu menyebutkannya) |
| **Divider** | judul (nomornya disamarkan, tidak diterjemahkan) | **Judul EN** | judul Inggris dari templat pembatas |
| **Table** | judul; setiap sel baris pertama (masing-masing dipecah pada `--`); sel pertama setiap baris berikutnya — label baris, atau baris catatan/sumber yang terdeteksi otomatis | **Judul EN**; satu entri memori per header, label, dan catatan | judul dan baris catatan/sumber, serta sel header dan baris yang berpasangan; tabel langsung BRS tetap satu bahasa (lihat batasan) |
| **Chart** | judul dan label seri | **Judul EN**; memori | judul grafik Inggris |
| **MapChart** | judul; kolom `title` dari sheet peta | **Judul EN**; memori | judul peta — kolom `title_en` di sheet peta tetap menang |
| **Image** | keterangan, atau judul bila tidak ada keterangan | **Judul EN** | keterangan di bawah grafik |
| **Toc** | hanya judul daftar itu sendiri; entri mewarisi bahasa Inggris kartunya dan tidak pernah diterjemahkan sendiri | **Judul EN** | judul Inggris daftar |
| **Header/Footer** | belum ada: `UnitRole::HeaderFooter` dideklarasikan tetapi tidak ada jenis kartu yang mengumpulkan baris tersebut | — | `{title_en}` di templat header/footer mencetak judul publikasi yang diterjemahkan |

**Tidak pernah diterjemahkan** — unit tidak dikirim bila:

- kosong atau hanya spasi;
- angka sendirian (`1.234,56`, `(1)`, `2026`, `12,5%`, `-`);
- pengenal — ISSN/ISBN, kode katalog (`Katalog BPS: …`), URL, alamat e-mail, jalur yang berakhiran ekstensi berkas yang dikenal;
- nama diri yang dilindungi — nama wilayah GeoJSON pada kartu peta (`nmdesa`/`nmkec`), serta nama instansi, kantor, dan kontak pada sampul;
- sudah dwibahasa (mengandung ` / ` atau pemisah baris paksa) atau sudah dikenal (teks yang sama hanya dikumpulkan sekali).

Baris header dan footer juga tidak dikumpulkan: `UnitRole::HeaderFooter` ada, tetapi tidak ada jenis kartu yang menghasilkannya, sehingga teks header/footer tetap berbahasa Indonesia; token `{title_en}` tetap mencetak judul yang diterjemahkan.

Metadata sampul tidak pernah dikumpulkan sama sekali: tahun, volume, katalog, ISSN/ISBN, nomor/tanggal/bulan rilis, instansi, alamat kantor, serta nama, telepon, e-mail, dan foto PIC. Nilai sumbu dan label data grafik, sel angka tabel, nomor halaman daftar isi, serta batas klasifikasi dan angka legenda peta digambar dari angka, bukan teks. Angka di dalam unit yang diterjemahkan (judul, catatan) disamarkan dan kembali identik byte demi byte.

### Gerbang ulasan

Tab ketiga di dock kanan adalah **Ulasan Terjemahan** (antarmuka Inggris: **Translation Review**). Tab ini menampilkan satu baris per draf AI yang belum diulas: sumber Indonesia dalam huruf tebal, kolom Inggris yang dapat diedit, tombol **Accept** dan **Reject**.

- **Accept** menyimpan target dan menandai entri sebagai *Reviewed*. Bila Anda mengedit kolomnya, teks hasil editan menggantikan draf dan juga ditandai *Reviewed*.
- **Reject** menghapus entri, sehingga proses berikutnya dapat menerjemahkan teks itu lagi.
- Kedua tindakan menandai proyek berubah; simpan paket untuk mempertahankannya.

Editor kartu menampilkan status di samping **Judul EN:** (antarmuka Inggris: **Title EN:**; kolom ini hanya muncul di dokumen dwibahasa atau Inggris) — lencana **AI draft** dengan tombol **Accept** tersendiri, atau **Reviewed** (label ini berbahasa Inggris di kedua bahasa antarmuka). Entri *Reviewed* tidak pernah ditimpa oleh proses AI berikutnya: target baru dari model dibuang dan yang sudah diulas dipertahankan. Kecocokan glosarium tidak menampilkan lencana — hasilnya deterministik, bukan draf.

### Memori terjemahan

Setiap unit yang diterima disimpan di `translation_memory.json` di dalam paket `.pubgen`: sumber, target, asal, peran, dan waktu. Di berkas, asalnya ditulis `ai` (draf), `reviewed` (keputusan manusia), dan `imported` (kecocokan glosarium utuh).

- Pencarian mengabaikan spasi berlebih dan beralih ke padanan huruf besar-kecil; entri *Reviewed* atau *Imported* menang atas draf AI dengan sumber yang sama.
- Memori dibatasi 100.000 entri per paket. Melewati batas itu unit baru tidak disimpan; entri lama masih bisa diganti.
- Paket yang ditulis sebelum memori ada akan terbuka dengan memori kosong.
- Membuka kembali proyek memuat memori ke lembar kerja: **Translate Worksheet** memakai ulang setiap unit yang sudah dikenal tanpa satu pun permintaan ke penyedia dan hanya mengirim yang baru atau berubah. Proses kedua biasanya tidak memakan biaya.

### Angka, penyamaran, dan glosarium

- Istilah glosarium dan angka diganti dengan placeholder `[[n]]` sebelum permintaan (paling banyak 512 per unit) dan dipulihkan sesudahnya. Pemulihan bersifat ketat: placeholder yang hilang, ganda, berubah urutan, atau dibuat-buat menolak unit tersebut alih-alih merusak sebuah statistik.
- Format angka Indonesia tidak pernah mencapai model, sehingga `1.234,56` dan `12,5%` bertahan persis.
- Setelah penyamaran dibuka, setiap deret angka di sumber dibandingkan dengan target (sensus angka). Perbedaan apa pun menolak unit itu: tidak ditulis ke kartu atau memori, dan dihitung sebagai gagal.
- Glosarium resmi diterapkan pada target secara deterministik sebelum sensus, sehingga model yang mengabaikan prompt tidak bisa mengirim istilah tidak baku.

### Menerjemahkan halaman

Tindakan di dalam dokumen tetap ada untuk satu halaman. Mode Dual Lang harus aktif (**Publication Tools ▸ Dual Language Mode**, `Ctrl+Shift+L`) dan penyedia sudah dikonfigurasi. Lalu:

- **Translate this Page** di panel Pubgen berubah menjadi **Translate Table on this Page** pada halaman tabel dan **Translate Review on this Page** pada halaman ulasan `RXX`; **Publication Tools ▸ AI Translate** menawarkan tindakan yang sama plus **Translate Table Pages…** untuk rentang halaman (dialog **Translate Table Pages (Batch)**).
- Halaman ulasan mendapat judul bab dan badan kanannya diganti bahasa Inggris, gaya paragraf dipertahankan. Halaman tabel mendapat judul, sel header, dan kotak sumber/catatan diterjemahkan di tempat, bahasa Inggris di baris baru dengan huruf miring.
- Bilah progres dan **Cancel** ada di baris status header panel; hasilnya di-cache, jadi proses kedua atas baris yang sama berjalan seketika.

### Memori terminologi

Glosarium resmi menjaga keluaran AI tetap memakai istilah statistik baku — *Kecamatan* → *Subdistrict*, *PDRB* → *GRDP*, *Luas Wilayah* → *Total Area*.

- Glosarium bawaan ada di `assets/pubgen/bps_glossary.json`. Urutan pencarian: jalur tersimpan di preferensi, lalu aset, lalu folder build. Berkas hilang atau rusak menjadi glosarium kosong, bukan kesalahan.
- Glosarium disuntikkan ke prompt (maksimal 200 istilah, `id = en`), menutupi istilah utuh dalam satu permintaan, dan diterapkan **secara deterministik pada sel utuh** setelah terjemahan.
- Tab **Terminology** di dialog penyiapan AI adalah editor lengkap: cari, tambah, ubah, hapus, **Import CSV…** (dua kolom — Indonesia, Inggris — dengan koma, titik koma, tab, atau pipa; header opsional), **Reset to bundled**, dan **Save…** ke `bps_glossary.json` milik Anda.
- **Collect Vocabulary** membaca halaman hasil generate (header, label baris, judul, baris catatan dan sumber, serta rangkaian frasa) dan menambahkan istilah Indonesia yang ditemukan dengan kolom Inggris kosong; setiap generate Excel melakukannya otomatis dan melaporkan `Vocabulary: N new term(s) collected`.
- **Fill AI Drafts** mengisi sel Inggris kosong itu dari model, istilah per istilah. Kotak **Auto** di sampingnya mengulang langkah ini setelah setiap generate.
- **Fix Terms** menerapkan glosarium pada terjemahan yang sudah ada di halaman terpilih; **Retranslate (Glossary)** membetulkan istilah tersebut lalu menerjemahkan ulang hanya teks yang ditandai glosarium (satu batch permintaan).

### Terjemahan tanpa antarmuka dengan pgs-cli

```sh
pgs-cli translate <PROJECT.pubgen> --provider <groq|custom|local> --model <MODEL> \
    [--base-url <URL>] [--key-env <VARIABLE>] [--dry-run]
```

- `--provider` adalah `groq`, `custom`, atau `local`; `offline` ditolak.
- `--base-url` opsional untuk `groq` (endpoint Groq menjadi bawaan) dan wajib untuk `custom` dan `local`.
- `--key-env` menamai variabel lingkungan yang menyimpan kunci — wajib untuk `groq` dan `custom`, tidak wajib untuk `local`; bila diberikan, variabelnya tetap dibaca untuk penyedia apa pun, sehingga variabel yang tidak disetel atau kosong menjadi kesalahan. Sengaja tidak ada opsi yang menerima kunci itu sendiri.
- `--dry-run` menghitung unit lalu berhenti: tanpa jaringan, tanpa membaca lingkungan, tanpa menulis paket.
- Laporannya berupa JSON di stdout — `units` (dikumpulkan), `translated` (target AI baru tersimpan), `reused` (kecocokan memori), `skipped` (kecocokan glosarium utuh), dan `failed`, masing-masing menyebut unit dan alasannya.
- Status keluar non-nol bila ada unit yang gagal. Hanya memori yang ditulis balik; pengaturan kartu dan isi paket tidak disentuh, sehingga GUI tetap menampilkan draf baru untuk diulas.

### Batasan yang perlu diketahui

- **Halaman tabel langsung BRS tetap satu bahasa.** Impor sheet BRS merender tabel semacam itu dengan judul dan sel satu bahasa Indonesia; hanya jalur tabel templat (DDA) yang menulis judul tabel dwibahasa dan baris catatan/sumber berpasangan.
- **Sel tabel Inggris dan badan narasi Inggris dirender oleh Generate, bukan oleh kompilasi manifest.** Halaman yang dihasilkan dari lembar kerja Orchestrator memasangkan setiap sel header dan baris dari memori (bahasa Inggris di baris kedua) dan membangun ulang badan narasi Inggris; jalur kompilasi manifest/`pgs-cli compile` tidak membaca memori, jadi merender sel dan badan berbahasa Indonesia. Tidak ada sakelar per halaman terpisah untuk ini: tahap **Translate Worksheet** mengisi memori, dan Generate membacanya otomatis. Anda tetap bisa membetulkan satu halaman tabel dengan tindakan tabel di dalam dokumen, atau menyebut RTF Inggris di `file_en` kartu.
- **Baris header dan footer belum diterjemahkan.** `UnitRole::HeaderFooter` dideklarasikan tetapi tidak ada jenis kartu yang mengumpulkan baris tersebut, sehingga teks header/footer tetap berbahasa Indonesia; `{title_en}` di templat tetap mencetak judul publikasi yang diterjemahkan.
- **Paragraf narasi tanpa bahasa Inggris yang bisa dipakai mempertahankan teks Indonesianya.** Saat badan Inggris dibangun ulang dari memori, paragraf yang targetnya hilang, kosong, atau akan menambah pemisah baris tetap memakai paragraf Indonesia; jumlah paragraf dan setiap gaya paragraf dipertahankan, dan sisa badan tetap terbit.
- **Deteksi judul Inggris membutuhkan gaya paragraf.** Dalam spread narasi, judul dikenali dari gaya paragrafnya (**Title**, **Heading 1**, **Heading 2**). Dokumen Indonesia lama yang paragrafnya tidak memakai gaya tersebut jatuh ke heuristik teks: judul Indonesia yang dikenal, plus paragraf pendek berawalan `A. `, `B. `, atau `C. ` yang dikenali sebagai **Title**.
- **Kompilasi manifest tidak menerjemahkan.** `pubgen.compileProject` dan `pgs-cli compile` mulai dari keadaan generasi baru: keduanya tidak menjalankan tahap lembar kerja dan tidak membaca memori, jadi hanya merender kolom Inggris milik buku kerja itu sendiri dan glosarium. Lakukan Generate dari Orchestrator untuk bahasa Inggris dari AI.
- **TLS hanya memercayai root publik.** Klien HTTP adalah ureq 2 dengan root store Mozilla yang dikompilasi ke dalamnya (webpki-roots); klien tidak membaca certificate store sistem operasi. Endpoint di balik CA privat — gateway internal atau proxy korporat — gagal dengan kesalahan sertifikat. Gunakan Local AI melalui `http://localhost` biasa, atau sertifikat dari CA yang dipercaya publik.
- **Model luring tidak ada di build ini**, apa pun yang dilaporkan panel perangkat keras.

> **Tidak ada di produk:** peringkas narasi *Draft Review* yang lama sudah dihapus pada Oktober 2026. Pubgen tidak menyusun prosa bab dari angka tabel saat ini; pekerjaan AI-nya adalah terjemahan dan terminologi. Bila perlu paragraf ulasan, tulis sendiri lalu tempatkan sebagai RTF.
