---
layout: home

hero:
  name: PubGen Studio
  text: Generator Publikasi Otomatis
  tagline: Aplikasi desktop untuk menyusun publikasi statistik dari Excel, CSV, dan naskah — impor sumber, susun kartu halaman, terbitkan .pubgen + PDF, lalu rapikan halaman apa pun di kanvas DesignCraft.
  actions:
    - theme: brand
      text: Mulai Belajar
      link: /guide/01-pengenalan
    - theme: alt
      text: Tutorial Praktis
      link: /guide/13-tutorial
    - theme: alt
      text: GitHub
      link: https://github.com/diorz38/pubgen

features:
  - icon: 🔀
    title: Mode Ganda — Orchestrator & DesignCraft
    details: Satu aplikasi, dua ruang kerja. Orchestrator menyusun publikasi dari sumber data; DesignCraft membuka kanvas tata letak penuh untuk merapikan tipografi, tabel, dan halaman hasil generate. Beralih lewat pil mode di bilah atas.
  - icon: 🗂️
    title: Alur Kartu Publikasi
    details: Sumber diimpor menjadi kartu halaman — Sampul, Paragraf, Grafik, Peta, Tabel, Gambar, Daftar Isi, Pembatas — di tab Page Structure, lengkap dengan QA Preflight, Visual Live Preview, dan Execution Log.
  - icon: ⚡
    title: Generate Satu Klik
    details: Susun kartu, lalu tekan Generate Publication atau Generate & Publish PDF. Bila kanvas sudah pernah diedit manual, Compilation Safety Guard menawarkan simpan versi baru, timpa, atau batal — tidak ada perubahan yang hilang diam-diam.
  - icon: 📦
    title: Satu Berkas Proyek (.pubgen)
    details: Metadata, sumber CSV/RTF, aset, glosarium, memori terjemahan, dan tata letak hasil kompilasi tersimpan dalam satu paket .pubgen yang portabel dan bisa dibuka kembali persis di titik terakhir.
  - icon: 📗
    title: Sampul Instan
    details: Sampul korporat eksekutif lengkap dengan geometri modern, bayangan lembut, foto terpotong rapi, dan tipografi resmi — depan, dalam, dan belakang, dengan 13 gaya vektor plus sampul AI Base.
  - icon: 📊
    title: Tabel Standar Penerbitan
    details: Format tabel baku BPS — booktabs, header bertingkat, penomoran kolom resmi, penyorotan baris Jumlah/Total otomatis.
  - icon: 📈
    title: Grafik Vektor
    details: Sembilan arketipe grafik asli yang bisa diedit — batang, garis, piramida penduduk, gauge, hingga peta GeoJSON.
  - icon: 🌐
    title: Dwi Bahasa & Terjemahan AI
    details: Keluaran Indonesia–Inggris aktif secara bawaan; profil BRS mematikannya. Terjemahan AI opsional lewat Groq, endpoint kustom, atau server lokal, dengan draf yang diulas di tab Translation Review.
  - icon: ✅
    title: Audit Mutu
    details: Preflight statistik memeriksa sel terpotong, sumber hilang, nomor loncat, kesetaraan dwibahasa, DPI gambar, spine, dan bleed.
---

## Antarmuka PubGen Studio

Satu jendela, dua mode. **Orchestrator** adalah beranda dan ruang kerja publikasi: impor buku kerja atau buka paket `.pubgen`, pilih profil, lalu periksa kartu halaman sebelum diterbitkan.

![Beranda Orchestrator PubGen Studio: pil mode, kartu profil publikasi, dan daftar dokumen terakhir](../images/pgs-orchestrator-home.png)

*Beranda **Orchestrator** — kartu profil BPS DDA, BPS BRS, dan General membuka Studio dengan profil tersebut; paket terakhir muncul di daftar Recent Opened Documents.*

![Studio Orchestrator: daftar kartu halaman, strip QA Preflight, dan dock Sources & Content](../images/pgs-orchestrator-workspace.png)

*Studio **Orchestrator** — kartu halaman di tab **Page Structure**, strip **QA Preflight Checklist**, dan dock kanan **Sources & Content**. Tombol **Generate Publication** dan **Generate & Publish PDF** ada di tab dock **Profile & Execution**.*

Mulai dari [pengenalan PGS dan kedua modenya](/guide/01-pengenalan), atau langsung ikuti [tutorial praktis](/guide/13-tutorial).
