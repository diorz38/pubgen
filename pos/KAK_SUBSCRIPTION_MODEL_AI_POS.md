# KERANGKA ACUAN KERJA (KAK)
## PENGADAAN LANGGANAN LAYANAN MODEL ARTIFICIAL INTELLIGENCE (AI) UNTUK OPTIMALISASI APLIKASI PUBLICATION ORCHESTRATOR STUDIO (POS)
### BADAN PUSAT STATISTIK KABUPATEN KARAWANG TAHUN ANGGARAN 2026/2027

---

### I. LATAR BELAKANG DAN PENDAHULUAN

Badan Pusat Statistik (BPS) Kabupaten Karawang memegang peranan strategis sebagai penyedia data statistik rujukan utama (*the agent of trust*) bagi perencanaan, pemantauan, dan evaluasi pembangunan daerah. Dalam menjalankan tugas pokok dan fungsinya, BPS Kabupaten Karawang memiliki kewajiban menerbitkan sekurang-kurangnya 36 jenis publikasi berkala setiap tahun. Publikasi ini mencakup publikasi induk seperti *Kabupaten Karawang Dalam Angka (DDA)*, 30 judul publikasi *Kecamatan Dalam Angka*, *Statistik Daerah*, publikasi tematik lintas sektor, hingga *Berita Resmi Statistik (BRS)*. Bersamaan dengan meningkatnya kebutuhan pemangku kepentingan daerah terhadap ketersediaan data yang cepat dan akurat, BPS menghadapi tuntutan peningkatan kualitas penyajian publikasi, percepatan jadwal rilis, serta standardisasi format dwibahasa (*bilingual*) Indonesia–Inggris yang presisi.

Untuk menjawab kebutuhan tersebut, BPS Kabupaten Karawang mengembangkan aplikasi *Publication Orchestrator Studio* (POS), sebuah sistem otomasi penyusunan dan penerbitan publikasi statistik. Namun, dalam implementasinya saat ini, modul kecerdasan buatan pada aplikasi POS masih mengandalkan layanan tanpa biaya (*free tier*), yaitu API gratis Groq untuk penerjemahan bahasa dan *Free Worker AI* dari Cloudflare. Ketergantungan pada tier gratis ini memicu hambatan operasional yang nyata, seperti pembatasan kuota permintaan (*rate limit* TPM dan RPD) yang sangat ketat, latensi respon yang tidak stabil, keterbatasan memori konteks naskah, serta ketiadaan jaminan ketersediaan layanan (*uptime SLA*). Kendala ini kerap memicu kegagalan pemrosesan (*timeout*) di tengah jalan saat menyusun publikasi tebal. Oleh sebab itu, alokasi anggaran untuk berlangganan penyedia (*provider*) Model AI berbayar menjadi kebutuhan mendesak guna menjaga keberlangsungan penerbitan seluruh publikasi resmi secara optimal.

---

### II. MAKSUD DAN TUJUAN

1. **Maksud:** Melakukan pengadaan langganan antarmuka pemrograman aplikasi (*API Subscription*) dari penyedia Model Kecerdasan Buatan (AI) profesional untuk memperkuat sistem otomasi publikasi BPS Kabupaten Karawang.
2. **Tujuan:**
   - Menyediakan akses komputasi Model AI yang andal, cepat, dan berkapasitas besar untuk menggantikan keterbatasan akun gratis.
   - Mengoptimalkan fungsi fitur otomasi narasi, penerjemahan dwibahasa, dan penataan tata letak pada aplikasi POS.
   - Menjamin mutu dan ketepatan jadwal terbit bagi 36 jenis publikasi tahunan BPS Kabupaten Karawang.

---

### III. MANFAAT BAGI KANTOR BPS KABUPATEN KARAWANG DAN PEGAWAI SECARA UMUM

Peralihan menuju langganan Model AI komersial memberikan dampak positif yang luas bagi institusi BPS Kabupaten Karawang dan para pegawai:

1. **Peningkatan Efisiensi dan Produktivitas Kerja:**
   Pegawai fungsional statistisi dan pranata komputer terbebas dari beban kerja administratif yang repetitif, seperti penerjemahan manual ratusan lembar tabel statistik. Waktu penyusunan naskah dapat dipersingkat hingga lebih dari separuh waktu normal, sehingga jam kerja pegawai dapat dialihkan untuk kegiatan analisis data mendalam serta penjaminan kualitas angka statistik.
2. **Standardisasi dan Akurasi Narasi Statistik:**
   Model AI berbayar memiliki kemampuan penalaran kontekstual yang jauh lebih matang dibanding model gratis berukuran kecil. Hal ini meminimalkan risiko salah tafsir angka, menjaga konsistensi gaya bahasa, serta mencegah halusinasi data saat menyusun ulasan indikator strategis, seperti kemiskinan, inflasi, ketenagakerjaan, dan pertumbuhan ekonomi daerah.
3. **Modernisasi Lingkungan Kerja Birokrasi:**
   Pemanfaatan kecerdasan buatan tingkat lanjut mendorong terciptanya budaya kerja modern berbasis teknologi (*smart governance*). Pegawai terbiasa memanfaatkan teknologi mutakhir secara etis dan produktif, yang secara langsung meningkatkan indeks profesionalitas aparatur sipil negara di lingkungan BPS Kabupaten Karawang.
4. **Penguatan Reputasi Lembaga:**
   Publikasi yang tersaji rapi, minim kesalahan ketik, taat terminologi baku, dan terbit tepat waktu memperkuat posisi BPS di mata pemerintah daerah, akademisi, dan masyarakat sebagai lembaga penyedia data yang kredibel dan tepercaya.

---

### IV. MANFAAT KHUSUS DALAM OPTIMALISASI APLIKASI POS (PUBLICATION ORCHESTRATOR STUDIO)

Secara teknis, ketersediaan langganan Model AI berkinerja tinggi akan memaksimalkan kinerja modul-modul utama aplikasi POS:

1. **Penerjemahan Dwibahasa Berbasis Glosarium Resmi (*BPS Glossary Compliance*):**
   Aplikasi POS membutuhkan model AI yang mampu memproses instruksi sistem (*system prompt*) berukuran panjang untuk mengunci istilah teknis statistik resmi BPS. Model berbayar menjamin penerjemahan judul tabel, bab ulasan, dan *running footer* dwibahasa pada publikasi DDA berjalan presisi tanpa terpotong batas token.
2. **Efektivitas Siklus *Copyfitting* dan Deteksi *Overset* Teks:**
   Aplikasi POS terhubung dengan *layout interpreter* yang memerlukan penyesuaian panjang karakter (*character budget*) agar teks tidak meluap (*overset/overflow*) dari batas bingkai tata letak. Model AI berbayar dapat melakukan parafrase cerdas secara cepat guna mempertahankan keindahan tipografi tanpa mengubah makna angka.
3. **Ketahanan Pemrosesan Massal (*High-Throughput Batch Processing*):**
   Tier berbayar menyediakan batas *Tokens Per Minute* (TPM) yang lapang, memungkinkan aplikasi POS mengekstrak puluhan lembar kerja Excel (`.xlsx`) dan menghasilkan publikasi 30 kecamatan sekaligus tanpa terhenti oleh galat batas kuota (*HTTP 429 Too Many Requests*).

---

### V. RUANG LINGKUP DAN SPESIFIKASI KEBUTUHAN TEKNIS

1. **Ruang Lingkup:** Langganan API Model Kecerdasan Buatan (seperti OpenAI API, Anthropic Claude API, atau Groq Developer Tier) dengan status akun komersial aktif.
2. **Spesifikasi Minimal:**
   - Jendela konteks minimal 32.000 hingga 128.000 token per *request*.
   - Kuota pemrosesan minimal 200.000 *Tokens Per Minute* (TPM) dengan dukungan akses konkuren.
   - Waktu tanggap (*latency*) rendah dan ketersediaan layanan (*uptime*) minimal 99,5%.
   - Jaminan privasi bahwa data masukan statistik tidak dijadikan bahan pelatihan model publik.

---

### VI. PERKIRAAN BIAYA DAN SUMBER ANGGARAN

Kebutuhan anggaran untuk pengadaan langganan layanan Model AI ini diusulkan melalui DIPA BPS Kabupaten Karawang dengan rincian:

- **Alokasi Anggaran:** Rp 5.000.000,- sampai dengan Rp 10.000.000,- (Lima Juta hingga Sepuluh Juta Rupiah) per tahun.
- **Mekanisme Belanja:** Pembelian saldo kredit penggunaan API (*usage-based billing*) atau pembayaran paket langganan tahunan resmi.

---

### VII. KELUARAN (OUTPUT) DAN PENUTUP

Keluaran dari kegiatan ini adalah ketersediaan kunci API Model AI berbayar yang terintegrasi langsung pada aplikasi POS, tuntasnya 36 jenis publikasi BPS Kabupaten Karawang tepat waktu dengan mutu penyajian berstandar tinggi, serta meningkatnya efisiensi kerja pegawai. KAK ini disusun sebagai dasar pertimbangan teknis dan administrasi pelaksanaan anggaran.
