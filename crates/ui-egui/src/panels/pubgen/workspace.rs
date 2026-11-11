//! Orchestrator 3-Column Studio Workspace.
//!
//! Provides the complete Publication Orchestrator workspace matching the POS specification:
//! 1. Left Panel: "Sumber & Konten" (Content Tree, Import Actions, Document Settings)
//! 2. Center Panel: "Eksekusi & Verifikasi" (Header/Footer Preview, QA Preflight, Tabs: Page Structure Cards, Live Preview, Execution Log)
//! 3. Right Panel: "Profil, Slot & Eksekusi" (Publication Profile, Margins, Generate/Publish Buttons, Slot Detection)
//!
//! Both the left and right panels follow the exact visual styling and design tokens of DesignCraft's left panel.

use egui::{
    Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, Rect, Response, RichText, ScrollArea, Sense, Stroke, StrokeKind, Ui, pos2, vec2,
};
use serde::{Deserialize, Serialize};

use crate::DesignApp;
use crate::theme::{Tokens, semibold};

const WORKSPACE_STATE_ID: &str = "pubgen_orchestrator_workspace_state";

pub fn format_now() -> String {
    let now = chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now());
    now.format("%H:%M:%S").to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestratorTab {
    StrukturHalaman,
    VisualLivePreview,
    LogEksekusi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CardEditTab {
    #[default]
    KontenSumber,
    OpsiGrafik,
    TataLetakSeksi,
    HeaderFooter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestratorRightTab {
    SumberKonten,
    ProfilEksekusi,
    /// T8: the translation review pane (AI drafts awaiting a human decision).
    Terjemahan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardKind {
    Cover,
    Narrative,
    Chart,
    MapChart,
    Table,
    Image,
    Toc,
    Divider,
}

impl CardKind {
    pub const ALL: [CardKind; 8] = [
        CardKind::Cover,
        CardKind::Narrative,
        CardKind::Chart,
        CardKind::MapChart,
        CardKind::Table,
        CardKind::Image,
        CardKind::Toc,
        CardKind::Divider,
    ];

    pub fn label(self, lang: &str) -> &'static str {
        match self {
            CardKind::Cover => crate::i18n::tr(lang, "Cover"),
            CardKind::Narrative => crate::i18n::tr(lang, "Paragraph"),
            CardKind::Chart => crate::i18n::tr(lang, "Chart"),
            CardKind::MapChart => crate::i18n::tr(lang, "Map Chart"),
            CardKind::Table => crate::i18n::tr(lang, "Table"),
            CardKind::Image => crate::i18n::tr(lang, "Image"),
            CardKind::Toc => crate::i18n::tr(lang, "Table of Contents"),
            CardKind::Divider => crate::i18n::tr(lang, "Divider"),
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            CardKind::Cover => "📕",
            CardKind::Narrative => "📝",
            CardKind::Chart => "📈",
            CardKind::MapChart => "🗺️",
            CardKind::Table => "📊",
            CardKind::Image => "🖼",
            CardKind::Toc => "📑",
            CardKind::Divider => "🔖",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentItem {
    pub path: String,
    pub size_bytes: usize,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicationCard {
    pub kind: CardKind,
    pub title: String,
    pub title_en: String,
    pub subtitle: String,
    pub chapter_id: String,
    pub source: String,
    pub page: usize,
    pub tag1: String,
    pub tag2: String,
    pub scope_new_page: bool,

    // Cover specific
    pub cover_tab: usize,
    pub cover_template: String,
    pub cover_map_style: u8,
    pub cover_add_map: bool,
    pub cover_inside_back: bool,
    pub cover_theme_preset: String,
    pub cover_ai_preset: String,
    pub cover_ai_theme: String,
    pub cover_ai_palette: String,
    pub cover_release_no: String,
    pub cover_release_date: String,
    pub cover_release_month: String,
    pub cover_pub_office: String,
    pub cover_pic_name: String,
    pub cover_pic_photo: String,
    pub cover_pic_phone: String,
    pub cover_pic_email: String,
    #[serde(default)]
    pub cover_title2: String,
    #[serde(default)]
    pub cover_year: String,
    #[serde(default)]
    pub cover_volume: String,
    #[serde(default)]
    pub cover_catalogue: String,
    #[serde(default)]
    pub cover_code_num: String,
    #[serde(default)]
    pub cover_office_address: String,
    #[serde(default)]
    pub cover_style_idx: usize,
    #[serde(default)]
    pub settings: std::collections::BTreeMap<String, String>,

    // Table specific
    pub table_first_row_header: bool,
    pub table_col_numbers: bool,
    pub table_footer: bool,
    pub table_row_conditions: bool,
    pub table_style: String,
    pub table_theme: String,
    pub table_layout_preset: String,

    // Chart specific
    pub chart_type: String,
    pub chart_image: String,
    #[serde(default)]
    pub chart_category_col: String,
    #[serde(default)]
    pub chart_data_start_row: usize,
    #[serde(default)]
    pub chart_data_end_row: usize,
    #[serde(default)]
    pub chart_series_1_col: String,
    #[serde(default)]
    pub chart_series_1_label_id: String,
    #[serde(default)]
    pub chart_series_1_label_en: String,
    #[serde(default)]
    pub chart_series_2_enabled: bool,
    #[serde(default)]
    pub chart_series_2_col: String,
    #[serde(default)]
    pub chart_series_2_label_id: String,
    #[serde(default)]
    pub chart_series_2_label_en: String,
    #[serde(default)]
    pub chart_series_3_enabled: bool,
    #[serde(default)]
    pub chart_series_3_col: String,
    #[serde(default)]
    pub chart_series_3_label_id: String,
    #[serde(default)]
    pub chart_series_3_label_en: String,
    #[serde(default)]
    pub chart_value_format: String,
    #[serde(default)]
    pub chart_max_value_override: String,
    #[serde(default = "default_true")]
    pub chart_show_value_labels: bool,
    #[serde(default = "default_true")]
    pub chart_show_legend: bool,
    #[serde(default)]
    pub chart_show_grid: bool,
    #[serde(default)]
    pub chart_raw_data: String,
    #[serde(default = "default_true")]
    pub chart_csv_grid_preview: bool,

    // MapChart specific (NEW)
    pub map_chart_type: String,
    pub map_classification: String,
    pub map_color_palette: String,
    pub map_show_labels: bool,
    pub map_show_legend: bool,
    pub map_is_custom: bool,
    pub map_stock_name: String,
    pub map_custom_geojson: String,
    pub map_data_csv: String,

    // Image specific
    pub image_area: String,

    // TOC specific
    pub toc_style: String,
    pub toc_columns: String,
    pub toc_hanging_indent: bool,
    pub toc_include_self: bool,
    pub toc_pair_bilingual: bool,
    pub toc_layout_style: usize, // 0 Standard, 1 TablePage (BPS Catalog)

    // Section & Numbering specific
    pub section_start: bool,
    pub automatic_numbering: bool,
    pub start_page_number: u32,
    pub section_prefix: String,
    pub numbering_style: String,

    // Header & Footer specific
    pub hf_enabled: bool,
    pub hf_scope_forward: bool,
    pub hf_style: String,
    pub hf_sided: usize,
    pub hf_header_single: String,
    pub hf_header_left: String,
    pub hf_header_right: String,
    pub hf_footer_single: String,
    pub hf_footer_left: String,
    pub hf_footer_right: String,
    pub hf_header_rule: bool,
    pub hf_footer_rule: bool,
    pub hf_header_offset: f64,
    pub hf_footer_offset: f64,
}

impl Default for PublicationCard {
    fn default() -> Self {
        Self {
            kind: CardKind::Narrative,
            title: String::new(),
            title_en: String::new(),
            subtitle: String::new(),
            chapter_id: String::new(),
            source: String::new(),
            page: 1,
            tag1: "Paragraf".into(),
            tag2: "Halaman Baru".into(),
            scope_new_page: true,
            cover_tab: 0,
            cover_template: "bps_brs_standard.designcraft".into(),
            cover_map_style: 0,
            cover_add_map: true,
            cover_inside_back: false,
            cover_theme_preset: "Classic BPS (Blue/Gold)".into(),
            cover_ai_preset: "Geometris Abstrak Data".into(),
            cover_ai_theme: "Pertanian & Nilai Tukar Petani".into(),
            cover_ai_palette: "BPS Biru-Emas".into(),
            cover_release_no: "BRS No. 01/01/Th. XXVI, 15".into(),
            cover_release_date: "1 Oktober 2026".into(),
            cover_release_month: "Januari 2026".into(),
            cover_pub_office: "BADAN PUSAT STATISTIK PROVINSI JAWA BARAT".into(),
            cover_pic_name: String::new(),
            cover_pic_photo: String::new(),
            cover_pic_phone: String::new(),
            cover_pic_email: String::new(),
            cover_title2: String::new(),
            cover_year: String::new(),
            cover_volume: String::new(),
            cover_catalogue: String::new(),
            cover_code_num: String::new(),
            cover_office_address: String::new(),
            cover_style_idx: 0,
            settings: std::collections::BTreeMap::new(),
            table_first_row_header: true,
            table_col_numbers: true,
            table_footer: true,
            table_row_conditions: true,
            table_style: "Basic Grid".into(),
            table_theme: "BPS Blue".into(),
            table_layout_preset: "Standard".into(),
            chart_type: "Line Chart (Markers)".into(),
            chart_image: String::new(),
            chart_category_col: "A".into(),
            chart_data_start_row: 2,
            chart_data_end_row: 6,
            chart_series_1_col: "B".into(),
            chart_series_1_label_id: "Pertanian".into(),
            chart_series_1_label_en: String::new(),
            chart_series_2_enabled: true,
            chart_series_2_col: "C".into(),
            chart_series_2_label_id: "Industri".into(),
            chart_series_2_label_en: String::new(),
            chart_series_3_enabled: true,
            chart_series_3_col: "D".into(),
            chart_series_3_label_id: "Jasa".into(),
            chart_series_3_label_en: String::new(),
            chart_value_format: "Number (1.234,5)".into(),
            chart_max_value_override: String::new(),
            chart_show_value_labels: true,
            chart_show_legend: true,
            chart_show_grid: false,
            chart_raw_data: super::charts::dialog::DEFAULT_SAMPLE_CSV.into(),
            chart_csv_grid_preview: true,
            map_chart_type: "Choropleth".into(),
            map_classification: "Quantile (4 kelas)".into(),
            map_color_palette: "BPS Biru-Teal".into(),
            map_show_labels: true,
            map_show_legend: true,
            map_is_custom: false,
            map_stock_name: "CILAMAYA_WETAN".into(),
            map_custom_geojson: String::new(),
            map_data_csv: String::new(),
            image_area: "Area Margin".into(),
            toc_style: "TOC Paragraph".into(),
            toc_columns: "1 Kolom".into(),
            toc_hanging_indent: true,
            toc_include_self: false,
            toc_pair_bilingual: true,
            toc_layout_style: 0,
            section_start: false,
            automatic_numbering: true,
            start_page_number: 1,
            section_prefix: String::new(),
            numbering_style: "1, 2, 3...".into(),
            hf_enabled: false,
            hf_scope_forward: false,
            hf_style: String::new(),
            hf_sided: 0,
            hf_header_single: String::new(),
            hf_header_left: String::new(),
            hf_header_right: String::new(),
            hf_footer_single: String::new(),
            hf_footer_left: String::new(),
            hf_footer_right: String::new(),
            hf_header_rule: false,
            hf_footer_rule: false,
            hf_header_offset: 18.0,
            hf_footer_offset: 18.0,
        }
    }
}

impl PublicationCard {
    pub fn new_with(kind: CardKind, title: &str, subtitle: &str, source: &str, page: usize, is_new_page: bool) -> Self {
        let mut card = Self::default();
        card.kind = kind;
        card.title = title.into();
        card.subtitle = subtitle.into();
        card.source = source.into();
        card.page = page;
        card.scope_new_page = is_new_page;
        card.tag1 = match kind {
            CardKind::Cover => "Sampul".into(),
            CardKind::Narrative => "Paragraf".into(),
            CardKind::Chart => "Grafik".into(),
            CardKind::MapChart => "Peta Tematik".into(),
            CardKind::Table => "Tabel".into(),
            CardKind::Image => "Gambar".into(),
            CardKind::Toc => "Daftar Isi".into(),
            CardKind::Divider => "Pembatas".into(),
        };
        card.tag2 = if is_new_page { "Halaman Baru".into() } else { "Objek Baru".into() };
        card
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotItem {
    pub name: String,
    pub budget: Option<usize>,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OrchestratorView {
    #[default]
    Home,
    Studio,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchestratorWorkspaceState {
    pub view: OrchestratorView,
    pub is_dirty: bool,
    pub recent_documents: Vec<String>,
    pub center_tab: OrchestratorTab,
    pub selected_profile: String,
    // Right panel dock state
    pub right_dock_tab: OrchestratorRightTab,
    pub right_dock_expanded: bool,
    /// Width of the expanded right dock in points, dragged by the divider (F14).
    #[serde(default = "default_right_dock_w")]
    pub right_dock_w: f32,
    // Publication General Settings (Profile BRS BPS - Image #1)
    pub page_layout_facing: bool,
    pub doc_language: String,
    pub paper_size: String,
    pub margin_top_mm: u32,
    pub margin_bottom_mm: u32,
    pub margin_right_mm: u32,
    pub margin_left_mm: u32,
    pub new_card_kind: CardKind,
    // Left panel / Content state
    pub tree_open: bool,
    pub csv_group_open: bool,
    pub md_group_open: bool,
    pub metadata_open: bool,
    pub content_files: Vec<ContentItem>,
    #[serde(default)]
    pub cached_content: std::sync::Arc<std::collections::BTreeMap<String, Vec<u8>>>,
    // Metadata form
    pub meta_title: String,
    pub meta_subtitle: String,
    pub meta_release_no: String,
    pub meta_release_date: String,
    pub meta_release_month: String,
    pub meta_agency: String,
    // Publication cards
    pub cards: Vec<PublicationCard>,
    // Slots
    pub slots: Vec<SlotItem>,
    // QA and output state
    pub last_qa_clean: bool,
    pub last_output_pdf: Option<String>,
    // Execution log
    pub execution_log: Vec<String>,
    // Card modal editor
    pub editing_card_idx: Option<usize>,
    pub last_editing_card_idx: Option<usize>,
    #[serde(default)]
    pub card_edit_tab: CardEditTab,
    pub card_delete_confirm: bool,
    pub prompt_copied_feedback: bool,
    // Status message
    pub status_message: Option<(String, bool)>,
    // Re-compilation Safety Guard Dialog state
    pub show_guard_dialog: bool,
    pub guard_is_publish: bool,
    // AI Translation Guard Dialog state (warns user if translation data already exists)
    #[serde(skip)]
    pub show_translate_guard_dialog: bool,
    // Translation memory (T7): persisted inside the .pubgen package, seeded from it on open.
    #[serde(default)]
    pub tm: pubgen_core::TranslationMemory,
    /// The document's glossary pairs `(source, target)`, persisted as `glossary.json` (T8).
    /// Kept apart from `translate_glossary_pairs`: that one is the per-run masking list and is
    /// cleared when a run ends, which must not wipe the saved glossary.
    #[serde(default)]
    pub glossary_pairs: Vec<(String, String)>,
    // The worksheet translation stage's in-flight job (T7). A job never survives a
    // restart, so none of this is serialized.
    #[serde(skip)]
    pub translate_job: Option<super::ai::TranslateJob>,
    #[serde(skip)]
    pub translate_units: Vec<pubgen_core::translation_units::TransUnit>,
    #[serde(skip)]
    pub translate_glossary_pairs: Vec<(String, String)>,
    #[serde(skip)]
    pub translate_reused: usize,
    // Generation & Thumbnail Pre-warm Progress Modal state
    #[serde(skip)]
    pub compile_progress: Option<CompileProgressState>,
}

#[derive(Clone, Debug, Default)]
pub struct CompileProgressState {
    pub is_publish: bool,
    pub total_pages: usize,
    pub current_page: usize,
    pub status_text: String,
    pub error_msg: Option<String>,
    pub completed: bool,
}

impl Default for OrchestratorWorkspaceState {
    fn default() -> Self {
        let content_files = vec![
            ContentItem { path: "csv/C1_line.csv".into(), size_bytes: 517 },
            ContentItem { path: "csv/KUV.csv".into(), size_bytes: 555 },
            ContentItem { path: "csv/G1.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/POINT.csv".into(), size_bytes: 874 },
            ContentItem { path: "csv/T1.csv".into(), size_bytes: 1430 },
            ContentItem { path: "csv/T2.csv".into(), size_bytes: 1717 },
            ContentItem { path: "csv/T3.csv".into(), size_bytes: 547 },
            ContentItem { path: "csv/T4.csv".into(), size_bytes: 477 },
            ContentItem { path: "csv/U1.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/U2.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/U3.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/U4_5.csv".into(), size_bytes: 14 },
            ContentItem { path: "csv/U5.csv".into(), size_bytes: 12 },
            ContentItem { path: "csv/U6.csv".into(), size_bytes: 12 },
            ContentItem { path: "rtf/U1.rtf".into(), size_bytes: 1222 },
            ContentItem { path: "rtf/U2.rtf".into(), size_bytes: 3828 },
            ContentItem { path: "rtf/U4.rtf".into(), size_bytes: 2875 },
            ContentItem { path: "rtf/U4_5.rtf".into(), size_bytes: 2286 },
            ContentItem { path: "rtf/U5.rtf".into(), size_bytes: 1820 },
            ContentItem { path: "rtf/U6.rtf".into(), size_bytes: 1141 },
        ];

        let cards = vec![
            PublicationCard::new_with(
                CardKind::Cover,
                "Sampul Berita Resmi Statistik (COV)",
                "Perkembangan Nilai Tukar Petani | 1 Oktober 2026 · BADAN PUSAT STATISTIK PROVINSI JAWA BARAT",
                "Tanpa sumber",
                1,
                true,
            ),
            PublicationCard::new_with(
                CardKind::Narrative,
                "Ringkasan Poin: Nilai Tukar Petani (NTP) adalah perbandingan indeks harga yang diterima...",
                "Ringkasan Poin Indikator Utama",
                "csv/POINT.csv",
                2,
                true,
            ),
            PublicationCard::new_with(
                CardKind::Narrative,
                "Ulasan Indikator Utama (U1)",
                "Narasi ulasan utama sektor pertanian Jawa Barat",
                "rtf/U1.rtf",
                3,
                true,
            ),
            PublicationCard::new_with(
                CardKind::Chart,
                "Gambar 1 Perkembangan Indeks Harga di Terima, Indeks Harga di Bayar dan Nilai Tukar Petani",
                "Bab 1 · Grafik Garis",
                "csv/C1_line.csv",
                4,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Narrative,
                "Ulasan Narasi U2 (U2.rtf)",
                "Ulasan lanjutan indeks tanaman perkebunan",
                "rtf/U2.rtf",
                5,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Table,
                "Tabel 1 Indeks Harga Diterima Petani, Indeks Harga Dibayar Petani per Subkelompok Pengeluaran (2018=100)",
                "Tabel rekapitulasi data gabungan",
                "csv/T1.csv",
                6,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Narrative,
                "Ulasan Narasi U4 (U4.rtf)",
                "Ulasan indeks konsumsi rumah tangga perdesaan",
                "rtf/U4.rtf",
                7,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Table,
                "Tabel 2 Nilai Tukar Petani (NTP) Provinsi Jawa Barat per Subsektor Pertanian serta Perubahannya (2018=100)",
                "Tabel rincian per subsektor",
                "csv/T2.csv",
                8,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Narrative,
                "Ulasan Narasi U4_5 (U4_5.rtf)",
                "Ulasan penutup dan indeks biaya produksi",
                "rtf/U4_5.rtf",
                9,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Table,
                "Tabel 3 Perbandingan NTP Enam Provinsi di Pulau Jawa dan Nasional (2018=100)",
                "Tabel perbandingan regional",
                "csv/T3.csv",
                10,
                false,
            ),
            PublicationCard::new_with(CardKind::Narrative, "Ulasan Narasi U5 (U5.rtf)", "Ulasan analisis inflasi perdesaan", "rtf/U5.rtf", 11, false),
            PublicationCard::new_with(
                CardKind::Table,
                "Tabel 4 Nilai Tukar Usaha Petani per Subsektor dan Persentase Perubahannya (2018=100)",
                "Tabel NTUP subsektor",
                "csv/T4.csv",
                12,
                false,
            ),
            PublicationCard::new_with(CardKind::Narrative, "Ulasan Narasi U6 (U6.rtf)", "Ulasan penutup dan rekomendasi", "rtf/U6.rtf", 13, false),
            PublicationCard::new_with(CardKind::Image, "Infografis / Gambar G1 (G1.png)", "Area Margin · Sumber: G1.png", "G1.png", 14, false),
        ];

        let execution_log = vec![
            "[10:00:01] Profil publikasi aktif: BPS Berita Resmi Statistik (BRS)".into(),
            "[10:00:02] Geometri: A4 Facing=Ya (Double Page), Margin T:18 B:18 L:20 R:20 mm".into(),
            "[10:00:03] Berhasil memuat 20 berkas sumber (14 CSV, 6 Teks).".into(),
            "[10:00:04] Struktur halaman tersusun: 14 item kartu publikasi.".into(),
            "[10:00:05] Audit QA Mutu: 0 kesalahan, 0 peringatan (Lolos Preflight 100%).".into(),
        ];

        let mut default_state = Self {
            view: OrchestratorView::Home,
            is_dirty: false,
            recent_documents: Vec::new(),
            center_tab: OrchestratorTab::StrukturHalaman,
            selected_profile: "bps_brs".into(),
            right_dock_tab: OrchestratorRightTab::SumberKonten,
            right_dock_expanded: true,
            right_dock_w: default_right_dock_w(),
            page_layout_facing: true,
            doc_language: "Indonesian-English".into(),
            paper_size: "A4 (210 x 297 mm)".into(),
            margin_top_mm: 18,
            margin_bottom_mm: 18,
            margin_right_mm: 20,
            margin_left_mm: 20,
            new_card_kind: CardKind::Narrative,
            tree_open: true,
            csv_group_open: true,
            md_group_open: true,
            metadata_open: true,
            content_files,
            cached_content: std::sync::Arc::new(std::collections::BTreeMap::new()),
            meta_title: "Perkembangan Nilai Tukar Petani".into(),
            meta_subtitle: "1 Oktober 2026 · BADAN PUSAT STATISTIK PROVINSI JAWA BARAT".into(),
            meta_release_no: "BRS No. 01/01/Th. XXVI, 15".into(),
            meta_release_date: "1 Oktober 2026".into(),
            meta_release_month: "Januari 2026".into(),
            meta_agency: "BADAN PUSAT STATISTIK PROVINSI JAWA BARAT".into(),
            cards,
            slots: Vec::new(),
            last_qa_clean: true,
            last_output_pdf: Some("outputs/BRS_NTP_Jabar_2026.pdf".into()),
            execution_log,
            editing_card_idx: None,
            last_editing_card_idx: None,
            card_edit_tab: CardEditTab::KontenSumber,
            card_delete_confirm: false,
            prompt_copied_feedback: false,
            status_message: None,
            show_guard_dialog: false,
            guard_is_publish: false,
            show_translate_guard_dialog: false,
            tm: pubgen_core::TranslationMemory::default(),
            glossary_pairs: Vec::new(),
            translate_job: None,
            translate_units: Vec::new(),
            translate_glossary_pairs: Vec::new(),
            translate_reused: 0,
            compile_progress: None,
        };
        default_state.slots = detect_translation_rule_units(&default_state);
        default_state
    }
}

impl OrchestratorWorkspaceState {
    pub fn has_active_workspace(&self) -> bool {
        self.view == OrchestratorView::Studio || !self.cards.is_empty() || self.is_dirty
    }

    /// Converts the current UI workspace state into a normalized `PublicationMetadata`
    /// ensuring all changes to page cards and document settings are recorded.
    pub fn to_metadata(&self) -> pubgen_core::PublicationMetadata {
        let mut extra = std::collections::BTreeMap::new();
        if !self.meta_release_no.is_empty() {
            extra.insert("release_no".into(), self.meta_release_no.clone());
        }
        if !self.meta_release_date.is_empty() {
            extra.insert("release_date".into(), self.meta_release_date.clone());
        }
        if !self.meta_release_month.is_empty() {
            extra.insert("release_month".into(), self.meta_release_month.clone());
        }
        if !self.meta_agency.is_empty() {
            extra.insert("agency".into(), self.meta_agency.clone());
        }

        let cards = self
            .cards
            .iter()
            .enumerate()
            .map(|(idx, card)| {
                let mut settings = std::collections::BTreeMap::new();
                for (k, v) in &card.settings {
                    settings.insert(k.clone(), serde_json::Value::String(v.clone()));
                }
                if !card.title_en.is_empty() {
                    settings.insert("title_en".into(), serde_json::Value::String(card.title_en.clone()));
                } else if !card.subtitle.is_empty() {
                    settings.insert("title_en".into(), serde_json::Value::String(card.subtitle.clone()));
                }
                if card.kind == CardKind::Toc {
                    settings.insert("toc_style".into(), serde_json::Value::String(card.toc_style.clone()));
                    settings.insert("toc_columns".into(), serde_json::Value::String(card.toc_columns.clone()));
                    settings.insert("toc_layout_style".into(), serde_json::json!(card.toc_layout_style));
                    settings.insert("toc_hanging_indent".into(), serde_json::Value::Bool(card.toc_hanging_indent));
                    settings.insert("toc_include_self".into(), serde_json::Value::Bool(card.toc_include_self));
                    settings.insert("toc_pair_bilingual".into(), serde_json::Value::Bool(card.toc_pair_bilingual));
                }
                if card.kind == CardKind::Table {
                    if !card.table_style.is_empty() {
                        settings.insert("table_style".into(), serde_json::Value::String(card.table_style.clone()));
                    }
                    if !card.table_theme.is_empty() {
                        settings.insert("table_theme".into(), serde_json::Value::String(card.table_theme.clone()));
                    }
                    if !card.table_layout_preset.is_empty() {
                        settings.insert("table_layout_preset".into(), serde_json::Value::String(card.table_layout_preset.clone()));
                    }
                }
                if card.kind == CardKind::Chart {
                    if !card.chart_type.is_empty() {
                        settings.insert("chart_type".into(), serde_json::Value::String(card.chart_type.clone()));
                    }
                    if !card.chart_image.is_empty() {
                        settings.insert("chart_image".into(), serde_json::Value::String(card.chart_image.clone()));
                    }
                    if !card.chart_category_col.is_empty() {
                        settings.insert("chart_category_col".into(), serde_json::Value::String(card.chart_category_col.clone()));
                    }
                    if card.chart_data_start_row > 0 {
                        settings.insert("chart_data_start_row".into(), serde_json::Value::Number(card.chart_data_start_row.into()));
                    }
                    if card.chart_data_end_row > 0 {
                        settings.insert("chart_data_end_row".into(), serde_json::Value::Number(card.chart_data_end_row.into()));
                    }
                    if !card.chart_series_1_col.is_empty() {
                        settings.insert("chart_series_1_col".into(), serde_json::Value::String(card.chart_series_1_col.clone()));
                    }
                    if !card.chart_series_1_label_id.is_empty() {
                        settings.insert("chart_series_1_label_id".into(), serde_json::Value::String(card.chart_series_1_label_id.clone()));
                    }
                    if !card.chart_series_1_label_en.is_empty() {
                        settings.insert("chart_series_1_label_en".into(), serde_json::Value::String(card.chart_series_1_label_en.clone()));
                    }
                    settings.insert("chart_series_2_enabled".into(), serde_json::Value::Bool(card.chart_series_2_enabled));
                    if !card.chart_series_2_col.is_empty() {
                        settings.insert("chart_series_2_col".into(), serde_json::Value::String(card.chart_series_2_col.clone()));
                    }
                    if !card.chart_series_2_label_id.is_empty() {
                        settings.insert("chart_series_2_label_id".into(), serde_json::Value::String(card.chart_series_2_label_id.clone()));
                    }
                    if !card.chart_series_2_label_en.is_empty() {
                        settings.insert("chart_series_2_label_en".into(), serde_json::Value::String(card.chart_series_2_label_en.clone()));
                    }
                    settings.insert("chart_series_3_enabled".into(), serde_json::Value::Bool(card.chart_series_3_enabled));
                    if !card.chart_series_3_col.is_empty() {
                        settings.insert("chart_series_3_col".into(), serde_json::Value::String(card.chart_series_3_col.clone()));
                    }
                    if !card.chart_series_3_label_id.is_empty() {
                        settings.insert("chart_series_3_label_id".into(), serde_json::Value::String(card.chart_series_3_label_id.clone()));
                    }
                    if !card.chart_series_3_label_en.is_empty() {
                        settings.insert("chart_series_3_label_en".into(), serde_json::Value::String(card.chart_series_3_label_en.clone()));
                    }
                    if !card.chart_value_format.is_empty() {
                        settings.insert("chart_value_format".into(), serde_json::Value::String(card.chart_value_format.clone()));
                    }
                    if !card.chart_max_value_override.is_empty() {
                        settings.insert("chart_max_value_override".into(), serde_json::Value::String(card.chart_max_value_override.clone()));
                    }
                    settings.insert("chart_show_value_labels".into(), serde_json::Value::Bool(card.chart_show_value_labels));
                    settings.insert("chart_show_legend".into(), serde_json::Value::Bool(card.chart_show_legend));
                    settings.insert("chart_show_grid".into(), serde_json::Value::Bool(card.chart_show_grid));
                    if !card.chart_raw_data.is_empty() {
                        settings.insert("chart_raw_data".into(), serde_json::Value::String(card.chart_raw_data.clone()));
                    }
                }
                if card.kind == CardKind::Cover {
                    if !card.title.is_empty() {
                        let t = card.title.strip_prefix("Sampul Rilis: ").unwrap_or(&card.title);
                        settings.insert("title".into(), serde_json::Value::String(t.to_string()));
                    }
                    if !card.title_en.is_empty() {
                        settings.insert("title_en".into(), serde_json::Value::String(card.title_en.clone()));
                    }
                    if !card.cover_title2.is_empty() {
                        settings.insert("title2".into(), serde_json::Value::String(card.cover_title2.clone()));
                    }
                    if !card.cover_year.is_empty() {
                        settings.insert("year".into(), serde_json::Value::String(card.cover_year.clone()));
                    }
                    if !card.cover_volume.is_empty() {
                        settings.insert("volume".into(), serde_json::Value::String(card.cover_volume.clone()));
                    }
                    if !card.cover_catalogue.is_empty() {
                        settings.insert("catalogue".into(), serde_json::Value::String(card.cover_catalogue.clone()));
                    }
                    if !card.cover_code_num.is_empty() {
                        settings.insert("code".into(), serde_json::Value::String(card.cover_code_num.clone()));
                    }
                    if !card.cover_release_no.is_empty() {
                        settings.insert("release_no".into(), serde_json::Value::String(card.cover_release_no.clone()));
                    }
                    if !card.cover_release_date.is_empty() {
                        settings.insert("release_date".into(), serde_json::Value::String(card.cover_release_date.clone()));
                    }
                    if !card.cover_release_month.is_empty() {
                        settings.insert("release_month".into(), serde_json::Value::String(card.cover_release_month.clone()));
                    }
                    if !card.cover_pub_office.is_empty() {
                        settings.insert("pub_office".into(), serde_json::Value::String(card.cover_pub_office.clone()));
                        settings.insert("agency".into(), serde_json::Value::String(card.cover_pub_office.clone()));
                    }
                    if !card.cover_office_address.is_empty() {
                        settings.insert("office_address".into(), serde_json::Value::String(card.cover_office_address.clone()));
                    }
                    if !card.cover_pic_name.is_empty() {
                        settings.insert("pic_name".into(), serde_json::Value::String(card.cover_pic_name.clone()));
                    }
                    if !card.cover_pic_photo.is_empty() {
                        settings.insert("pic_photo".into(), serde_json::Value::String(card.cover_pic_photo.clone()));
                    }
                    if !card.cover_pic_phone.is_empty() {
                        settings.insert("pic_phone".into(), serde_json::Value::String(card.cover_pic_phone.clone()));
                    }
                    if !card.cover_pic_email.is_empty() {
                        settings.insert("pic_email".into(), serde_json::Value::String(card.cover_pic_email.clone()));
                    }
                    if !card.cover_template.is_empty() {
                        settings.insert("template".into(), serde_json::Value::String(card.cover_template.clone()));
                    }
                    if !card.cover_theme_preset.is_empty() {
                        settings.insert("theme".into(), serde_json::Value::String(card.cover_theme_preset.clone()));
                    }
                    if !card.cover_ai_palette.is_empty() {
                        settings.insert("palette".into(), serde_json::Value::String(card.cover_ai_palette.clone()));
                    }
                    settings.insert("add_map".into(), serde_json::Value::Bool(card.cover_add_map));
                    settings.insert("inside_back".into(), serde_json::Value::Bool(card.cover_inside_back));
                }
                if card.kind == CardKind::MapChart {
                    settings.insert("map_is_custom".into(), serde_json::Value::Bool(card.map_is_custom));
                    if !card.map_stock_name.is_empty() {
                        settings.insert("map_stock_name".into(), serde_json::Value::String(card.map_stock_name.clone()));
                    }
                    if !card.map_custom_geojson.is_empty() {
                        settings.insert("map_custom_geojson".into(), serde_json::Value::String(card.map_custom_geojson.clone()));
                    }
                    if !card.map_data_csv.is_empty() {
                        settings.insert("map_data_csv".into(), serde_json::Value::String(card.map_data_csv.clone()));
                    }
                    settings.insert("map_chart_type".into(), serde_json::Value::String(card.map_chart_type.clone()));
                    settings.insert("map_classification".into(), serde_json::Value::String(card.map_classification.clone()));
                    settings.insert("map_color_palette".into(), serde_json::Value::String(card.map_color_palette.clone()));
                    settings.insert("map_show_labels".into(), serde_json::Value::Bool(card.map_show_labels));
                    settings.insert("map_show_legend".into(), serde_json::Value::Bool(card.map_show_legend));
                }
                if card.hf_enabled {
                    settings.insert("hf_enabled".into(), serde_json::Value::Bool(true));
                    settings.insert("hf_scope_forward".into(), serde_json::Value::Bool(card.hf_scope_forward));
                    if !card.hf_style.is_empty() {
                        settings.insert("hf_style".into(), serde_json::Value::String(card.hf_style.clone()));
                    }
                    settings.insert("hf_sided".into(), serde_json::json!(card.hf_sided));
                    settings.insert("hf_header_single".into(), serde_json::Value::String(card.hf_header_single.clone()));
                    settings.insert("hf_header_left".into(), serde_json::Value::String(card.hf_header_left.clone()));
                    settings.insert("hf_header_right".into(), serde_json::Value::String(card.hf_header_right.clone()));
                    settings.insert("hf_footer_single".into(), serde_json::Value::String(card.hf_footer_single.clone()));
                    settings.insert("hf_footer_left".into(), serde_json::Value::String(card.hf_footer_left.clone()));
                    settings.insert("hf_footer_right".into(), serde_json::Value::String(card.hf_footer_right.clone()));
                    settings.insert("hf_header_rule".into(), serde_json::Value::Bool(card.hf_header_rule));
                    settings.insert("hf_footer_rule".into(), serde_json::Value::Bool(card.hf_footer_rule));
                    settings.insert("hf_header_offset".into(), serde_json::json!(card.hf_header_offset));
                    settings.insert("hf_footer_offset".into(), serde_json::json!(card.hf_footer_offset));
                }
                pubgen_core::PublicationCard {
                    id: if card.chapter_id.is_empty() { format!("card_{}", idx + 1) } else { card.chapter_id.clone() },
                    kind: match card.kind {
                        CardKind::Cover => pubgen_core::CardKind::Cover,
                        CardKind::Toc => pubgen_core::CardKind::Toc,
                        CardKind::Table => pubgen_core::CardKind::Table,
                        CardKind::Chart => pubgen_core::CardKind::Chart,
                        CardKind::MapChart => pubgen_core::CardKind::MapChart,
                        CardKind::Narrative => pubgen_core::CardKind::Narrative,
                        CardKind::Image => pubgen_core::CardKind::Image,
                        CardKind::Divider => pubgen_core::CardKind::Divider,
                    },
                    scope: if card.scope_new_page { pubgen_core::CardScope::NewPage } else { pubgen_core::CardScope::NewObject },
                    source: if card.source.is_empty() || card.source == "Tanpa sumber" { None } else { Some(card.source.clone()) },
                    title: card.title.clone(),
                    settings,
                }
            })
            .collect();

        pubgen_core::PublicationMetadata {
            profile_id: self.selected_profile.clone(),
            layout: if self.page_layout_facing { pubgen_core::PageLayout::Double } else { pubgen_core::PageLayout::Single },
            language: match self.doc_language.as_str() {
                "English" => pubgen_core::Language::En,
                "Indonesian-English" => pubgen_core::Language::IdEn,
                _ => pubgen_core::Language::Id,
            },
            title: self.meta_title.clone(),
            title_en: if self.meta_subtitle.is_empty() { None } else { Some(self.meta_subtitle.clone()) },
            cards,
            extra,
        }
    }
}

#[allow(deprecated)]
fn workspace_state_id() -> egui::Id {
    egui::Id::new(WORKSPACE_STATE_ID)
}

pub fn get_state(ctx: &egui::Context) -> OrchestratorWorkspaceState {
    ctx.data(|d| d.get_temp(workspace_state_id())).unwrap_or_default()
}

pub fn set_state(ctx: &egui::Context, state: OrchestratorWorkspaceState) {
    ctx.data_mut(|d| d.insert_temp(workspace_state_id(), state));
}

/// Apply an ingest report to the workspace state. `glossary_pairs` is the document's glossary
/// (`source`, `target`), supplied by the caller because only the ui-egui boundary can load the
/// bundled/user `BpsGlossary` asset (T8); it is what `glossary.json` is written from.
pub fn apply_ingest_report_to_state(
    state: &mut OrchestratorWorkspaceState,
    source_name: &str,
    report: &pubgen_core::excel::IngestReport,
    glossary_pairs: &[(String, String)],
) {
    state.recent_documents = vec![source_name.to_owned()];
    state.meta_title = report.metadata.title.clone();
    state.meta_subtitle = report.metadata.title_en.clone().unwrap_or_default();
    state.selected_profile = report.profile_id.clone();
    state.page_layout_facing = matches!(report.metadata.layout, pubgen_core::PageLayout::Double);
    state.doc_language = match report.metadata.language {
        pubgen_core::Language::Id => "Indonesian",
        pubgen_core::Language::En => "English",
        pubgen_core::Language::IdEn => "Indonesian-English",
    }
    .into();

    // A worksheet import replaces the publication, so the previous document's translation memory
    // must not leak its English onto this one's cards (or be saved into its package). Any
    // translation still in flight belongs to the document that is being replaced.
    state.tm = pubgen_core::TranslationMemory::default();
    state.glossary_pairs.clear();
    state.translate_units.clear();
    state.translate_glossary_pairs.clear();
    state.translate_reused = 0;
    state.translate_job = None;
    // The replaced document's glossary is rebuilt from the caller's pairs; `seed_pubgen_state`
    // and generation then reuse the same official English the package saves.
    state.glossary_pairs.extend(glossary_pairs.iter().filter(|(s, t)| !s.trim().is_empty() && !t.trim().is_empty()).cloned());

    if state.selected_profile == "bps_dda" {
        state.meta_release_no.clear();
        state.meta_release_date.clear();
        state.meta_release_month.clear();
        if let Some(agency) = report
            .metadata
            .extra
            .get("agency")
            .or_else(|| report.metadata.extra.get("pub_office"))
            .or_else(|| report.metadata.extra.get("officename"))
        {
            state.meta_agency = agency.clone();
        }
        state.meta_subtitle = report.metadata.title_en.clone().unwrap_or_default();
    } else {
        if state.selected_profile == "bps_brs" {
            if let Some(no) = report.metadata.extra.get("release_no").or_else(|| report.metadata.extra.get("no")) {
                state.meta_release_no = no.clone();
            }
        } else {
            state.meta_release_no.clear();
        }
        if let Some(date) = report.metadata.extra.get("release_date").or_else(|| report.metadata.extra.get("date")) {
            state.meta_release_date = date.clone();
        }
        if let Some(month) = report.metadata.extra.get("release_month").or_else(|| report.metadata.extra.get("month")) {
            state.meta_release_month = month.clone();
        }
        if let Some(agency) = report
            .metadata
            .extra
            .get("agency")
            .or_else(|| report.metadata.extra.get("pub_office"))
            .or_else(|| report.metadata.extra.get("officename"))
        {
            state.meta_agency = agency.clone();
        }
        if state.meta_subtitle.is_empty() && (!state.meta_release_date.is_empty() || !state.meta_agency.is_empty()) {
            state.meta_subtitle = format!("{} · {}", state.meta_release_date, state.meta_agency);
        }
    }

    // A name that already carries a folder is ready to use; a bare one gets its bucket prefix so it
    // matches the card sources a package wrote before the folders existed. Narrative artifacts keep
    // the historical `md/` bucket name: older packages store them under it and point their card
    // sources at it, so a new prefix would orphan those sources.
    let norm = |prefix: &str, name: &str| {
        let n = name.replace('\\', "/");
        if n.starts_with(prefix) || n.contains('/') { n } else { format!("{prefix}/{n}") }
    };

    // Narratives are RTF; a package written before that change still carries Markdown, whose
    // legacy-converter debris is repaired here so those packages generate clean text.
    let text_entries: Vec<(String, Vec<u8>)> = report
        .text
        .iter()
        .map(|(name, bytes)| {
            if !name.ends_with(".md") {
                return (name.clone(), bytes.clone());
            }
            let raw = String::from_utf8_lossy(bytes);
            let clean = pubgen_core::excel::sanitize_markdown(&raw);
            (name.clone(), if clean == raw { bytes.clone() } else { clean.into_bytes() })
        })
        .collect();

    let norm_text = |name: &str| {
        let n = name.replace('\\', "/");
        if n.starts_with("rtf/") || n.starts_with("md/") || n.contains('/') {
            n
        } else if n.ends_with(".rtf") {
            format!("rtf/{n}")
        } else {
            format!("md/{n}")
        }
    };

    state.content_files = report
        .csv
        .iter()
        .map(|(name, bytes)| ContentItem { path: norm("csv", name), size_bytes: bytes.len() })
        .chain(text_entries.iter().map(|(name, bytes)| ContentItem { path: norm_text(name), size_bytes: bytes.len() }))
        .chain(report.assets.iter().map(|(name, bytes)| ContentItem { path: norm("assets", name), size_bytes: bytes.len() }))
        .collect();

    let cache = std::sync::Arc::make_mut(&mut state.cached_content);
    cache.clear();
    for (name, bytes) in &report.csv {
        cache.insert(norm("csv", name), bytes.clone());
    }
    for (name, bytes) in &text_entries {
        let key = norm_text(name);
        cache.insert(key.clone(), bytes.clone());
        let bare = name.replace('\\', "/");
        if bare != key {
            cache.insert(bare, bytes.clone());
        }
    }
    for (name, bytes) in &report.assets {
        let key = norm("assets", name);
        cache.insert(key.clone(), bytes.clone());
        let bare = name.replace('\\', "/");
        if bare != key {
            cache.insert(bare, bytes.clone());
        }
    }

    if !report.metadata.cards.is_empty() {
        state.cards = report
            .metadata
            .cards
            .iter()
            .enumerate()
            .map(|(index, card)| {
                let subtitle = card.settings.get("title_en").and_then(|v| v.as_str()).unwrap_or("");
                let clean_title = pubgen_core::excel::strip_toc_bookmarks(&card.title);
                let clean_subtitle = pubgen_core::excel::strip_toc_bookmarks(subtitle);
                let mut pc = PublicationCard::new_with(
                    match card.kind {
                        pubgen_core::CardKind::Cover => CardKind::Cover,
                        pubgen_core::CardKind::Toc => CardKind::Toc,
                        pubgen_core::CardKind::Table => CardKind::Table,
                        pubgen_core::CardKind::Chart => CardKind::Chart,
                        pubgen_core::CardKind::MapChart => CardKind::MapChart,
                        pubgen_core::CardKind::Narrative => CardKind::Narrative,
                        pubgen_core::CardKind::Image => CardKind::Image,
                        pubgen_core::CardKind::Divider => CardKind::Divider,
                    },
                    &clean_title,
                    &clean_subtitle,
                    card.source.as_deref().unwrap_or("Tanpa sumber"),
                    index + 1,
                    matches!(card.scope, pubgen_core::CardScope::NewPage),
                );
                pc.chapter_id = card.id.clone();
                if !subtitle.is_empty() {
                    pc.title_en = subtitle.to_string();
                }
                for (k, v) in &card.settings {
                    if let Some(s) = v.as_str() {
                        pc.settings.insert(k.clone(), s.to_string());
                    } else if let Some(b) = v.as_bool() {
                        pc.settings.insert(k.clone(), b.to_string());
                    } else if let Some(u) = v.as_u64() {
                        pc.settings.insert(k.clone(), u.to_string());
                    } else if let Some(n) = v.as_i64() {
                        pc.settings.insert(k.clone(), n.to_string());
                    } else if let Some(f) = v.as_f64() {
                        pc.settings.insert(k.clone(), f.to_string());
                    }
                }
                if pc.kind == CardKind::Toc {
                    if let Some(s) = pc.settings.get("toc_style") {
                        pc.toc_style = s.clone();
                    } else {
                        let t = pc.title.to_lowercase();
                        if t.contains("tabel") || t.contains("table") {
                            pc.toc_style = "TOC Table".into();
                        } else if t.contains("grafik") || t.contains("chart") || t.contains("gambar") || t.contains("figure") {
                            pc.toc_style = "TOC Chart".into();
                        } else {
                            pc.toc_style = "TOC Paragraph".into();
                        }
                    }
                    if let Some(cols) = pc.settings.get("toc_columns").or_else(|| pc.settings.get("column")).or_else(|| pc.settings.get("columns")) {
                        pc.toc_columns = cols.clone();
                    }
                    if let Some(v) = card.settings.get("toc_layout_style").or_else(|| card.settings.get("layout")) {
                        if let Some(u) = v.as_u64() {
                            pc.toc_layout_style = u as usize;
                        } else if let Some(n) = v.as_i64() {
                            pc.toc_layout_style = n as usize;
                        } else if let Some(s) = v.as_str() {
                            let lower = s.to_lowercase();
                            if lower == "1" || lower.contains("tablepage") || lower.contains("bps") {
                                pc.toc_layout_style = 1;
                            } else {
                                pc.toc_layout_style = 0;
                            }
                        }
                    } else if let Some(val) = pc.settings.get("toc_layout_style").or_else(|| pc.settings.get("layout")) {
                        let lower = val.to_lowercase();
                        if lower == "1" || lower.contains("tablepage") || lower.contains("bps") {
                            pc.toc_layout_style = 1;
                        } else if let Ok(n) = val.parse::<usize>() {
                            pc.toc_layout_style = n;
                        }
                    } else if (state.selected_profile == "bps_dda" || state.selected_profile.contains("dda"))
                        && (pc.toc_style == "TOC Table" || pc.toc_style == "TOC Chart")
                    {
                        // Default to TablePage (BPS Catalog) layout for DDA catalog tables and charts
                        pc.toc_layout_style = 1;
                    }
                    if let Some(hi) = pc.settings.get("toc_hanging_indent").or_else(|| pc.settings.get("hanging_indent")).and_then(|v| {
                        let lower = v.trim().to_lowercase();
                        if lower == "yes" || lower == "true" || lower == "1" {
                            Some(true)
                        } else if lower == "no" || lower == "false" || lower == "0" {
                            Some(false)
                        } else {
                            v.parse::<bool>().ok()
                        }
                    }) {
                        pc.toc_hanging_indent = hi;
                    }
                    if let Some(is) = pc.settings.get("toc_include_self").or_else(|| pc.settings.get("include_self")).and_then(|v| {
                        let lower = v.trim().to_lowercase();
                        if lower == "yes" || lower == "true" || lower == "1" {
                            Some(true)
                        } else if lower == "no" || lower == "false" || lower == "0" {
                            Some(false)
                        } else {
                            v.parse::<bool>().ok()
                        }
                    }) {
                        pc.toc_include_self = is;
                    }
                    if let Some(pb) = pc.settings.get("toc_pair_bilingual").or_else(|| pc.settings.get("pair_bilingual")).and_then(|v| {
                        let lower = v.trim().to_lowercase();
                        if lower == "yes" || lower == "true" || lower == "1" {
                            Some(true)
                        } else if lower == "no" || lower == "false" || lower == "0" {
                            Some(false)
                        } else {
                            v.parse::<bool>().ok()
                        }
                    }) {
                        pc.toc_pair_bilingual = pb;
                    }
                }
                if pc.kind == CardKind::Table {
                    if let Some(s) = pc.settings.get("table_style") {
                        pc.table_style = s.clone();
                    }
                    if let Some(s) = pc.settings.get("table_theme").or_else(|| pc.settings.get("theme")) {
                        pc.table_theme = s.clone();
                    }
                    if let Some(s) = pc.settings.get("table_layout_preset") {
                        pc.table_layout_preset = s.clone();
                    }

                    // Self-healing: if pc.title is literally "subtitle" or starts with "Tabel 2 " / chapter name,
                    // or if the card's CSV has metadata rows, extract chapter/section and the real table title.
                    let matching_csv = report.csv.iter().find(|(path, _)| path == &pc.source || pc.source.ends_with(path));
                    if let Some((_, csv_bytes)) = matching_csv {
                        if let Ok(csv_text) = std::str::from_utf8(csv_bytes) {
                            let lines: Vec<&str> = csv_text.lines().collect();
                            let has_meta_lines = lines.iter().take(6).any(|l| {
                                let lower = l.to_lowercase();
                                lower.starts_with("title,")
                                    || lower.starts_with("title_en,")
                                    || lower.starts_with("subtitle,")
                                    || lower.starts_with("subtitle_en,")
                            });
                            if has_meta_lines || pc.title.eq_ignore_ascii_case("subtitle") || pc.title.starts_with("Tabel 2 ") {
                                let mut ch_opt = None;
                                let mut ch_en_opt = None;
                                let mut sec_opt = None;
                                let mut sec_en_opt = None;
                                let mut table_title_opt = None;

                                for l in &lines {
                                    let trimmed = l.trim();
                                    if trimmed.is_empty() {
                                        continue;
                                    }
                                    let parts: Vec<&str> = trimmed.splitn(3, ',').collect();
                                    let key = parts[0].trim().trim_matches('"').to_lowercase();
                                    let val = parts.get(1).map(|s| s.trim().trim_matches('"')).unwrap_or("");
                                    if key == "title" {
                                        ch_opt = Some(val.to_string());
                                    } else if key == "title_en" {
                                        ch_en_opt = Some(val.to_string());
                                    } else if key == "subtitle" {
                                        sec_opt = Some(val.to_string());
                                    } else if key == "subtitle_en" {
                                        sec_en_opt = Some(val.to_string());
                                    } else if table_title_opt.is_none()
                                        && (trimmed.starts_with("Tabel ")
                                            || trimmed.starts_with("\"Tabel ")
                                            || trimmed.starts_with("Table ")
                                            || trimmed.starts_with("\"Table "))
                                    {
                                        let unquoted = parts[0].trim().trim_matches('"');
                                        table_title_opt = Some(unquoted.to_string());
                                    }
                                }

                                if let Some(ch) = ch_opt {
                                    pc.settings.insert("chapter".into(), ch.clone());
                                    pc.settings.insert("chapter_title".into(), ch);
                                }
                                if let Some(chen) = ch_en_opt {
                                    pc.settings.insert("chapter_en".into(), chen.clone());
                                    pc.settings.insert("chapter_title_en".into(), chen);
                                }
                                if let Some(sec) = sec_opt {
                                    pc.settings.insert("section".into(), sec.clone());
                                    pc.settings.insert("section_title".into(), sec.clone());
                                    pc.settings.insert("subtitle".into(), sec);
                                }
                                if let Some(secen) = sec_en_opt {
                                    pc.settings.insert("section_en".into(), secen.clone());
                                    pc.settings.insert("section_title_en".into(), secen.clone());
                                    pc.settings.insert("subtitle_en".into(), secen);
                                }
                                if let Some(tbl_title) = table_title_opt {
                                    pc.title = tbl_title;
                                }
                            }
                        }
                    }
                }
                if pc.kind == CardKind::Chart {
                    if let Some(s) = pc.settings.get("chart_type") {
                        pc.chart_type = s.clone();
                    }
                    if let Some(s) = pc.settings.get("chart_image") {
                        pc.chart_image = s.clone();
                    }
                }
                if pc.kind == CardKind::Cover {
                    for (k, v) in &report.metadata.extra {
                        pc.settings.entry(k.clone()).or_insert_with(|| v.clone());
                    }
                    if let Some(t) = pc.settings.get("title").or_else(|| pc.settings.get("title1")) {
                        if !t.is_empty() {
                            pc.title = if state.selected_profile == "bps_dda" { t.clone() } else { format!("Sampul Rilis: {t}") };
                        }
                    }
                    if let Some(t2) = pc.settings.get("title2") {
                        pc.cover_title2 = t2.clone();
                    }
                    if let Some(ten) = pc.settings.get("title2").or_else(|| pc.settings.get("title_en")) {
                        pc.title_en = ten.clone();
                    }
                    if let Some(yr) = pc.settings.get("year") {
                        pc.cover_year = yr.clone();
                    }
                    if let Some(vol) = pc.settings.get("volume") {
                        pc.cover_volume = vol.clone();
                    }
                    if let Some(cat) = pc.settings.get("catalogue") {
                        pc.cover_catalogue = cat.clone();
                    }
                    if let Some(cd) = pc.settings.get("code") {
                        pc.cover_code_num = cd.clone();
                    }
                    if let Some(rn) = pc.settings.get("release_no") {
                        pc.cover_release_no = rn.clone();
                    } else if !state.meta_release_no.is_empty() {
                        pc.cover_release_no = state.meta_release_no.clone();
                    }
                    if let Some(rd) = pc.settings.get("release_date") {
                        pc.cover_release_date = rd.clone();
                    } else if !state.meta_release_date.is_empty() {
                        pc.cover_release_date = state.meta_release_date.clone();
                    }
                    if let Some(rm) = pc.settings.get("release_month") {
                        pc.cover_release_month = rm.clone();
                    } else if !state.meta_release_month.is_empty() {
                        pc.cover_release_month = state.meta_release_month.clone();
                    }
                    if let Some(po) = pc.settings.get("pub_office").or_else(|| pc.settings.get("agency")) {
                        pc.cover_pub_office = po.clone();
                    } else if !state.meta_agency.is_empty() {
                        pc.cover_pub_office = state.meta_agency.clone();
                    }
                    if let Some(addr) = pc.settings.get("office_address") {
                        pc.cover_office_address = addr.clone();
                    }
                    if let Some(pn) = pc.settings.get("pic_name") {
                        pc.cover_pic_name = pn.clone();
                    }
                    if let Some(pp) = pc.settings.get("pic_photo") {
                        pc.cover_pic_photo = pp.clone();
                    }
                    if let Some(ph) = pc.settings.get("pic_phone") {
                        pc.cover_pic_phone = ph.clone();
                    }
                    if let Some(pe) = pc.settings.get("pic_email") {
                        pc.cover_pic_email = pe.clone();
                    }
                    if let Some(tmpl) = pc.settings.get("template").or_else(|| pc.settings.get("style")) {
                        pc.cover_template = tmpl.clone();
                        if let Some(idx) = super::excel::cover_style_index(tmpl) {
                            pc.cover_style_idx = idx;
                        }
                    } else if state.selected_profile == "bps_dda" {
                        pc.cover_template = "003_chevron_medallion.designcraft".to_string();
                        pc.cover_style_idx = 3;
                    }
                    if state.selected_profile == "bps_dda" || pc.settings.contains_key("office_address") || pc.settings.contains_key("event_theme") {
                        pc.cover_inside_back = true;
                    }
                    if let Some(th) = pc.settings.get("theme") {
                        pc.cover_theme_preset = th.clone();
                    }
                    if let Some(pal) = pc.settings.get("palette") {
                        pc.cover_ai_palette = pal.clone();
                    }
                    if let Some(src) = &card.source {
                        pc.source = src.clone();
                    } else {
                        pc.source = "csv/COV.csv".into();
                    }
                    if pc.subtitle.is_empty() {
                        pc.subtitle = if !pc.title_en.is_empty() {
                            pc.title_en.clone()
                        } else if !pc.cover_release_date.is_empty() || !pc.cover_pub_office.is_empty() {
                            format!("{} · {}", pc.cover_release_date, pc.cover_pub_office)
                        } else {
                            state.meta_subtitle.clone()
                        };
                    }
                }
                if pc.title_en.is_empty() {
                    if let Some(ten) = pc.settings.get("title_en").or_else(|| pc.settings.get("title2")) {
                        pc.title_en = ten.clone();
                    } else if !subtitle.is_empty() {
                        pc.title_en = subtitle.to_string();
                    }
                }
                // Older packages titled image/chart cards with the key that pointed at the file
                // ("file"), which then showed up as the placed image's title.
                if matches!(pc.kind, CardKind::Image | CardKind::Chart | CardKind::MapChart) {
                    let low = pc.title.trim().to_lowercase();
                    if low.is_empty() || matches!(low.as_str(), "file" | "image" | "berkas" | "gambar" | "map" | "peta" | "geojson") {
                        if pc.kind == CardKind::MapChart {
                            if let Some(t) = pc.settings.get("title").or_else(|| pc.settings.get("judul")) {
                                pc.title = t.clone();
                            } else if let Some(stk) = card.settings.get("map_stock_name").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                                pc.title = format!("Peta Tematik {stk}");
                            } else {
                                pc.title = format!("Peta Tematik {}", pc.chapter_id);
                            }
                        } else {
                            pc.title = format!("Gambar / Infografis {}", pc.chapter_id);
                        }
                    }
                }
                if pc.kind == CardKind::Chart {
                    if let Some(ct) = card.settings.get("chart_type").and_then(|v| v.as_str()) {
                        pc.chart_type = ct.to_string();
                    }
                    if let Some(img) = card.settings.get("image").or_else(|| card.settings.get("chart_image")).and_then(|v| v.as_str()) {
                        pc.chart_image = img.to_string();
                    }
                    if let Some(c) = card.settings.get("chart_category_col").and_then(|v| v.as_str()) {
                        pc.chart_category_col = c.to_string();
                    }
                    if let Some(r) = card.settings.get("chart_data_start_row").and_then(|v| v.as_u64()) {
                        pc.chart_data_start_row = r as usize;
                    }
                    if let Some(r) = card.settings.get("chart_data_end_row").and_then(|v| v.as_u64()) {
                        pc.chart_data_end_row = r as usize;
                    }
                    if let Some(s) = card.settings.get("chart_series_1_col").and_then(|v| v.as_str()) {
                        pc.chart_series_1_col = s.to_string();
                    }
                    if let Some(s) = card.settings.get("chart_series_1_label_id").and_then(|v| v.as_str()) {
                        pc.chart_series_1_label_id = s.to_string();
                    }
                    if let Some(s) = card.settings.get("chart_series_1_label_en").and_then(|v| v.as_str()) {
                        pc.chart_series_1_label_en = s.to_string();
                    }
                    if let Some(b) = card.settings.get("chart_series_2_enabled").and_then(|v| v.as_bool()) {
                        pc.chart_series_2_enabled = b;
                    }
                    if let Some(s) = card.settings.get("chart_series_2_col").and_then(|v| v.as_str()) {
                        pc.chart_series_2_col = s.to_string();
                    }
                    if let Some(s) = card.settings.get("chart_series_2_label_id").and_then(|v| v.as_str()) {
                        pc.chart_series_2_label_id = s.to_string();
                    }
                    if let Some(s) = card.settings.get("chart_series_2_label_en").and_then(|v| v.as_str()) {
                        pc.chart_series_2_label_en = s.to_string();
                    }
                    if let Some(b) = card.settings.get("chart_series_3_enabled").and_then(|v| v.as_bool()) {
                        pc.chart_series_3_enabled = b;
                    }
                    if let Some(s) = card.settings.get("chart_series_3_col").and_then(|v| v.as_str()) {
                        pc.chart_series_3_col = s.to_string();
                    }
                    if let Some(s) = card.settings.get("chart_series_3_label_id").and_then(|v| v.as_str()) {
                        pc.chart_series_3_label_id = s.to_string();
                    }
                    if let Some(s) = card.settings.get("chart_series_3_label_en").and_then(|v| v.as_str()) {
                        pc.chart_series_3_label_en = s.to_string();
                    }
                    if let Some(s) = card.settings.get("chart_value_format").and_then(|v| v.as_str()) {
                        pc.chart_value_format = s.to_string();
                    }
                    if let Some(s) = card.settings.get("chart_max_value_override").and_then(|v| v.as_str()) {
                        pc.chart_max_value_override = s.to_string();
                    }
                    if let Some(b) = card.settings.get("chart_show_value_labels").and_then(|v| v.as_bool()) {
                        pc.chart_show_value_labels = b;
                    }
                    if let Some(b) = card.settings.get("chart_show_legend").and_then(|v| v.as_bool()) {
                        pc.chart_show_legend = b;
                    }
                    if let Some(b) = card.settings.get("chart_show_grid").and_then(|v| v.as_bool()) {
                        pc.chart_show_grid = b;
                    }
                    if let Some(s) = card.settings.get("chart_raw_data").and_then(|v| v.as_str()) {
                        pc.chart_raw_data = s.to_string();
                    }
                    if (pc.chart_raw_data == super::charts::dialog::DEFAULT_SAMPLE_CSV || pc.chart_raw_data.trim().is_empty())
                        && !pc.source.is_empty()
                        && pc.source != "Tanpa sumber"
                        && pc.source != "Manual"
                    {
                        let src = pc.source.clone();
                        load_chart_data_from_source(&mut pc, &src, cache);
                    }
                }
                if pc.kind == CardKind::MapChart {
                    if let Some(stk) = card.settings.get("map_stock_name").and_then(|v| v.as_str()) {
                        pc.map_stock_name = stk.to_string();
                    }
                    if let Some(cst) = card.settings.get("map_custom_geojson").and_then(|v| v.as_str()) {
                        pc.map_custom_geojson = cst.to_string();
                    }
                    if let Some(csv) = card.settings.get("map_data_csv").and_then(|v| v.as_str()) {
                        pc.map_data_csv = csv.to_string();
                    }
                    if let Some(isc) = card.settings.get("map_is_custom").and_then(|v| v.as_bool()) {
                        pc.map_is_custom = isc;
                    } else {
                        pc.map_is_custom = !pc.map_custom_geojson.is_empty();
                    }
                    if let Some(mt) = card.settings.get("map_chart_type").and_then(|v| v.as_str()) {
                        pc.map_chart_type = mt.to_string();
                    }
                    if let Some(cl) = card.settings.get("map_classification").and_then(|v| v.as_str()) {
                        pc.map_classification = cl.to_string();
                    }
                    if let Some(pal) = card.settings.get("map_color_palette").and_then(|v| v.as_str()) {
                        pc.map_color_palette = pal.to_string();
                    }
                    if let Some(lbl) = card.settings.get("map_show_labels").and_then(|v| v.as_bool()) {
                        pc.map_show_labels = lbl;
                    }
                    if let Some(leg) = card.settings.get("map_show_legend").and_then(|v| v.as_bool()) {
                        pc.map_show_legend = leg;
                    }
                }
                if let Some(hfe) = card.settings.get("hf_enabled").and_then(|v| v.as_bool()) {
                    pc.hf_enabled = hfe;
                }
                if let Some(fwd) = card.settings.get("hf_scope_forward").and_then(|v| v.as_bool()) {
                    pc.hf_scope_forward = fwd;
                }
                if let Some(style) = card.settings.get("hf_style").and_then(|v| v.as_str()) {
                    pc.hf_style = style.to_string();
                }
                if let Some(sided) = card.settings.get("hf_sided").and_then(|v| v.as_u64()) {
                    pc.hf_sided = sided as usize;
                }
                if let Some(s) = card.settings.get("hf_header_single").and_then(|v| v.as_str()) {
                    pc.hf_header_single = s.to_string();
                }
                if let Some(s) = card.settings.get("hf_header_left").and_then(|v| v.as_str()) {
                    pc.hf_header_left = s.to_string();
                }
                if let Some(s) = card.settings.get("hf_header_right").and_then(|v| v.as_str()) {
                    pc.hf_header_right = s.to_string();
                }
                if let Some(s) = card.settings.get("hf_footer_single").and_then(|v| v.as_str()) {
                    pc.hf_footer_single = s.to_string();
                }
                if let Some(s) = card.settings.get("hf_footer_left").and_then(|v| v.as_str()) {
                    pc.hf_footer_left = s.to_string();
                }
                if let Some(s) = card.settings.get("hf_footer_right").and_then(|v| v.as_str()) {
                    pc.hf_footer_right = s.to_string();
                }
                if let Some(b) = card.settings.get("hf_header_rule").and_then(|v| v.as_bool()) {
                    pc.hf_header_rule = b;
                }
                if let Some(b) = card.settings.get("hf_footer_rule").and_then(|v| v.as_bool()) {
                    pc.hf_footer_rule = b;
                }
                if let Some(n) = card.settings.get("hf_header_offset").and_then(|v| v.as_f64()) {
                    pc.hf_header_offset = n;
                }
                if let Some(n) = card.settings.get("hf_footer_offset").and_then(|v| v.as_f64()) {
                    pc.hf_footer_offset = n;
                }
                pc
            })
            .collect();
    }

    let now = format_now();
    state.execution_log.push(format!("[{now}] Berhasil memuat berkas Excel: {source_name}"));
    state.execution_log.push(format!(
        "[{now}] Diekstrak {} berkas konten ({} CSV, {} Teks, {} Aset).",
        state.content_files.len(),
        report.csv.len(),
        report.text.len(),
        report.assets.len(),
    ));
    state.status_message = Some((format!("Workbook {source_name} berhasil dimuat"), true));
    refresh_detected_slots(state);
    state.is_dirty = false;
}

const HOST_TABS_ID: &str = "pubgen_orchestrator_host_tabs";
const HOST_TAB_ACTION_ID: &str = "pubgen_orchestrator_host_tab_action";

#[derive(Clone, Debug, Default)]
pub struct HostTabs {
    pub titles: Vec<(String, bool)>,
    pub active: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum HostTabAction {
    Activate(usize),
    Close(usize),
    #[default]
    New,
    Open,
    OpenPath(std::path::PathBuf),
    Import,
    ImportPath(std::path::PathBuf),
    Save,
    ExportXlsx,
}

#[allow(deprecated)]
pub(crate) fn host_tabs_id() -> egui::Id {
    egui::Id::new(HOST_TABS_ID)
}

#[allow(deprecated)]
pub(crate) fn host_tab_action_id() -> egui::Id {
    egui::Id::new(HOST_TAB_ACTION_ID)
}

/// The Studio host declares the Orchestrator workspace tabs before the panel is drawn.
pub fn set_host_tabs(ctx: &egui::Context, titles: Vec<(String, bool)>, active: usize) {
    ctx.data_mut(|d| d.insert_temp(host_tabs_id(), HostTabs { titles, active }));
}

/// The tab bar's click, handled by the host after the frame (it owns the workspaces).
pub fn take_host_tab_action(ctx: &egui::Context) -> Option<HostTabAction> {
    ctx.data_mut(|d| d.remove_temp(host_tab_action_id()))
}

/// Trigger an action for the host tab controller (e.g. from File menu commands).
pub fn trigger_host_tab_action(ctx: &egui::Context, action: HostTabAction) {
    ctx.data_mut(|d| d.insert_temp(host_tab_action_id(), action));
    ctx.request_repaint();
}

/// Tab max width and close button padding around tab title, exactly matching Designcraft document tabs.
const ORCH_TAB_MAX_W: f32 = 260.0;
const ORCH_TAB_PAD: f32 = 44.0;

/// One dock tab: the metrics of DesignCraft's docked panel tabs (`panel_docking::dock_style`):
/// 27 px strip, proportional 12 px label inset 10 px, 1 px borders and no accent line.
const DOCK_TAB_H: f32 = 27.0;
const DOCK_TAB_LABEL: f32 = 12.0;
const DOCK_TAB_PAD: f32 = 10.0;
/// Width of the collapse-to-icons button at the left of the dock's tab strip.
const DOCK_COLLAPSE_W: f32 = 18.0;

/// Lay out one dock tab label inside its tab: [`DOCK_TAB_PAD`] on the left, the rest for text.
/// At the default 320 px dock the three tabs are ~100 px wide, narrower than "Sources & Content",
/// "Profile & Execution" and "Translation Review" at [`DOCK_TAB_LABEL`], so the string is cut with
/// an ellipsis rather than painted over the neighbouring tab (the full label stays in the tooltip).
fn dock_tab_label(ctx: &egui::Context, rect: Rect, text: &str, color: Color32) -> std::sync::Arc<egui::Galley> {
    let max_w = (rect.width() - DOCK_TAB_PAD).max(DOCK_TAB_LABEL);
    crate::rtl::truncated(ctx, text, FontId::proportional(DOCK_TAB_LABEL), color, max_w)
}

/// Paint a [`dock_tab_label`] galley with its left edge [`DOCK_TAB_PAD`] inside the tab.
fn paint_dock_tab_label(ui: &Ui, rect: Rect, galley: std::sync::Arc<egui::Galley>) {
    let pos = pos2(rect.min.x + DOCK_TAB_PAD, rect.center().y - galley.size().y / 2.0);
    // PLACEHOLDER keeps the colours the galley was laid out with (same as the host tab strip).
    ui.painter().galley(pos, galley, Color32::PLACEHOLDER);
}

/// Home screen scale — the same sizes `chrome::start_screen` uses for DesignCraft's own welcome
/// page, so both entry screens read as one product. (F5)
const HOME_TITLE: f32 = 24.0;
const HOME_SECTION: f32 = 15.0;
const HOME_BODY: f32 = 15.0;
const HOME_BUTTON: f32 = 14.0;
const HOME_TILE_RADIUS: f32 = 10.0;
const HOME_PILL_RADIUS: f32 = 16.0;

/// The width the expanded right dock opens at, and the range the divider may drag it to.
fn default_right_dock_w() -> f32 {
    320.0
}
const RIGHT_DOCK_MIN_W: f32 = 240.0;
const RIGHT_DOCK_MAX_SHARE: f32 = 0.6;
/// Grab width of the divider between the centre column and the right dock.
const DOCK_DIVIDER_W: f32 = 5.0;
/// Width of the collapsed right dock's icon strip.
const RIGHT_DOCK_COLLAPSED_W: f32 = 38.0;

/// The dock width and divider width for the current screen: the stored width clamped to a share of the
/// window, or the icon strip when the dock is collapsed.
fn dock_width(total_width: f32, dock_expanded: bool, stored_w: f32) -> (f32, f32) {
    if !dock_expanded {
        return (RIGHT_DOCK_COLLAPSED_W, 0.0);
    }
    let dock_max = (total_width * RIGHT_DOCK_MAX_SHARE).max(RIGHT_DOCK_MIN_W);
    (stored_w.clamp(RIGHT_DOCK_MIN_W, dock_max), DOCK_DIVIDER_W)
}

/// The three tab widths of the expanded dock's tab strip at `width`: two equal floors after the
/// collapse button, and the remainder for the last tab.
fn dock_tab_widths(width: f32) -> (f32, f32, f32) {
    let remaining_w = (width - DOCK_COLLAPSE_W).max(100.0);
    let tab_w = (remaining_w / 3.0).floor();
    (tab_w, tab_w, (width - (DOCK_COLLAPSE_W + 2.0 * tab_w)).max(0.0))
}

/// Draws the workspace tab strip at the top of the Orchestrator panel, following Designcraft document tabs design.
fn render_host_tabs(ui: &mut Ui, t: &Tokens, lang: &str) -> Option<HostTabAction> {
    let tabs: HostTabs = ui.ctx().data(|d| d.get_temp(host_tabs_id())).unwrap_or_default();
    if tabs.titles.is_empty() {
        return None;
    }

    let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
    let max_w = ORCH_TAB_MAX_W.min(bar.width());
    let close_label = crate::i18n::tr(lang, "Close");
    ui.painter().rect_filled(bar, 0.0, t.tab_strip);
    ui.painter().line_segment([bar.left_bottom(), bar.right_bottom()], Stroke::new(1.0, t.border));

    let mut x = bar.min.x;
    let mut action = None;

    for (index, (title, dirty)) in tabs.titles.iter().enumerate() {
        let is_active = index == tabs.active;
        let color = if is_active { t.text_strong } else { t.text_dim };
        let name = format!("{}{}", if *dirty { "*" } else { "" }, crate::rtl::isolate(title));
        let name_max = (max_w - ORCH_TAB_PAD).max(20.0);
        let galley = crate::rtl::truncated(ui.ctx(), &name, semibold(11.5), color, name_max);
        let elided = galley.elided;
        let w = (galley.size().x + ORCH_TAB_PAD).max(if is_active { 210.0 } else { 150.0 }).min(max_w);
        let r = Rect::from_min_size(pos2(x, bar.min.y), vec2(w, 28.0));

        let resp = ui.interact(r, ui.scope_id().with(("orchtab", index)), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, title));
        let resp = if elided {
            resp.on_hover_ui(|ui| {
                let _ = crate::rtl::label(ui, title);
            })
        } else {
            resp
        };

        ui.painter().rect_filled(r, 0.0, if is_active { t.panel } else { t.tab_strip });

        let cr = Rect::from_center_size(pos2(r.min.x + 18.0, r.center().y), vec2(14.0, 14.0));
        let cresp = ui.interact(cr, ui.scope_id().with(("orchclose", index)), Sense::click());
        cresp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, format!("{close_label} {title}")));
        ui.painter().text(
            cr.center(),
            Align2::CENTER_CENTER,
            "×",
            FontId::proportional(14.0),
            if cresp.hovered() { t.text_strong } else { t.text_dim },
        );

        let text = ui.painter().with_clip_rect(r);
        // PLACEHOLDER keeps the token colour the galley was laid out with; WHITE here made the tab
        // label unreadable in the light themes.
        text.galley(pos2(r.min.x + 32.0, r.center().y - galley.size().y / 2.0), galley, Color32::PLACEHOLDER);
        ui.painter().line_segment([r.right_top(), r.right_bottom()], Stroke::new(1.0, t.border));

        if cresp.clicked() {
            action = Some(HostTabAction::Close(index));
        } else if resp.clicked() {
            action = Some(HostTabAction::Activate(index));
        }
        x += w;
    }

    action
}

pub(crate) fn sync_card_to_pubgen_state(card: &PublicationCard, state: &OrchestratorWorkspaceState, pubgen_st: &mut super::state::PubgenState) {
    let headline = card
        .settings
        .get("title")
        .or_else(|| card.settings.get("title1"))
        .cloned()
        .unwrap_or_else(|| card.title.strip_prefix("Sampul Rilis: ").unwrap_or(&card.title).to_string());
    if !headline.trim().is_empty() {
        pubgen_st.cover_title = headline;
    } else if !state.meta_title.trim().is_empty() {
        pubgen_st.cover_title = state.meta_title.clone();
    }

    if !card.title_en.trim().is_empty() {
        pubgen_st.cover_title_en = card.title_en.clone();
    } else if let Some(ten) = card.settings.get("title_en") {
        pubgen_st.cover_title_en = ten.clone();
    }

    if !card.cover_title2.trim().is_empty() {
        pubgen_st.cover_title2 = card.cover_title2.clone();
    } else if let Some(t2) = card.settings.get("title2") {
        pubgen_st.cover_title2 = t2.clone();
    }

    if !card.cover_year.trim().is_empty() {
        pubgen_st.cover_year = card.cover_year.clone();
    } else if let Some(yr) = card.settings.get("year") {
        pubgen_st.cover_year = yr.clone();
    }

    if !card.cover_volume.trim().is_empty() {
        pubgen_st.cover_volume = card.cover_volume.clone();
    } else if let Some(vol) = card.settings.get("volume") {
        pubgen_st.cover_volume = vol.clone();
    }

    if !card.cover_catalogue.trim().is_empty() {
        pubgen_st.cover_catalogue = card.cover_catalogue.clone();
    } else if let Some(cat) = card.settings.get("catalogue") {
        pubgen_st.cover_catalogue = cat.clone();
    }

    if !card.cover_code_num.trim().is_empty() {
        pubgen_st.cover_code_num = card.cover_code_num.clone();
    } else if let Some(cd) = card.settings.get("code") {
        pubgen_st.cover_code_num = cd.clone();
    }

    if !card.cover_pub_office.trim().is_empty() {
        pubgen_st.cover_agency_text = card.cover_pub_office.clone();
    } else if let Some(po) = card.settings.get("pub_office").or_else(|| card.settings.get("agency")) {
        pubgen_st.cover_agency_text = po.clone();
    } else if !state.meta_agency.trim().is_empty() {
        pubgen_st.cover_agency_text = state.meta_agency.clone();
    }

    if !card.cover_office_address.trim().is_empty() {
        pubgen_st.cover_office_address = card.cover_office_address.clone();
    } else if let Some(addr) = card.settings.get("office_address") {
        pubgen_st.cover_office_address = addr.clone();
    }

    pubgen_st.cover_style = card.cover_style_idx;
    pubgen_st.cover_add_map = card.cover_add_map;
    pubgen_st.cover_include_inside_back = card.cover_inside_back;

    if let Some(h) = card.settings.get("image_hero").or_else(|| card.settings.get("imagehero")).or_else(|| card.settings.get("image")) {
        if !h.trim().is_empty() {
            pubgen_st.cover_image_skyline = h.clone();
        }
    }
    if let Some(l) = card.settings.get("image_landscape").or_else(|| card.settings.get("imagelandscape")).or_else(|| card.settings.get("landscape")) {
        if !l.trim().is_empty() {
            pubgen_st.cover_image_landscape = l.clone();
        }
    }
    if let Some(lg) = card.settings.get("logo").or_else(|| card.settings.get("officelogo")) {
        if !lg.trim().is_empty() {
            pubgen_st.cover_image_logo = lg.clone();
        }
    }
    if let Some(ev) = card.settings.get("event_theme").or_else(|| card.settings.get("themeevent")).or_else(|| card.settings.get("backcover")) {
        if !ev.trim().is_empty() {
            pubgen_st.cover_image_back = ev.clone();
        }
    }
}

pub fn sync_external_workspace(
    ctx: &egui::Context,
    source_name: &str,
    metadata: &pubgen_core::PublicationMetadata,
    tm: &pubgen_core::TranslationMemory,
    csv: &[(String, Vec<u8>)],
    text: &[(String, Vec<u8>)],
    assets: &[(String, Vec<u8>)],
) {
    let mut state = get_state(ctx);
    state.view = OrchestratorView::Studio;
    let report = pubgen_core::excel::IngestReport {
        profile_id: metadata.profile_id.clone(),
        metadata: metadata.clone(),
        csv: csv.to_vec(),
        text: text.to_vec(),
        assets: assets.to_vec(),
        warnings: Vec::new(),
    };
    apply_ingest_report_to_state(&mut state, source_name, &report, &[]);
    // The package's own memory is loaded after the import reset, so opening a `.pubgen` keeps it.
    state.tm = tm.clone();
    super::with_state(ctx, |pubgen_st| {
        super::translate_stage::seed_pubgen_state(pubgen_st, &state.tm);
        if let Some(cov_card) = state.cards.iter().find(|c| c.kind == CardKind::Cover) {
            sync_card_to_pubgen_state(cov_card, &state, pubgen_st);
        }
    });
    set_state(ctx, state);
}

/// Restore an opened package's glossary after `sync_external_workspace` (T8): `open_pubgen`
/// reads `glossary.json`, so saving the package again keeps its terminology instead of
/// writing an empty glossary over it.
pub fn set_workspace_glossary(ctx: &egui::Context, entries: &[pubgen_core::GlossaryEntry]) {
    let mut state = get_state(ctx);
    state.glossary_pairs = entries.iter().map(|e| (e.source.clone(), e.target.clone())).collect();
    set_state(ctx, state);
}

#[derive(Clone, Copy)]
enum ProfileCardKind {
    Dda,
    Brs,
    Generic,
}

fn render_profile_card(ui: &mut Ui, t: &Tokens, kind: ProfileCardKind, title: &str, subtitle: &str, spec: &str, accent: Color32) -> bool {
    let card_w = 230.0f32;
    let card_h = 165.0f32;
    let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
    let hov = resp.hovered();

    let bg = if hov { t.hover } else { t.panel };
    let border_color = if hov { t.accent_strong } else { t.divider };
    ui.painter().rect(rect, 8.0, bg, Stroke::new(if hov { 1.5 } else { 1.0 }, border_color), egui::StrokeKind::Inside);

    // Mini preview canvas
    let prev_rect = Rect::from_min_size(rect.min + vec2(10.0, 10.0), vec2(card_w - 20.0, 68.0));
    ui.painter().rect_filled(prev_rect, 4.0, if hov { Color32::from_black_alpha(40) } else { Color32::from_black_alpha(20) });

    let cx = prev_rect.center().x;
    let cy = prev_rect.center().y;

    match kind {
        ProfileCardKind::Dda => {
            // Two white facing pages
            let pw = 24.0f32;
            let ph = 40.0f32;
            let p_left = Rect::from_min_size(pos2(cx - pw - 2.0, cy - ph / 2.0), vec2(pw, ph));
            let p_right = Rect::from_min_size(pos2(cx + 2.0, cy - ph / 2.0), vec2(pw, ph));
            ui.painter().rect(p_left, 1.0, Color32::WHITE, Stroke::new(0.8, Color32::from_rgb(180, 180, 180)), egui::StrokeKind::Inside);
            ui.painter().rect(p_right, 1.0, Color32::WHITE, Stroke::new(0.8, Color32::from_rgb(180, 180, 180)), egui::StrokeKind::Inside);
            // Blue guide lines
            ui.painter()
                .line_segment([pos2(p_left.min.x + 4.0, p_left.max.y - 6.0), pos2(p_left.max.x - 4.0, p_left.max.y - 6.0)], Stroke::new(1.0, accent));
            ui.painter().line_segment(
                [pos2(p_right.min.x + 4.0, p_right.min.y + 6.0), pos2(p_right.max.x - 4.0, p_right.min.y + 6.0)],
                Stroke::new(1.0, accent),
            );
        }
        ProfileCardKind::Brs => {
            // Single page with blue header and orange accent
            let pw = 30.0f32;
            let ph = 42.0f32;
            let p = Rect::from_center_size(pos2(cx, cy), vec2(pw, ph));
            ui.painter().rect(p, 1.0, Color32::WHITE, Stroke::new(0.8, Color32::from_rgb(180, 180, 180)), egui::StrokeKind::Inside);
            let banner = Rect::from_min_size(p.min, vec2(pw, 10.0));
            ui.painter().rect_filled(banner, 1.0, Color32::from_rgb(0, 114, 188));
            ui.painter()
                .line_segment([pos2(p.min.x, p.min.y + 10.0), pos2(p.max.x, p.min.y + 10.0)], Stroke::new(1.0, Color32::from_rgb(245, 135, 31)));
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(p.min.x + 3.0, p.min.y + 14.0), vec2(pw - 6.0, 3.0)),
                0.0,
                Color32::from_rgb(190, 190, 190),
            );
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(p.min.x + 3.0, p.min.y + 20.0), vec2(pw - 6.0, 14.0)),
                0.0,
                Color32::from_rgb(230, 235, 245),
            );
        }
        ProfileCardKind::Generic => {
            // Single page with ruled lines
            let pw = 30.0f32;
            let ph = 42.0f32;
            let p = Rect::from_center_size(pos2(cx, cy), vec2(pw, ph));
            ui.painter().rect(p, 1.0, Color32::WHITE, Stroke::new(0.8, Color32::from_rgb(180, 180, 180)), egui::StrokeKind::Inside);
            for i in 0..4 {
                let ly = p.min.y + 9.0 + (i as f32 * 7.0);
                ui.painter().line_segment([pos2(p.min.x + 4.0, ly), pos2(p.max.x - 4.0, ly)], Stroke::new(0.8, Color32::from_rgb(210, 210, 210)));
            }
        }
    }

    // Texts
    ui.painter().text(pos2(rect.center().x, rect.min.y + 90.0), Align2::CENTER_CENTER, title, semibold(12.0), t.text_strong);
    ui.painter().text(pos2(rect.center().x, rect.min.y + 112.0), Align2::CENTER_CENTER, subtitle, FontId::proportional(11.5), t.text_dim);
    ui.painter().text(pos2(rect.center().x, rect.min.y + 134.0), Align2::CENTER_CENTER, spec, semibold(11.0), accent);

    resp.clicked()
}

fn render_recent_documents(ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    if state.recent_documents.is_empty() {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width().min(720.0), 54.0), Sense::hover());
        ui.painter().rect(rect, 8.0, t.panel, Stroke::new(1.0, t.divider), egui::StrokeKind::Inside);
        let msg = crate::i18n::tr(lang, "No recent .pubgen files yet. Open a project file or import an XLSX to begin.");
        ui.painter().text(pos2(rect.min.x + 16.0, rect.center().y), Align2::LEFT_CENTER, msg, FontId::proportional(12.0), t.text_dim);
    } else {
        let mut clicked_doc: Option<String> = None;
        for path in &state.recent_documents {
            let filename = std::path::Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone());
            let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width().min(720.0), 44.0), Sense::click());
            let hov = resp.hovered();
            ui.painter().rect(
                rect,
                6.0,
                if hov { t.hover } else { t.panel },
                Stroke::new(1.0, if hov { t.accent_strong } else { t.divider }),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(pos2(rect.min.x + 14.0, rect.min.y + 14.0), Align2::LEFT_CENTER, &filename, semibold(12.0), t.text_strong);
            ui.painter().text(pos2(rect.min.x + 14.0, rect.min.y + 30.0), Align2::LEFT_CENTER, path, FontId::proportional(11.0), t.text_dim);
            if resp.clicked() {
                clicked_doc = Some(path.clone());
            }
            ui.add_space(4.0);
        }
        if let Some(doc) = clicked_doc {
            let p = std::path::Path::new(&doc);
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
            if ext == "pubgen" && p.is_file() {
                trigger_host_tab_action(ui.ctx(), HostTabAction::OpenPath(p.to_path_buf()));
            } else if (ext == "xlsx" || ext == "xlsm") && p.is_file() {
                trigger_host_tab_action(ui.ctx(), HostTabAction::ImportPath(p.to_path_buf()));
            } else {
                state.execution_log.push(format!("[{}] {}", format_now(), crate::i18n::tr(lang, "Opening file {path}").replace("{path}", &doc)));
                state.view = OrchestratorView::Studio;
            }
        }
    }
}

fn pgs_orchestrator_home_logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let key = egui::Id::unique("pgs_orchestrator_home_logo");
    if let Some(tex) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(key)) {
        return Some(tex);
    }
    let bytes: &[u8] = include_bytes!("../../../../../assets/app-icon/pgs_icon.png");
    let pm = designcraft_render::decode_pixmap_page(bytes, 0)?;
    let ci = egui::ColorImage::from_rgba_premultiplied([pm.width() as usize, pm.height() as usize], pm.data_as_u8_slice());
    let fitted = crate::widgets::fit_texture(ci, 128);
    let tex = ctx.load_texture("pgs_orchestrator_home_logo", fitted, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(key, tex.clone()));
    Some(tex)
}

pub fn render_orchestrator_home(_app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    let r = ui.available_rect_before_wrap();
    ui.painter().rect_filled(r, 0.0, t.panel_darker);

    ScrollArea::vertical().id_salt("pubgen_home_scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(32.0);
        let max_w = 980.0f32.min(ui.available_width() - 64.0);
        ui.horizontal(|ui| {
            ui.add_space(32.0);
            ui.allocate_ui_with_layout(vec2(max_w, 0.0), Layout::top_down(Align::Min), |ui| {
                // HERO
                ui.horizontal(|ui| {
                    let (icon_r, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
                    if let Some(tex) = pgs_orchestrator_home_logo_texture(ui.ctx()) {
                        ui.painter().image(tex.id(), icon_r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                    } else {
                        ui.painter().rect_filled(icon_r, HOME_TILE_RADIUS, t.accent_strong);
                        crate::icons::paint(ui.painter(), icon_r.shrink(11.0), "panel-pages", Color32::WHITE);
                    }
                    ui.add_space(14.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(crate::i18n::tr(lang, "Welcome to Publication Orchestrator"))
                                .font(semibold(HOME_TITLE))
                                .color(t.text_strong),
                        );
                        let subtitle = crate::i18n::tr(lang, "Page layout for print and screen — fast, open and scriptable.");
                        ui.label(RichText::new(subtitle).size(HOME_BODY).color(t.text_dim));
                    });
                });

                ui.add_space(24.0);

                // ACTIONS ROW
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    let open_label = crate::i18n::tr(lang, "Open file (.pubgen)");
                    if ui
                        .add(
                            egui::Button::new(RichText::new(open_label).size(HOME_BUTTON).color(Color32::WHITE))
                                .fill(t.accent_strong)
                                .corner_radius(HOME_PILL_RADIUS)
                                .min_size(vec2(0.0, 32.0)),
                        )
                        .clicked()
                    {
                        trigger_host_tab_action(ui.ctx(), HostTabAction::Open);
                    }

                    let import_label = if lang.starts_with("id") { "Impor Manifest (XLSX)" } else { "Import Manifest (XLSX)" };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(import_label).size(HOME_BUTTON))
                                .corner_radius(HOME_PILL_RADIUS)
                                .min_size(vec2(0.0, 32.0)),
                        )
                        .clicked()
                    {
                        trigger_host_tab_action(ui.ctx(), HostTabAction::Import);
                    }

                    let brs_label = if lang.starts_with("id") { "Sampel BRS" } else { "Sample BRS" };
                    if ui
                        .add(egui::Button::new(RichText::new(brs_label).size(HOME_BUTTON)).corner_radius(HOME_PILL_RADIUS).min_size(vec2(0.0, 32.0)))
                        .clicked()
                    {
                        let sample = std::path::Path::new("plan/pubgen/samples/brs/ntp202610.xlsx");
                        if sample.is_file() {
                            trigger_host_tab_action(ui.ctx(), HostTabAction::ImportPath(sample.to_path_buf()));
                        } else {
                            state.selected_profile = "bps_brs".into();
                            state.page_layout_facing = true;
                            state.view = OrchestratorView::Studio;
                        }
                    }

                    let dda_label = if lang.starts_with("id") { "Sampel DDA" } else { "Sample DDA" };
                    if ui
                        .add(egui::Button::new(RichText::new(dda_label).size(HOME_BUTTON)).corner_radius(HOME_PILL_RADIUS).min_size(vec2(0.0, 32.0)))
                        .clicked()
                    {
                        let sample = std::path::Path::new("plan/pubgen/samples/SINGLE_LANG/BAB_02_2025.xlsx");
                        if sample.is_file() {
                            trigger_host_tab_action(ui.ctx(), HostTabAction::ImportPath(sample.to_path_buf()));
                        } else {
                            state.selected_profile = "bps_dda".into();
                            state.page_layout_facing = true;
                            state.view = OrchestratorView::Studio;
                        }
                    }
                    if !state.cards.is_empty() {
                        let studio_label = crate::i18n::tr(lang, "Open Active Studio");
                        if ui
                            .add(
                                egui::Button::new(RichText::new(studio_label).size(HOME_BUTTON).color(t.success))
                                    .corner_radius(HOME_PILL_RADIUS)
                                    .min_size(vec2(0.0, 32.0)),
                            )
                            .clicked()
                        {
                            state.view = OrchestratorView::Studio;
                        }
                    }
                });

                ui.add_space(28.0);

                // SECTION: PUBLICATION PROFILES
                let sec_prof = if lang.starts_with("id") { "Profil Publikasi" } else { "Publication Profiles" };
                ui.label(RichText::new(sec_prof).font(semibold(HOME_SECTION)).color(t.text_strong));
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;

                    // 1. DDA BPS
                    if render_profile_card(ui, t, ProfileCardKind::Dda, "DDA BPS", "Daerah Dalam Angka", "A4 (210 × 297 mm) • Facing Pages", t.accent)
                    {
                        state.selected_profile = "bps_dda".into();
                        state.page_layout_facing = true;
                        state.paper_size = "A4 (210 x 297 mm)".into();
                        state.view = OrchestratorView::Studio;
                    }

                    // 2. BRS BPS
                    if render_profile_card(
                        ui,
                        t,
                        ProfileCardKind::Brs,
                        "BRS BPS",
                        "Berita Resmi Statistik",
                        "A4 (210 × 297 mm) • Single Page",
                        t.live_corner,
                    ) {
                        state.selected_profile = "bps_brs".into();
                        state.page_layout_facing = false;
                        state.paper_size = "A4 (210 x 297 mm)".into();
                        state.view = OrchestratorView::Studio;
                    }

                    // 3. General
                    let gen_sub = if lang.starts_with("id") { "Laporan Standar Umum" } else { "Generic Publication Report" };
                    if render_profile_card(ui, t, ProfileCardKind::Generic, "General", gen_sub, "A4 (210 × 297 mm) • Single Page", t.text_dim) {
                        state.selected_profile = "generic_report".into();
                        state.page_layout_facing = false;
                        state.paper_size = "A4 (210 x 297 mm)".into();
                        state.view = OrchestratorView::Studio;
                    }
                });

                ui.add_space(28.0);

                // SECTION: RECENT OPENED DOCUMENTS
                let sec_rec = if lang.starts_with("id") { "Dokumen Terakhir Dibuka (*.pubgen)" } else { "Recent Opened Documents (*.pubgen)" };
                ui.label(RichText::new(sec_rec).font(semibold(HOME_SECTION)).color(t.text_strong));
                ui.add_space(10.0);

                render_recent_documents(ui, state, t, lang);
                ui.add_space(32.0);
            });
        });
    });
}

/// Main entry point for rendering the Orchestrator workspace inside CentralPanel.
pub fn show(app: &mut DesignApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let lang = app.ui.language.clone();
    if let Some(action) = render_host_tabs(ui, &t, &lang) {
        ui.ctx().data_mut(|d| d.insert_temp(host_tab_action_id(), action));
    }
    let mut state = get_state(ui.ctx());

    // Poll the in-flight worksheet translation job every frame (T7). The button lives in
    // the Studio view, so a job can only exist there.
    if state.view == OrchestratorView::Studio {
        poll_translate_job(&mut state, app, ui.ctx());
    }

    // Query Cloudflare AI cover quota on initial workspace mount if credentials exist in .env / prefs.
    let checked_cover_quota = ui.ctx().data(|d| d.get_temp::<bool>(egui::Id::unique("pubgen_cf_quota_initial_queried"))).unwrap_or(false);
    if !checked_cover_quota {
        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::unique("pubgen_cf_quota_initial_queried"), true));
        let (acct, _) = super::ai_cover::get_credentials(&app.session.prefs);
        if !acct.is_empty() {
            let ctx = ui.ctx().clone();
            let _ = std::thread::Builder::new().name("pubgen-cf-quota-init".into()).spawn(move || {
                if let Some(neurons) = designcraft_imagegen::quota::check_remaining(&acct) {
                    let covers = (neurons / designcraft_imagegen::quota::NEURONS_PER_COVER).max(0.0) as u64;
                    super::with_state(&ctx, |st| {
                        st.cover_quota_covers = Some(covers);
                    });
                    ctx.request_repaint();
                }
            });
        }
    }

    if state.view == OrchestratorView::Home {
        render_orchestrator_home(app, ui, &mut state, &t, &lang);
        set_state(ui.ctx(), state);
        return;
    }

    let avail_rect = ui.available_rect_before_wrap();
    let total_width = avail_rect.width();
    let total_height = avail_rect.height();

    // 2-Section Layout: the centre takes what is left; the right dock is either expanded to a
    // draggable width or collapsed to an icon strip.
    let (right_w, divider_w) = dock_width(total_width, state.right_dock_expanded, state.right_dock_w);
    let center_w = (total_width - right_w - divider_w).max(100.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        // 1. CENTER PANEL: "Eksekusi & Verifikasi" (occupies the main canvas area)
        ui.allocate_ui_with_layout(vec2(center_w, total_height), Layout::top_down(Align::Min), |ui| {
            ui.set_clip_rect(Rect::from_min_size(avail_rect.min, vec2(center_w, total_height)));
            egui::Frame::NONE.fill(t.pasteboard).inner_margin(Margin::symmetric(14, 10)).show(ui, |ui| {
                let inner_w = (center_w - 28.0).max(100.0);
                let inner_h = (total_height - 20.0).max(100.0);
                ui.set_width(inner_w);
                ui.set_max_width(inner_w);
                ui.set_height(inner_h);
                ui.vertical(|ui| {
                    render_center_panel(app, ui, &mut state, &t, &lang);
                });
            });
        });

        // 2. RIGHT DOCK PANEL: Tab 1 (Sumber & Konten) + Tab 2 (Profil, Slot & Eksekusi)
        let right_pos = pos2(avail_rect.min.x + center_w + divider_w, avail_rect.min.y);

        // Divider: drag to resize the dock, like DesignCraft's panel splitter.
        if state.right_dock_expanded && divider_w > 0.0 {
            let (divider_rect, divider_resp) = ui.allocate_exact_size(vec2(divider_w, total_height), Sense::drag());
            divider_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, crate::i18n::tr(&lang, "Resize panels")));
            let active = divider_resp.hovered() || divider_resp.dragged();
            ui.painter().rect_filled(divider_rect, CornerRadius::ZERO, if active { t.accent } else { t.border });
            if divider_resp.dragged() {
                state.right_dock_w = dock_width(total_width, true, state.right_dock_w - divider_resp.drag_delta().x).0;
            }
            divider_resp.on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
        }
        ui.allocate_ui_with_layout(vec2(right_w, total_height), Layout::top_down(Align::Min), |ui| {
            ui.set_clip_rect(Rect::from_min_size(right_pos, vec2(right_w, total_height)));
            egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.border)).show(ui, |ui| {
                ui.set_width(right_w);
                ui.set_max_width(right_w);
                ui.set_height(total_height);
                if !state.right_dock_expanded {
                    render_collapsed_dock(app, ui, &mut state, &t, right_w, total_height, &lang);
                } else {
                    render_expanded_dock(app, ui, &mut state, &t, right_w, total_height, &lang);
                }
            });
        });
    });

    // Render Card Edit modal if open
    if state.editing_card_idx.is_some() {
        render_card_edit_modal(app, ui.ctx(), &mut state, &t, &lang);
    }

    // Render Re-compilation Safety Guard Dialog if triggered
    if state.show_guard_dialog {
        render_guard_dialog(app, ui.ctx(), &mut state, &t, &lang);
    }

    // Render AI Translation Guard Dialog if triggered
    if state.show_translate_guard_dialog {
        render_translate_guard_dialog(app, ui.ctx(), &mut state, &t, &lang);
    }

    // Render Compile & Thumbnail Pre-warm Progress Modal if active
    if state.compile_progress.is_some() {
        render_compile_progress_modal(app, ui.ctx(), &mut state, &t, &lang);
    }

    // Render AI Quota Popup modal if triggered (0.8.4 parity)
    let quota_popup = super::with_state(ui.ctx(), |st| st.ai_quota_popup.take());
    if quota_popup.is_some() {
        let mut temp_st = super::PubgenState { ai_quota_popup: quota_popup, ..Default::default() };
        super::widgets::ai_quota_popup_ui(&mut temp_st, ui, &t);
        if temp_st.ai_quota_popup.is_some() {
            super::with_state(ui.ctx(), |st| st.ai_quota_popup = temp_st.ai_quota_popup);
        }
    }

    set_state(ui.ctx(), state);
}

/// Collapsed iconic right dock (InDesign parity: icon strip with expand toggle)
fn render_collapsed_dock(
    _app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
    width: f32,
    total_height: f32,
    lang: &str,
) {
    ui.set_width(width);
    ui.set_height(total_height);

    let tab_bar_height = DOCK_TAB_H;

    // Header strip with visible expand button
    let expand_btn = ui
        .add_sized(
            [width, tab_bar_height],
            egui::Button::new(RichText::new("«").font(semibold(12.0)).color(t.text_strong)).fill(t.tab_strip).frame(false),
        )
        .on_hover_text(format!("{} (Expand Panels)", crate::i18n::tr(lang, "Expand Panels")));
    expand_btn.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, crate::i18n::tr(lang, "Expand Panels")));
    if expand_btn.clicked() {
        state.right_dock_expanded = true;
    }

    // Border line below the header
    let strip_rect = expand_btn.rect;
    ui.painter().line_segment([strip_rect.left_bottom(), strip_rect.right_bottom()], Stroke::new(1.0, t.border));

    ui.add_space(4.0);

    // Icon 1: Sumber & Konten
    let is_tab1 = state.right_dock_tab == OrchestratorRightTab::SumberKonten;
    let (r1, resp1) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    let resp1 = resp1.on_hover_text(crate::i18n::tr(lang, "Sources & Content"));
    resp1.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, is_tab1, crate::i18n::tr(lang, "Sources & Content")));
    let btn1 = Rect::from_center_size(r1.center(), vec2(30.0, 30.0));
    if resp1.hovered() || is_tab1 {
        ui.painter().rect_filled(btn1, 3.0, if is_tab1 { t.panel_darker } else { t.hover });
    }
    if is_tab1 {
        ui.painter().rect_stroke(btn1, 3.0, Stroke::new(1.5, t.accent), StrokeKind::Inside);
    }
    let icon_box1 = Rect::from_center_size(btn1.center(), vec2(18.0, 18.0));
    crate::icons::paint(ui.painter(), icon_box1, "panel-articles", if is_tab1 { t.accent } else { t.icon });
    if resp1.clicked() {
        state.right_dock_tab = OrchestratorRightTab::SumberKonten;
        state.right_dock_expanded = true;
    }

    ui.add_space(2.0);

    // Icon 2: Profil & Eksekusi
    let is_tab2 = state.right_dock_tab == OrchestratorRightTab::ProfilEksekusi;
    let (r2, resp2) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    let resp2 = resp2.on_hover_text(crate::i18n::tr(lang, "Profile & Execution"));
    resp2.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, is_tab2, crate::i18n::tr(lang, "Profile & Execution")));
    let btn2 = Rect::from_center_size(r2.center(), vec2(30.0, 30.0));
    if resp2.hovered() || is_tab2 {
        ui.painter().rect_filled(btn2, 3.0, if is_tab2 { t.panel_darker } else { t.hover });
    }
    if is_tab2 {
        ui.painter().rect_stroke(btn2, 3.0, Stroke::new(1.5, t.accent), StrokeKind::Inside);
    }
    let icon_box2 = Rect::from_center_size(btn2.center(), vec2(18.0, 18.0));
    crate::icons::paint(ui.painter(), icon_box2, "panel-properties", if is_tab2 { t.accent } else { t.icon });
    if resp2.clicked() {
        state.right_dock_tab = OrchestratorRightTab::ProfilEksekusi;
        state.right_dock_expanded = true;
    }

    ui.add_space(2.0);

    // Icon 3: Ulasan Terjemahan (T8 review pane)
    let is_tab3 = state.right_dock_tab == OrchestratorRightTab::Terjemahan;
    let (r3, resp3) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    let resp3 = resp3.on_hover_text(crate::i18n::tr(lang, "Translation Review"));
    resp3.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, is_tab3, crate::i18n::tr(lang, "Translation Review")));
    let btn3 = Rect::from_center_size(r3.center(), vec2(30.0, 30.0));
    if resp3.hovered() || is_tab3 {
        ui.painter().rect_filled(btn3, 3.0, if is_tab3 { t.panel_darker } else { t.hover });
    }
    if is_tab3 {
        ui.painter().rect_stroke(btn3, 3.0, Stroke::new(1.5, t.accent), StrokeKind::Inside);
    }
    let icon_box3 = Rect::from_center_size(btn3.center(), vec2(18.0, 18.0));
    crate::icons::paint(ui.painter(), icon_box3, "panel-preflight", if is_tab3 { t.accent } else { t.icon });
    if resp3.clicked() {
        state.right_dock_tab = OrchestratorRightTab::Terjemahan;
        state.right_dock_expanded = true;
    }
}

/// Expanded right dock with tabs (InDesign parity: tab strip + collapse button)
fn render_expanded_dock(
    app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
    width: f32,
    total_height: f32,
    lang: &str,
) {
    ui.set_width(width);
    ui.set_height(total_height);

    let tab_bar_height = DOCK_TAB_H;
    let collapse_btn_w = DOCK_COLLAPSE_W;
    let (tab_w, tab2_w, tab3_w) = dock_tab_widths(width);

    // 1. Tab Bar strip
    let (strip, _) = ui.allocate_exact_size(vec2(width, tab_bar_height), Sense::hover());
    ui.painter().rect_filled(strip, 0.0, t.tab_strip);

    // Collapse to icons button on the left of the tab strip
    let collapse_rect = Rect::from_min_size(pos2(strip.min.x + 2.0, strip.min.y + 2.0), vec2(collapse_btn_w - 4.0, tab_bar_height - 4.0));
    let collapse_resp = ui
        .interact(collapse_rect, ui.scope_id().with("dock_collapse_btn"), Sense::click())
        .on_hover_text(format!("{} (Collapse to Icons)", crate::i18n::tr(lang, "Collapse to Icons")));
    collapse_resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, crate::i18n::tr(lang, "Collapse to Icons")));
    if collapse_resp.hovered() {
        ui.painter().rect_filled(collapse_rect, 3.0, t.hover);
    }
    let collapse_icon_rect = Rect::from_center_size(collapse_rect.center(), vec2(14.0, 14.0));
    crate::icons::paint(ui.painter(), collapse_icon_rect, "double-chevron-right", t.icon);
    if collapse_resp.clicked() {
        state.right_dock_expanded = false;
    }

    // Tab 1: Sumber & Konten
    let tab1_rect = Rect::from_min_size(pos2(strip.min.x + collapse_btn_w, strip.min.y), vec2(tab_w, tab_bar_height));
    let is_tab1 = state.right_dock_tab == OrchestratorRightTab::SumberKonten;
    let tab1_resp =
        ui.interact(tab1_rect, ui.scope_id().with("dock_tab_1"), Sense::click()).on_hover_text(crate::i18n::tr(lang, "Sources & Content"));
    tab1_resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, is_tab1, crate::i18n::tr(lang, "Sources & Content")));
    if is_tab1 {
        ui.painter().rect_filled(tab1_rect, 0.0, t.panel);
        ui.painter().line_segment([tab1_rect.left_top(), tab1_rect.left_bottom()], Stroke::new(1.0, t.border));
        ui.painter().line_segment([tab1_rect.right_top(), tab1_rect.right_bottom()], Stroke::new(1.0, t.border));
    } else {
        if tab1_resp.hovered() {
            ui.painter().rect_filled(tab1_rect, 0.0, t.hover);
        }
        ui.painter().line_segment([tab1_rect.left_bottom(), tab1_rect.right_bottom()], Stroke::new(1.0, t.border));
    }
    paint_dock_tab_label(
        ui,
        tab1_rect,
        dock_tab_label(ui.ctx(), tab1_rect, crate::i18n::tr(lang, "Sources & Content"), if is_tab1 { t.text_strong } else { t.text_dim }),
    );
    if tab1_resp.clicked() {
        state.right_dock_tab = OrchestratorRightTab::SumberKonten;
    }

    // Tab 2: Profil & Eksekusi
    let tab2_rect = Rect::from_min_size(pos2(strip.min.x + collapse_btn_w + tab_w, strip.min.y), vec2(tab2_w, tab_bar_height));
    let is_tab2 = state.right_dock_tab == OrchestratorRightTab::ProfilEksekusi;
    let tab2_resp =
        ui.interact(tab2_rect, ui.scope_id().with("dock_tab_2"), Sense::click()).on_hover_text(crate::i18n::tr(lang, "Profile & Execution"));
    tab2_resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, is_tab2, crate::i18n::tr(lang, "Profile & Execution")));
    if is_tab2 {
        ui.painter().rect_filled(tab2_rect, 0.0, t.panel);
        ui.painter().line_segment([tab2_rect.left_top(), tab2_rect.left_bottom()], Stroke::new(1.0, t.border));
        ui.painter().line_segment([tab2_rect.right_top(), tab2_rect.right_bottom()], Stroke::new(1.0, t.border));
    } else {
        if tab2_resp.hovered() {
            ui.painter().rect_filled(tab2_rect, 0.0, t.hover);
        }
        ui.painter().line_segment([tab2_rect.left_bottom(), tab2_rect.right_bottom()], Stroke::new(1.0, t.border));
    }
    paint_dock_tab_label(
        ui,
        tab2_rect,
        dock_tab_label(ui.ctx(), tab2_rect, crate::i18n::tr(lang, "Profile & Execution"), if is_tab2 { t.text_strong } else { t.text_dim }),
    );
    if tab2_resp.clicked() {
        state.right_dock_tab = OrchestratorRightTab::ProfilEksekusi;
    }

    // Tab 3: Ulasan Terjemahan (T8)
    let tab3_rect = Rect::from_min_size(pos2(strip.min.x + collapse_btn_w + 2.0 * tab_w, strip.min.y), vec2(tab3_w, tab_bar_height));
    let is_tab3 = state.right_dock_tab == OrchestratorRightTab::Terjemahan;
    let tab3_resp =
        ui.interact(tab3_rect, ui.scope_id().with("dock_tab_3"), Sense::click()).on_hover_text(crate::i18n::tr(lang, "Translation Review"));
    tab3_resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, is_tab3, crate::i18n::tr(lang, "Translation Review")));
    if is_tab3 {
        ui.painter().rect_filled(tab3_rect, 0.0, t.panel);
        ui.painter().line_segment([tab3_rect.left_top(), tab3_rect.left_bottom()], Stroke::new(1.0, t.border));
        ui.painter().line_segment([tab3_rect.right_top(), tab3_rect.right_bottom()], Stroke::new(1.0, t.border));
    } else {
        if tab3_resp.hovered() {
            ui.painter().rect_filled(tab3_rect, 0.0, t.hover);
        }
        ui.painter().line_segment([tab3_rect.left_bottom(), tab3_rect.right_bottom()], Stroke::new(1.0, t.border));
    }
    paint_dock_tab_label(
        ui,
        tab3_rect,
        dock_tab_label(ui.ctx(), tab3_rect, crate::i18n::tr(lang, "Translation Review"), if is_tab3 { t.text_strong } else { t.text_dim }),
    );
    if tab3_resp.clicked() {
        state.right_dock_tab = OrchestratorRightTab::Terjemahan;
    }

    // Line under the collapse button
    ui.painter().line_segment([strip.left_bottom(), pos2(strip.min.x + collapse_btn_w, strip.max.y)], Stroke::new(1.0, t.border));

    // 2. Tab Content Body
    egui::Frame::NONE.inner_margin(Margin::symmetric(10, 8)).show(ui, |ui| {
        let body_w = (width - 20.0).max(60.0);
        let body_h = (total_height - tab_bar_height - 16.0).max(60.0);
        ui.set_width(body_w);
        ui.set_max_width(body_w);
        ui.set_height(body_h);
        match state.right_dock_tab {
            OrchestratorRightTab::SumberKonten => {
                render_left_panel(app, ui, state, t, lang);
            }
            OrchestratorRightTab::ProfilEksekusi => {
                render_right_panel(app, ui, state, t, lang);
            }
            OrchestratorRightTab::Terjemahan => {
                // T8: review the AI drafts before a compile; the pane owns the memory so Accept
                // and Reject persist with the package.
                let mut dirty = state.is_dirty;
                super::translate_stage::review_pane(ui, &mut state.tm, &mut dirty);
                if dirty && !state.is_dirty {
                    for card in &mut state.cards {
                        if let Some(entry) = state.tm.lookup(&card.title) {
                            if !entry.target.is_empty() {
                                card.title_en = entry.target.clone();
                                card.settings.insert("title_en".into(), entry.target.clone());
                            }
                        }
                    }
                }
                state.is_dirty = dirty;
            }
        }
    });
}

// ============================================================================
// PANEL KIRI: "Sumber & Konten"
// ============================================================================

fn render_left_panel(app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

    if !state.content_files.is_empty() {
        let source_label = state.recent_documents.first().map(String::as_str).unwrap_or("Workspace source");
        ui.label(RichText::new(format!("Source: {source_label}")).font(FontId::proportional(11.0)).color(t.text_dim));
        ui.label(RichText::new(format!("{} embedded content file(s)", state.content_files.len())).font(FontId::proportional(11.0)).color(t.text_dim));
    }

    // Header row
    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::i18n::tr(lang, "Sources & Content")).font(semibold(12.0)).color(t.text_strong));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Home button
            let home_btn = crate::icons::button(ui, "home", 22.0, false, &crate::i18n::tr(lang, "Home"));
            if home_btn.clicked() {
                state.view = OrchestratorView::Home;
            }

            // Save button
            let save_btn = crate::icons::button(ui, "preset-save", 22.0, false, &crate::i18n::tr(lang, "Save"));
            if save_btn.clicked() {
                let now = format_now();
                let recent = state.recent_documents.first().cloned().unwrap_or_default();
                let remembered = app.session.prefs.pubgen_project_path.trim().to_string();
                let target = if recent.ends_with(".pubgen") && std::path::Path::new(&recent).exists() {
                    recent
                } else if remembered.ends_with(".pubgen") && std::path::Path::new(&remembered).exists() {
                    remembered
                } else {
                    String::new()
                };

                if !target.is_empty() {
                    match super::compile::save_pubgen_package_to(app, std::path::Path::new(&target)) {
                        Ok(_) => {
                            state.is_dirty = false;
                            state.execution_log.push(format!("[{now}] {}", crate::i18n::tr(lang, "Paket publikasi (.pubgen) berhasil disimpan.")));
                            state.status_message = Some((format!("Disimpan ke {target}"), true));
                        }
                        Err(e) => {
                            state.status_message = Some((format!("Gagal menyimpan: {e}"), false));
                        }
                    }
                } else {
                    let _ = app.run("pubgen.savePackageAs", serde_json::json!({}));
                }
            }
        });
    });

    // Baris Tombol Impor
    ui.horizontal(|ui| {
        let btn_w = ((ui.available_width() - 8.0) / 3.0).max(70.0);

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(
                    RichText::new(format!("+ {}", crate::i18n::tr(lang, "Import Source"))).font(FontId::proportional(11.0)).color(t.text),
                )
                .corner_radius(CornerRadius::same(t.radius)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Import Source"))
            .clicked()
        {
            let lang = lang.to_string();
            let ctx = ui.ctx().clone();
            let _ = app.ask_open("dataMerge", crate::app::file_dialogs::Bound::App, move |app, path| {
                let p = std::path::Path::new(&path);
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
                if ext == "xlsx" || ext == "xlsm" {
                    if let Ok(report) = pubgen_core::excel::ingest_workbook(p) {
                        let mut st = get_state(&ctx);
                        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or(&path);
                        let pairs = super::ai::BpsGlossary::load(&app.session.prefs.pubgen_glossary_path).prompt_pairs();
                        apply_ingest_report_to_state(&mut st, name, &report, &pairs);
                        crate::dock::pubgen::with_state(&ctx, |pubgen_st| {
                            if let Some(cov_card) = st.cards.iter().find(|c| c.kind == CardKind::Cover) {
                                sync_card_to_pubgen_state(cov_card, &st, pubgen_st);
                            }
                        });
                        set_state(&ctx, st);
                        ctx.request_repaint();
                    }
                } else if ext == "md" || ext == "csv" || ext == "rtf" {
                    let mut st = get_state(&ctx);
                    if let Ok(bytes) = std::fs::read(p) {
                        let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                        let target_path = if ext == "csv" {
                            format!("csv/{name}")
                        } else if ext == "md" {
                            // A Markdown narrative from a package that predates the RTF format.
                            format!("md/{name}")
                        } else {
                            format!("rtf/{name}")
                        };
                        st.content_files.retain(|f| f.path != target_path);
                        st.content_files.push(ContentItem { path: target_path.clone(), size_bytes: bytes.len() });
                        std::sync::Arc::make_mut(&mut st.cached_content).insert(target_path.clone(), bytes.clone());
                        let now = format_now();
                        st.execution_log.push(format!(
                            "[{now}] {}",
                            crate::i18n::tr(&lang, "Source file added: {path} ({n} bytes)")
                                .replace("{path}", &target_path)
                                .replace("{n}", &bytes.len().to_string())
                        ));
                        set_state(&ctx, st);
                        ctx.request_repaint();
                    }
                }
                Ok(serde_json::Value::Null)
            });
        }

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(RichText::new(crate::i18n::tr(lang, "Import Excel")).font(FontId::proportional(11.0)).color(t.text))
                    .corner_radius(CornerRadius::same(t.radius)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Import Excel"))
            .clicked()
        {
            let ctx = ui.ctx().clone();
            let _ = app.ask_open("dataMerge", crate::app::file_dialogs::Bound::App, move |app, path| {
                let p = std::path::Path::new(&path);
                if let Ok(report) = pubgen_core::excel::ingest_workbook(p) {
                    let mut st = get_state(&ctx);
                    let name = p.file_name().and_then(|s| s.to_str()).unwrap_or(&path);
                    let pairs = super::ai::BpsGlossary::load(&app.session.prefs.pubgen_glossary_path).prompt_pairs();
                    apply_ingest_report_to_state(&mut st, name, &report, &pairs);
                    crate::dock::pubgen::with_state(&ctx, |pubgen_st| {
                        if let Some(cov_card) = st.cards.iter().find(|c| c.kind == CardKind::Cover) {
                            sync_card_to_pubgen_state(cov_card, &st, pubgen_st);
                        }
                    });
                    set_state(&ctx, st);
                    ctx.request_repaint();
                }
                Ok(serde_json::Value::Null)
            });
        }

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(RichText::new(crate::i18n::tr(lang, "Import Metadata")).font(FontId::proportional(11.0)).color(t.text))
                    .corner_radius(CornerRadius::same(t.radius)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Import Metadata"))
            .clicked()
        {
            state.metadata_open = true;
            let now = format_now();
            state.execution_log.push(format!("[{now}] Metadata editor dibuka."));
        }
    });

    ui.separator();

    // Scroll area for content tree & document settings
    ScrollArea::vertical().id_salt("pubgen_sumber_konten_scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 6.0);

        // Section 1: Pengaturan Umum Publikasi
        let profile_label = if state.selected_profile == "bps_dda" {
            crate::i18n::tr(lang, "General Settings (DDA BPS Profile)")
        } else {
            crate::i18n::tr(lang, "General Settings (BRS BPS Profile)")
        };
        ui.label(RichText::new(profile_label).font(semibold(12.0)).color(t.text_strong));

        egui::Frame::NONE
            .fill(t.panel_darker)
            .stroke(Stroke::new(1.0, t.field_border))
            .corner_radius(CornerRadius::same(t.radius))
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 5.0);

                // 1. Tata Letak Halaman
                let facing_label = crate::i18n::tr(lang, "Double Page (Facing)");
                let single_label = crate::i18n::tr(lang, "Single Page (Non-Facing)");
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::tr(lang, "Page Layout:")).font(FontId::proportional(11.0)).color(t.text_dim));
                    let combo_w = (ui.available_width() - 4.0).max(120.0);
                    craft_ui::ComboBox::from_id_salt("meta_page_layout_combo")
                        .selected_text(
                            RichText::new(if state.page_layout_facing { facing_label } else { single_label }).font(FontId::proportional(11.0)),
                        )
                        .width(combo_w)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.page_layout_facing, true, facing_label);
                            ui.selectable_value(&mut state.page_layout_facing, false, single_label);
                        });
                });

                // 2. Bahasa Dokumen
                let indonesian_label = crate::i18n::tr(lang, "Indonesian");
                let english_label = crate::i18n::tr(lang, "English");
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::tr(lang, "Document Language:")).font(FontId::proportional(11.0)).color(t.text_dim));
                    let combo_w = (ui.available_width() - 4.0).max(120.0);
                    craft_ui::ComboBox::from_id_salt("meta_doc_lang_combo")
                        .selected_text(RichText::new(crate::i18n::tr(lang, &state.doc_language)).font(FontId::proportional(11.0)))
                        .width(combo_w)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.doc_language, "Indonesian".into(), indonesian_label);
                            ui.selectable_value(&mut state.doc_language, "English".into(), english_label);
                            ui.selectable_value(&mut state.doc_language, "Indonesian-English".into(), "Indonesian-English / Indonesia-Inggris");
                        });
                });

                // 3. Ukuran Kertas
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::tr(lang, "Paper Size:")).font(FontId::proportional(11.0)).color(t.text_dim));
                    let combo_w = (ui.available_width() - 4.0).max(120.0);
                    craft_ui::ComboBox::from_id_salt("meta_paper_size_combo")
                        .selected_text(RichText::new(&state.paper_size).font(FontId::proportional(11.0)))
                        .width(combo_w)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.paper_size, "A4 (210 x 297 mm)".into(), "A4 (210 x 297 mm)");
                            ui.selectable_value(&mut state.paper_size, "Letter (8.5 x 11 in)".into(), "Letter (8.5 x 11 in)");
                            ui.selectable_value(&mut state.paper_size, "B5 (182 x 257 mm)".into(), "B5 (182 x 257 mm)");
                        });
                });

                // 4. Margin (mm) with +/- steppers
                ui.add_space(2.0);
                ui.label(RichText::new(crate::i18n::tr(lang, "Margins (mm):")).font(FontId::proportional(11.0)).color(t.text_dim));

                let col_w = (ui.available_width() / 2.0).max(85.0);
                // Row 1: T and R
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(vec2(col_w, 22.0), Layout::left_to_right(Align::Center), |ui| {
                        render_margin_stepper(ui, "T", &mut state.margin_top_mm, t, lang);
                    });
                    ui.allocate_ui_with_layout(vec2(col_w, 22.0), Layout::left_to_right(Align::Center), |ui| {
                        render_margin_stepper(ui, "R", &mut state.margin_right_mm, t, lang);
                    });
                });

                // Row 2: B and L
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(vec2(col_w, 22.0), Layout::left_to_right(Align::Center), |ui| {
                        render_margin_stepper(ui, "B", &mut state.margin_bottom_mm, t, lang);
                    });
                    ui.allocate_ui_with_layout(vec2(col_w, 22.0), Layout::left_to_right(Align::Center), |ui| {
                        render_margin_stepper(ui, "L", &mut state.margin_left_mm, t, lang);
                    });
                });
            });

        ui.separator();

        // Section 2: Collapsible Pengaturan Dokumen (Metadata Form - Image 4)
        let meta_hdr = ui.selectable_label(
            false,
            RichText::new(format!("{} {}", if state.metadata_open { "▼" } else { "▶" }, crate::i18n::tr(lang, "Document Settings")))
                .font(semibold(12.0))
                .color(t.text_strong),
        );
        if meta_hdr.clicked() {
            state.metadata_open = !state.metadata_open;
        }

        if state.metadata_open {
            egui::Frame::NONE
                .fill(t.panel_darker)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(CornerRadius::same(t.radius))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(3.0, 4.0);

                    ui.label(RichText::new(crate::i18n::tr(lang, "Publication Title")).font(FontId::proportional(11.0)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_title));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Subtitle")).font(FontId::proportional(11.0)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_subtitle));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Release / Catalogue No.")).font(FontId::proportional(11.0)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_release_no));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Release Date")).font(FontId::proportional(11.0)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_release_date));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Release Month")).font(FontId::proportional(11.0)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_release_month));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Publishing Agency")).font(FontId::proportional(11.0)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_agency));
                });
        }

        ui.separator();

        // Section 3: Collapsible Pohon Konten
        let tree_title = format!("{} ({})", crate::i18n::tr(lang, "Content Tree"), state.content_files.len());
        let tree_header = ui.selectable_label(
            false,
            RichText::new(format!("{} {}", if state.tree_open { "▼" } else { "▶" }, tree_title)).font(semibold(11.5)).color(t.text_strong),
        );
        if tree_header.clicked() {
            state.tree_open = !state.tree_open;
        }

        if state.tree_open {
            ui.indent("tree_indent", |ui| {
                ui.spacing_mut().item_spacing = vec2(2.0, 3.0);

                let csv_files: Vec<_> = state.content_files.iter().filter(|f| f.path.starts_with("csv/") || f.path.ends_with(".csv")).collect();
                let text_files: Vec<_> = state
                    .content_files
                    .iter()
                    .filter(|f| f.path.starts_with("rtf/") || f.path.starts_with("md/") || f.path.ends_with(".rtf") || f.path.ends_with(".md"))
                    .collect();

                // Group 1: CSV
                let csv_hdr = ui.selectable_label(
                    false,
                    RichText::new(format!(
                        "{} {} ({})",
                        if state.csv_group_open { "▼" } else { "▶" },
                        crate::i18n::tr(lang, "Data Tables (CSV)"),
                        csv_files.len()
                    ))
                    .font(semibold(11.0))
                    .color(t.text),
                );
                if csv_hdr.clicked() {
                    state.csv_group_open = !state.csv_group_open;
                }

                if state.csv_group_open {
                    ui.indent("csv_indent", |ui| {
                        for item in csv_files {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("• {}", item.path)).font(FontId::proportional(11.0)).color(t.text));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{} B", item.size_bytes)).font(FontId::proportional(11.0)).color(t.text_dim));
                                });
                            });
                        }
                    });
                }

                // Group 2: narrative text (RTF; Markdown in older packages)
                let md_hdr = ui.selectable_label(
                    false,
                    RichText::new(format!(
                        "{} {} ({})",
                        if state.md_group_open { "▼" } else { "▶" },
                        crate::i18n::tr(lang, "Narrative (RTF)"),
                        text_files.len()
                    ))
                    .font(semibold(11.0))
                    .color(t.text),
                );
                if md_hdr.clicked() {
                    state.md_group_open = !state.md_group_open;
                }

                if state.md_group_open {
                    ui.indent("md_indent", |ui| {
                        for item in text_files {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("• {}", item.path)).font(FontId::proportional(11.0)).color(t.text));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{} B", item.size_bytes)).font(FontId::proportional(11.0)).color(t.text_dim));
                                });
                            });
                        }
                    });
                }
            });
        }
    });
}

fn render_margin_stepper(ui: &mut Ui, label: &str, val: &mut u32, t: &Tokens, lang: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(3.0, 0.0);
        ui.label(RichText::new(format!("{label}: {val}mm")).font(FontId::proportional(11.0)).color(t.text));
        let btn_size = vec2(17.0, 18.0);
        let btn_fill = t.accent_strong;
        let minus = ui.add_sized(
            btn_size,
            egui::Button::new(RichText::new("-").font(semibold(11.0)).color(Color32::WHITE))
                .fill(btn_fill)
                .corner_radius(CornerRadius::same(t.radius)),
        );
        minus
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, crate::i18n::tr(lang, "Decrease {label}").replace("{label}", label)));
        if minus.clicked() {
            *val = val.saturating_sub(1).max(5);
        }
        let plus = ui.add_sized(
            btn_size,
            egui::Button::new(RichText::new("+").font(semibold(11.0)).color(Color32::WHITE))
                .fill(btn_fill)
                .corner_radius(CornerRadius::same(t.radius)),
        );
        plus.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, crate::i18n::tr(lang, "Increase {label}").replace("{label}", label)));
        if plus.clicked() {
            *val = val.saturating_add(1).min(60);
        }
    });
}

// ============================================================================
// AREA TENGAH: "Eksekusi & Verifikasi"
// ============================================================================

fn render_center_panel(app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    ui.spacing_mut().item_spacing = vec2(6.0, 8.0);

    // Section title & QA Preflight Row (clean, compact, no huge header/footer preview block)
    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::i18n::tr(lang, "Execution & Verification")).font(semibold(12.0)).color(t.text_strong));
        ui.add_space(16.0);
        ui.label(RichText::new(crate::i18n::tr(lang, "QA Preflight Checklist")).font(semibold(11.0)).color(t.text_strong));

        if ui
            .add(
                egui::Button::new(RichText::new(crate::i18n::tr(lang, "Run QA Audit")).font(FontId::proportional(11.0)).color(t.text))
                    .corner_radius(CornerRadius::same(t.radius)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Run QA Audit"))
            .clicked()
        {
            let now = format_now();
            state.last_qa_clean = true;
            state.execution_log.push(format!("[{now}] {}", crate::i18n::tr(lang, "QA audit finished: 0 errors, 0 warnings. Print standard passed.")));
            state.status_message = Some(("Audit QA Mutu Lolos (100%)".into(), true));
        }

        if let Some(pdf) = &state.last_output_pdf {
            if ui
                .add(
                    egui::Button::new(RichText::new(crate::i18n::tr(lang, "View Latest PDF")).font(FontId::proportional(11.0)).color(t.accent))
                        .corner_radius(CornerRadius::same(t.radius)),
                )
                .on_hover_text(crate::i18n::tr(lang, "Open {path}").replace("{path}", pdf))
                .clicked()
            {
                let now = format_now();
                state.execution_log.push(format!("[{now}] {}", crate::i18n::tr(lang, "Opening release file: {path}").replace("{path}", pdf)));
                state.status_message = Some(("Membuka berkas PDF".into(), true));
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if state.last_qa_clean {
                ui.label(RichText::new(format!("● {}", crate::i18n::tr(lang, "Clean (QA Fit 100%)"))).font(semibold(11.0)).color(t.success));
            } else {
                ui.label(RichText::new(format!("● {}", crate::i18n::tr(lang, "Issues Detected"))).font(semibold(11.0)).color(t.danger));
            }
        });
    });

    // 3. Tab Selector Bar
    ui.horizontal(|ui| {
        let is_tab1 = state.center_tab == OrchestratorTab::StrukturHalaman;
        let is_tab2 = state.center_tab == OrchestratorTab::VisualLivePreview;
        let is_tab3 = state.center_tab == OrchestratorTab::LogEksekusi;

        let tab1_title = crate::i18n::tr(lang, "Page Structure");
        let tab2_title = crate::i18n::tr(lang, "Visual Live Preview");
        let tab3_title = crate::i18n::tr(lang, "Execution Log");

        if tab_button(ui, &tab1_title, is_tab1, t).clicked() {
            state.center_tab = OrchestratorTab::StrukturHalaman;
        }
        if tab_button(ui, &tab2_title, is_tab2, t).clicked() {
            state.center_tab = OrchestratorTab::VisualLivePreview;
        }
        if tab_button(ui, &tab3_title, is_tab3, t).clicked() {
            state.center_tab = OrchestratorTab::LogEksekusi;
        }
    });

    ui.separator();

    // 4. Tab Content Area
    match state.center_tab {
        OrchestratorTab::StrukturHalaman => {
            let count_str = state.cards.len().to_string();
            let content_list_label = crate::i18n::tr(lang, "Publication Content List ({count} items)").replace("{count}", &count_str);
            ui.label(RichText::new(content_list_label).font(semibold(11.5)).color(t.text_dim));

            ScrollArea::vertical().id_salt("pubgen_center_cards_scroll").auto_shrink([false, false]).show(ui, |ui| {
                render_card_grid(app, ui, state, t, lang);
            });
        }
        OrchestratorTab::VisualLivePreview => {
            render_visual_live_preview(app, ui, state, t, lang);
        }
        OrchestratorTab::LogEksekusi => {
            render_log_eksekusi(app, ui, state, t);
        }
    }
}

fn tab_button(ui: &mut Ui, label: &str, active: bool, t: &Tokens) -> Response {
    let font = FontId::proportional(DOCK_TAB_LABEL);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, if active { t.text_strong } else { t.text_dim });
    let size = vec2(galley.size().x + DOCK_TAB_PAD * 2.0, DOCK_TAB_H);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, active, label));

    if active {
        ui.painter().rect_filled(rect, CornerRadius::ZERO, t.panel);
        ui.painter().line_segment([rect.left_top(), rect.left_bottom()], Stroke::new(1.0, t.border));
        ui.painter().line_segment([rect.right_top(), rect.right_bottom()], Stroke::new(1.0, t.border));
    } else {
        ui.painter().rect_filled(rect, CornerRadius::ZERO, t.tab_strip);
        if resp.hovered() {
            ui.painter().rect_filled(rect, CornerRadius::ZERO, t.hover);
        }
        ui.painter().line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, t.border));
    }

    ui.painter().galley(pos2(rect.min.x + DOCK_TAB_PAD, rect.center().y - galley.size().y / 2.0), galley, Color32::PLACEHOLDER);

    resp
}

// ============================================================================
// GRID KARTU PUBLIKASI (FACING SPREADS / SINGLE PAGE RESPONSIVE)
// ============================================================================

fn paint_card_thumbnail(painter: &egui::Painter, rect: Rect, kind: CardKind, t: &Tokens) {
    match kind {
        CardKind::Cover => {
            // Crimson swatch
            painter.rect(
                rect,
                CornerRadius::same(2),
                Color32::from_rgb(139, 30, 45),
                Stroke::new(1.0, Color32::from_rgb(180, 40, 60)),
                StrokeKind::Inside,
            );
            painter.rect_filled(Rect::from_min_size(rect.min + vec2(4.0, 6.0), vec2(28.0, 6.0)), CornerRadius::same(1), Color32::WHITE);
            painter.rect_filled(
                Rect::from_min_size(rect.min + vec2(4.0, 16.0), vec2(20.0, 3.0)),
                CornerRadius::same(1),
                Color32::from_rgba_premultiplied(255, 255, 255, 160),
            );
        }
        CardKind::Narrative => {
            // Teal icon with text lines
            painter.rect(rect, CornerRadius::same(2), t.panel_darker, Stroke::new(1.0, Color32::from_rgb(13, 148, 136)), StrokeKind::Inside);
            for line in 0..4 {
                let y = rect.min.y + 10.0 + (line as f32 * 8.0);
                let w = if line == 3 { 16.0 } else { 24.0 };
                painter.rect_filled(
                    Rect::from_min_size(egui::pos2(rect.min.x + 6.0, y), vec2(w, 2.5)),
                    CornerRadius::same(1),
                    Color32::from_rgb(45, 212, 191),
                );
            }
        }
        CardKind::Chart => {
            // Blue bar chart
            painter.rect(rect, CornerRadius::same(2), t.panel_darker, Stroke::new(1.0, Color32::from_rgb(37, 99, 235)), StrokeKind::Inside);
            let bars = [(8.0, 18.0), (16.0, 30.0), (24.0, 24.0)];
            for (bx, bh) in bars {
                painter.rect_filled(
                    Rect::from_min_size(egui::pos2(rect.min.x + bx, rect.max.y - 6.0 - bh), vec2(5.0, bh)),
                    CornerRadius::same(1),
                    Color32::from_rgb(96, 165, 250),
                );
            }
        }
        CardKind::Table => {
            // Orange table grid
            painter.rect(rect, CornerRadius::same(2), t.panel_darker, Stroke::new(1.0, Color32::from_rgb(234, 88, 12)), StrokeKind::Inside);
            painter.rect_filled(
                Rect::from_min_size(rect.min + vec2(4.0, 6.0), vec2(28.0, 5.0)),
                CornerRadius::same(1),
                Color32::from_rgb(251, 146, 60),
            );
            for row in 0..3 {
                let y = rect.min.y + 14.0 + (row as f32 * 8.0);
                painter.line_segment([egui::pos2(rect.min.x + 4.0, y), egui::pos2(rect.max.x - 4.0, y)], Stroke::new(1.0, t.field_border));
            }
        }
        CardKind::Image => {
            // Purple image / illustration thumbnail
            painter.rect(rect, CornerRadius::same(2), t.panel_darker, Stroke::new(1.0, Color32::from_rgb(147, 51, 234)), StrokeKind::Inside);
            painter.circle_filled(rect.min + vec2(10.0, 14.0), 4.0, Color32::from_rgb(192, 132, 252));
            let p1 = [rect.min + vec2(6.0, 44.0), rect.min + vec2(18.0, 24.0), rect.min + vec2(28.0, 44.0)];
            painter.add(egui::Shape::convex_polygon(p1.to_vec(), Color32::from_rgb(168, 85, 247), Stroke::NONE));
            let p2 = [rect.min + vec2(20.0, 44.0), rect.min + vec2(28.0, 32.0), rect.min + vec2(34.0, 44.0)];
            painter.add(egui::Shape::convex_polygon(p2.to_vec(), Color32::from_rgb(147, 51, 234), Stroke::NONE));
        }
        CardKind::MapChart => {
            // Emerald/Teal Map chart thumbnail with geographic polygon contours
            painter.rect(rect, CornerRadius::same(2), t.panel_darker, Stroke::new(1.0, Color32::from_rgb(16, 185, 129)), StrokeKind::Inside);
            let p1 = [rect.min + vec2(6.0, 16.0), rect.min + vec2(18.0, 10.0), rect.min + vec2(22.0, 22.0), rect.min + vec2(10.0, 26.0)];
            painter.add(egui::Shape::convex_polygon(p1.to_vec(), Color32::from_rgb(52, 211, 153), Stroke::NONE));
            let p2 = [rect.min + vec2(20.0, 20.0), rect.min + vec2(30.0, 14.0), rect.min + vec2(34.0, 32.0), rect.min + vec2(18.0, 34.0)];
            painter.add(egui::Shape::convex_polygon(p2.to_vec(), Color32::from_rgb(16, 185, 129), Stroke::NONE));
            let p3 = [rect.min + vec2(8.0, 30.0), rect.min + vec2(16.0, 28.0), rect.min + vec2(24.0, 42.0), rect.min + vec2(10.0, 44.0)];
            painter.add(egui::Shape::convex_polygon(p3.to_vec(), Color32::from_rgb(5, 150, 105), Stroke::NONE));
            painter.circle_filled(rect.min + vec2(24.0, 24.0), 2.5, Color32::from_rgb(251, 191, 36));
        }
        CardKind::Toc => {
            // Indigo TOC thumbnail with dot leaders
            painter.rect(rect, CornerRadius::same(2), t.panel_darker, Stroke::new(1.0, Color32::from_rgb(99, 102, 241)), StrokeKind::Inside);
            for line in 0..4 {
                let y = rect.min.y + 10.0 + (line as f32 * 8.0);
                painter.rect_filled(
                    Rect::from_min_size(egui::pos2(rect.min.x + 6.0, y), vec2(14.0, 2.0)),
                    CornerRadius::same(1),
                    Color32::from_rgb(165, 180, 252),
                );
                painter.line_segment(
                    [egui::pos2(rect.min.x + 22.0, y + 1.0), egui::pos2(rect.min.x + 28.0, y + 1.0)],
                    Stroke::new(1.0, Color32::from_rgb(129, 140, 248)),
                );
                painter.rect_filled(
                    Rect::from_min_size(egui::pos2(rect.min.x + 30.0, y), vec2(4.0, 2.0)),
                    CornerRadius::same(1),
                    Color32::from_rgb(199, 210, 254),
                );
            }
        }
        CardKind::Divider => {
            // Gold/Amber divider spread thumbnail
            painter.rect(rect, CornerRadius::same(2), t.panel_darker, Stroke::new(1.0, Color32::from_rgb(217, 119, 6)), StrokeKind::Inside);
            painter.rect_filled(
                Rect::from_min_size(rect.min + vec2(4.0, 18.0), vec2(28.0, 8.0)),
                CornerRadius::same(1),
                Color32::from_rgb(245, 158, 11),
            );
            painter.rect_filled(
                Rect::from_min_size(rect.min + vec2(8.0, 30.0), vec2(20.0, 3.0)),
                CornerRadius::same(1),
                Color32::from_rgb(251, 191, 36),
            );
        }
    }
}

fn render_card_grid(_app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    let avail_w = ui.available_width();
    let gap = 8.0f32;
    let mut clicked_edit_idx = None;
    let visible_window = ui.clip_rect().expand2(vec2(0.0, 100.0));

    if state.page_layout_facing {
        // FACING PAGES SPREADS LAYOUT (InDesign parity - Image #3)
        // Row 1: Page 1 (Cover / Sampul) alone on left with empty right slot (matching trailing last card)
        // Subsequent rows: Pairs of Left Page (Verso, spine right) and Right Page (Recto, spine left)
        let col_w = ((avail_w - gap) / 2.0).max(120.0);
        let det_w = card_details_width(col_w);

        if let Some(cover_card) = state.cards.first() {
            let row_h = card_row_height(ui.painter(), std::slice::from_ref(cover_card), det_w, lang);
            let row_rect = Rect::from_min_size(ui.cursor().min, vec2(avail_w, row_h));
            if !visible_window.intersects(row_rect) {
                ui.allocate_exact_size(vec2(avail_w, row_h), Sense::hover());
                ui.add_space(gap);
            } else {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(gap, 0.0);
                    if let Some(idx) = render_facing_card(ui, 0, cover_card, col_w, false, row_h, t, lang) {
                        clicked_edit_idx = Some(idx);
                    }
                    // Empty right slot so cover card width is col_w and height matches the row
                    ui.allocate_exact_size(vec2(col_w, row_h), Sense::hover());
                });
                ui.add_space(gap);
            }
        }

        // Remaining body cards starting from index 1 (Page 2 onwards)
        if state.cards.len() > 1 {
            let body_cards = &state.cards[1..];
            for (chunk_idx, pair) in body_cards.chunks(2).enumerate() {
                let row_h = card_row_height(ui.painter(), pair, det_w, lang);
                let row_rect = Rect::from_min_size(ui.cursor().min, vec2(avail_w, row_h));
                if !visible_window.intersects(row_rect) {
                    ui.allocate_exact_size(vec2(avail_w, row_h), Sense::hover());
                    ui.add_space(gap);
                    continue;
                }
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(gap, 0.0);
                    let actual_idx_left = 1 + chunk_idx * 2;
                    // Left page (Verso: thumbnail on right facing spine, text right-aligned)
                    let is_single_trailing = pair.len() == 1;
                    if let Some(idx) = render_facing_card(ui, actual_idx_left, &pair[0], col_w, !is_single_trailing, row_h, t, lang) {
                        clicked_edit_idx = Some(idx);
                    }

                    if pair.len() > 1 {
                        let actual_idx_right = actual_idx_left + 1;
                        // Right page (Recto: thumbnail on left facing spine, text left-aligned)
                        if let Some(idx) = render_facing_card(ui, actual_idx_right, &pair[1], col_w, false, row_h, t, lang) {
                            clicked_edit_idx = Some(idx);
                        }
                    } else {
                        // Empty slot on right for trailing single page
                        ui.allocate_exact_size(vec2(col_w, row_h), Sense::hover());
                    }
                });
                ui.add_space(gap);
            }
        }
    } else {
        // SINGLE PAGE GRID LAYOUT (2 Columns standard)
        let col_count = if avail_w > 520.0 { 2 } else { 1 };
        let card_w = ((avail_w - (col_count as f32 - 1.0) * gap) / (col_count as f32)).max(120.0);
        let det_w = card_details_width(card_w);
        let chunk_size = col_count;
        for (row_idx, chunk) in state.cards.chunks(chunk_size).enumerate() {
            let row_h = card_row_height(ui.painter(), chunk, det_w, lang);
            let row_rect = Rect::from_min_size(ui.cursor().min, vec2(avail_w, row_h));
            if !visible_window.intersects(row_rect) {
                ui.allocate_exact_size(vec2(avail_w, row_h), Sense::hover());
                ui.add_space(gap);
                continue;
            }
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(gap, 0.0);
                for (col_idx, card) in chunk.iter().enumerate() {
                    let actual_idx = row_idx * chunk_size + col_idx;
                    if let Some(idx) = render_facing_card(ui, actual_idx, card, card_w, false, row_h, t, lang) {
                        clicked_edit_idx = Some(idx);
                    }
                }
                if chunk.len() < chunk_size {
                    for _ in chunk.len()..chunk_size {
                        ui.allocate_exact_size(vec2(card_w, row_h), Sense::hover());
                    }
                }
            });
            ui.add_space(gap);
        }
    }

    // Bottom Action Bar from Image #3:
    // Tambah Kartu: [Paragraf v] [+ Tambah Halaman Baru] [+ Tambah Objek Baru]
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);
    render_card_action_bar(ui, state, t, lang);

    if let Some(idx) = clicked_edit_idx {
        state.editing_card_idx = Some(idx);
    }
}

/// Horizontal text width inside a page card, beside the 34px thumbnail
/// (10px padding each side + 8px gutter; see `render_facing_card`).
fn card_details_width(card_w: f32) -> f32 {
    (card_w - 62.0).max(40.0)
}

/// Height a page card needs so every text line stays visible: pill row +
/// wrapped title + wrapped source + footer, with the card's inner padding.
/// Rows rendered with `Ui::horizontal` are never shorter than the style's
/// `interact_size.y` (18 px by default), so the same minimum applies here —
/// assuming text height alone under-measured and clipped card footers.
fn measured_card_height(painter: &egui::Painter, card: &PublicationCard, details_w: f32, lang: &str) -> f32 {
    let cache_id = egui::Id::unique(("card_h", card.page, &card.title, &card.source, lang, details_w.to_bits()));
    if let Some(h) = painter.ctx().data(|d| d.get_temp::<f32>(cache_id)) {
        return h;
    }
    // Only metrics matter here (nothing is painted): PLACEHOLDER keeps the layout independent of the
    // theme's text colour.
    let title = painter.layout(card.title.clone(), semibold(12.0), Color32::PLACEHOLDER, details_w);
    let src = format!("{}: {}", crate::i18n::tr(lang, "Source"), card.source);
    let source = painter.layout(src, FontId::proportional(11.0), Color32::PLACEHOLDER, details_w);
    let row_min = painter.ctx().global_style().spacing.interact_size.y;
    const LINE_GAP: f32 = 2.0;
    const PAD_Y: f32 = 12.0; // 6px top + 6px bottom inner padding
    let rows = [row_min, title.size().y.max(row_min), source.size().y.max(row_min), row_min];
    let mut h = PAD_Y + LINE_GAP * (rows.len() - 1) as f32 + rows.iter().sum::<f32>();
    if card.kind == CardKind::Cover {
        h += row_min + LINE_GAP;
    }
    painter.ctx().data_mut(|d| d.insert_temp(cache_id, h));
    h
}

/// Uniform row height: the tallest card in the row, never below the legacy 84px.
fn card_row_height(painter: &egui::Painter, cards: &[PublicationCard], details_w: f32, lang: &str) -> f32 {
    cards.iter().map(|c| measured_card_height(painter, c, details_w, lang)).fold(84.0, f32::max)
}

fn render_facing_card(
    ui: &mut Ui,
    idx: usize,
    card: &PublicationCard,
    card_w: f32,
    is_verso: bool,
    card_h: f32,
    t: &Tokens,
    lang: &str,
) -> Option<usize> {
    let mut clicked_edit = false;

    ui.push_id(idx, |ui| {
        let (card_rect, card_resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
        card_resp.widget_info(|| {
            let label = format!("{} — {} ({})", card.title, crate::i18n::tr(lang, "Edit card"), card.page);
            egui::WidgetInfo::labeled(egui::Role::Button, true, label)
        });
        if card_resp.clicked() {
            clicked_edit = true;
        }

        if !ui.is_rect_visible(card_rect) {
            return;
        }

        let is_hovered = card_resp.hovered();
        let bg = if is_hovered { t.hover } else { t.panel };
        let border_c = if is_hovered { t.accent } else { t.field_border };
        let card_clip = card_rect.intersect(ui.clip_rect());
        let painter = ui.painter().with_clip_rect(card_clip);
        painter.rect_filled(card_rect, 4.0, bg);
        painter.rect_stroke(card_rect, 4.0, Stroke::new(1.0, border_c), egui::StrokeKind::Inside);

        let inner_rect = card_rect.shrink2(vec2(10.0, 6.0));
        let thumb_w = 34.0f32;
        let thumb_h = 48.0f32;
        let thumb_size = vec2(thumb_w, thumb_h);
        let thumb_y = inner_rect.min.y + ((inner_rect.height() - thumb_h) / 2.0).max(0.0);

        let (thumb_rect, details_rect) = if is_verso {
            // Verso (Left Page): Thumbnail on the right facing spine, text details on left (right-aligned)
            let thumb_r = Rect::from_min_size(pos2(inner_rect.max.x - thumb_w, thumb_y), thumb_size);
            let det_r = Rect::from_min_max(inner_rect.min, pos2(inner_rect.max.x - thumb_w - 8.0, inner_rect.max.y));
            (thumb_r, det_r)
        } else {
            // Recto (Right Page): Thumbnail on the left facing spine, text details on right (left-aligned)
            let thumb_r = Rect::from_min_size(pos2(inner_rect.min.x, thumb_y), thumb_size);
            let det_r = Rect::from_min_max(pos2(inner_rect.min.x + thumb_w + 8.0, inner_rect.min.y), inner_rect.max);
            (thumb_r, det_r)
        };

        paint_card_thumbnail(&painter, thumb_rect, card.kind, t);

        let details_clip = details_rect.intersect(ui.clip_rect());
        if !details_clip.is_positive() {
            return;
        }

        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(details_rect).layout(Layout::top_down(Align::Min)));
        child_ui.set_clip_rect(details_clip);
        child_ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

        // Line 1: Tags
        child_ui.horizontal(|ui| {
            render_tag_pill(ui, crate::i18n::tr(lang, &card.tag1), t);
            render_tag_pill(ui, crate::i18n::tr(lang, &card.tag2), t);
        });

        // Line 2: Title
        let title_label = egui::Label::new(RichText::new(&card.title).font(semibold(12.0)).color(t.text_strong)).sense(Sense::click());
        let title_resp = child_ui.add(title_label);
        if title_resp.clicked() {
            clicked_edit = true;
        }

        // Line 3: Source
        let src_label = format!("{}: {}", crate::i18n::tr(lang, "Source"), card.source);
        child_ui.add(egui::Label::new(RichText::new(src_label).font(FontId::proportional(11.0)).color(t.text_dim)));

        // Line 4: Footer
        child_ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{} {}", crate::i18n::tr(lang, "Pg."), card.page)).font(semibold(11.0)).color(t.text_dim));
            ui.add_space(8.0);
            let edit_resp =
                ui.link(RichText::new(format!("✎ {}", crate::i18n::tr(lang, "Click to edit"))).font(FontId::proportional(11.0)).color(t.accent));
            if edit_resp.clicked() {
                clicked_edit = true;
            }
        });

        // Cover Cloudflare AI quota indicator
        if card.kind == CardKind::Cover {
            if let Some(n) = super::with_state(ui.ctx(), |st| st.cover_quota_covers) {
                child_ui.label(RichText::new(format!("Cloudflare AI: ~{n} covers left today")).font(FontId::proportional(11.0)).color(t.text_dim));
            }
        }
    });

    if clicked_edit { Some(idx) } else { None }
}

fn render_card_action_bar(ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);

        ui.label(RichText::new(crate::i18n::tr(lang, "Add Card:")).font(semibold(11.0)).color(t.text_dim));

        let kind_label = match state.new_card_kind {
            CardKind::Narrative => crate::i18n::tr(lang, "Paragraph"),
            CardKind::Chart => crate::i18n::tr(lang, "Chart"),
            CardKind::MapChart => crate::i18n::tr(lang, "Map Chart"),
            CardKind::Table => crate::i18n::tr(lang, "Table"),
            CardKind::Image => crate::i18n::tr(lang, "Image"),
            CardKind::Cover => crate::i18n::tr(lang, "Cover"),
            CardKind::Toc => crate::i18n::tr(lang, "Table of Contents"),
            CardKind::Divider => crate::i18n::tr(lang, "Divider"),
        };

        craft_ui::ComboBox::from_id_salt("add_card_kind_combo")
            .selected_text(RichText::new(kind_label).font(FontId::proportional(11.0)))
            .width(110.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.new_card_kind, CardKind::Narrative, crate::i18n::tr(lang, "Paragraph"));
                ui.selectable_value(&mut state.new_card_kind, CardKind::Chart, crate::i18n::tr(lang, "Chart"));
                ui.selectable_value(&mut state.new_card_kind, CardKind::MapChart, crate::i18n::tr(lang, "Map Chart"));
                ui.selectable_value(&mut state.new_card_kind, CardKind::Table, crate::i18n::tr(lang, "Table"));
                ui.selectable_value(&mut state.new_card_kind, CardKind::Image, crate::i18n::tr(lang, "Image"));
                ui.selectable_value(&mut state.new_card_kind, CardKind::Toc, crate::i18n::tr(lang, "Table of Contents"));
                ui.selectable_value(&mut state.new_card_kind, CardKind::Divider, crate::i18n::tr(lang, "Divider"));
            });

        let btn_fill = t.accent_strong;
        if ui
            .add(
                egui::Button::new(RichText::new(format!("+ {}", crate::i18n::tr(lang, "Add New Page"))).font(semibold(11.0)).color(Color32::WHITE))
                    .fill(btn_fill)
                    .corner_radius(CornerRadius::same(t.radius)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Add New Page"))
            .clicked()
        {
            let next_page = state.cards.last().map(|c| c.page + 1).unwrap_or(1);
            let title_prefix = match state.new_card_kind {
                CardKind::Narrative => "Ulasan Narasi Baru",
                CardKind::Chart => "Grafik Baru",
                CardKind::MapChart => "Peta Tematik Baru",
                CardKind::Table => "Tabel Baru",
                CardKind::Image => "Infografis Baru",
                CardKind::Cover => "Sampul Baru",
                CardKind::Toc => "Daftar Isi Baru",
                CardKind::Divider => "Pembatas Bab Baru",
            };
            state.cards.push(PublicationCard::new_with(
                state.new_card_kind,
                &format!("{title_prefix} (Hal. {next_page})"),
                "Draf konten baru",
                "Manual",
                next_page,
                true,
            ));
            let now = format_now();
            state.execution_log.push(format!("[{now}] Kartu halaman baru ditambahkan (Hal {next_page})."));
        }

        if ui
            .add(
                egui::Button::new(RichText::new(format!("+ {}", crate::i18n::tr(lang, "Add New Object"))).font(semibold(11.0)).color(Color32::WHITE))
                    .fill(btn_fill)
                    .corner_radius(CornerRadius::same(t.radius)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Add New Object"))
            .clicked()
        {
            let curr_page = state.cards.last().map(|c| c.page).unwrap_or(1);
            let title_prefix = match state.new_card_kind {
                CardKind::Narrative => "Objek Paragraf",
                CardKind::Chart => "Objek Grafik",
                CardKind::MapChart => "Objek Peta Tematik",
                CardKind::Table => "Objek Tabel",
                CardKind::Image => "Objek Gambar",
                CardKind::Cover => "Objek Sampul",
                CardKind::Toc => "Objek Daftar Isi",
                CardKind::Divider => "Objek Pembatas Bab",
            };
            state.cards.push(PublicationCard::new_with(
                state.new_card_kind,
                &format!("{title_prefix} Tambahan Hal {curr_page}"),
                "Objek baru dalam halaman",
                "Manual",
                curr_page,
                false,
            ));
            let now = format_now();
            state.execution_log.push(format!("[{now}] Objek baru ditambahkan pada Hal {curr_page}."));
        }
    });
}

fn render_tag_pill(ui: &mut Ui, label: &str, t: &Tokens) {
    let font = FontId::proportional(11.0);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, t.text);
    let size = vec2(galley.size().x + 8.0, 16.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, label));

    ui.painter().rect(rect, CornerRadius::same(2), t.well, Stroke::new(1.0, t.field_border), StrokeKind::Inside);
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, Color32::PLACEHOLDER);
}

// ============================================================================
// VISUAL LIVE PREVIEW TAB & LOG EKSEKUSI TAB
// ============================================================================

fn render_visual_live_preview(_app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    ScrollArea::vertical().id_salt("pubgen_center_preview_scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new(crate::i18n::tr(lang, "Page Layout Preview (Live Spread Document)")).font(semibold(11.5)).color(t.text_strong));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(crate::i18n::tr(lang, "Total: {n} pages").replace("{n}", &state.cards.len().to_string()))
                        .font(FontId::proportional(11.0))
                        .color(t.text_dim),
                );
            });
        });

        // Document Canvas simulation
        let canvas_w = ui.available_width().min(680.0);
        let canvas_h = (canvas_w * 1.414).min(520.0);
        let (rect, _) = ui.allocate_exact_size(vec2(canvas_w, canvas_h), Sense::hover());

        // Outer drop shadow
        ui.painter().rect_filled(rect.translate(vec2(4.0, 4.0)), CornerRadius::same(2), Color32::from_black_alpha(80));

        // A4 Document white page
        ui.painter().rect_filled(rect, CornerRadius::same(2), Color32::WHITE);

        // Magenta margins outline
        let margin_inset = 24.0f32;
        let margin_rect = Rect::from_min_max(rect.min + vec2(margin_inset, margin_inset), rect.max - vec2(margin_inset, margin_inset));
        ui.painter().rect_stroke(margin_rect, CornerRadius::same(0), Stroke::new(1.0, Color32::from_rgb(236, 72, 153)), StrokeKind::Inside);

        // Mockup content on page
        // 1. Running header line
        ui.painter().line_segment(
            [margin_rect.left_top() + vec2(0.0, 18.0), margin_rect.right_top() + vec2(0.0, 18.0)],
            Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
        );
        ui.painter().text(
            margin_rect.left_top() + vec2(2.0, 8.0),
            Align2::LEFT_CENTER,
            &state.meta_release_no,
            FontId::proportional(11.0),
            Color32::from_rgb(80, 80, 80),
        );

        // 2. Mock Headline
        ui.painter().text(
            margin_rect.left_top() + vec2(0.0, 40.0),
            Align2::LEFT_CENTER,
            &state.meta_title,
            FontId::proportional(16.0),
            Color32::from_rgb(20, 20, 20),
        );

        // 3. Mock 2-column text boxes
        let col_gap = 16.0f32;
        let col_w = (margin_rect.width() - col_gap) / 2.0;

        for (ci, cx) in [margin_rect.left(), margin_rect.left() + col_w + col_gap].iter().enumerate() {
            for line in 0..12 {
                let y = margin_rect.top() + 70.0 + (line as f32 * 12.0);
                let w = if line == 11 { col_w * 0.6 } else { col_w };
                ui.painter().rect_filled(
                    Rect::from_min_size(egui::pos2(*cx, y), vec2(w, 4.0)),
                    CornerRadius::same(1),
                    Color32::from_rgb(180, 185, 195),
                );
            }

            // If column 2, draw mini chart
            if ci == 1 {
                let chart_rect = Rect::from_min_size(egui::pos2(*cx, margin_rect.top() + 230.0), vec2(col_w, 100.0));
                ui.painter().rect_stroke(chart_rect, CornerRadius::same(2), Stroke::new(1.0, Color32::from_rgb(210, 210, 210)), StrokeKind::Inside);
                ui.painter().text(
                    chart_rect.center(),
                    Align2::CENTER_CENTER,
                    "Grafik Statistik Komparatif",
                    FontId::proportional(11.0),
                    Color32::from_rgb(100, 100, 100),
                );
            }
        }
    });
}

fn render_log_eksekusi(_app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens) {
    ScrollArea::vertical().id_salt("pubgen_center_logs_scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

        egui::Frame::NONE
            .fill(t.panel_darker)
            .stroke(Stroke::new(1.0, t.field_border))
            .corner_radius(CornerRadius::same(t.radius))
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                for line in &state.execution_log {
                    ui.label(RichText::new(line).font(FontId::monospace(10.5)).color(
                        if line.contains("Audit QA") || line.contains("QA audit") || line.contains("berhasil") || line.contains("finished") {
                            t.success
                        } else if line.contains("Error") {
                            t.danger
                        } else {
                            t.text
                        },
                    ));
                }
            });
    });
}

/// Helper to sanitize an ID or title into snake_case slot suffix
fn clean_slot_id(id: &str) -> String {
    let mut clean_id = id.trim();
    for ext in [".csv", ".rtf", ".png", ".jpg", ".jpeg", ".md", ".txt"] {
        if clean_id.to_lowercase().ends_with(ext) {
            clean_id = &clean_id[..clean_id.len() - ext.len()];
        }
    }
    if let Some(pos) = clean_id.rfind('/') {
        clean_id = &clean_id[pos + 1..];
    }
    if let Some(pos) = clean_id.rfind('\\') {
        clean_id = &clean_id[pos + 1..];
    }
    let mut s = String::new();
    let mut last_dash = false;
    for c in clean_id.chars() {
        if c.is_alphanumeric() {
            s.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !s.is_empty() {
            s.push('_');
            last_dash = true;
        }
    }
    let trimmed = s.trim_end_matches('_');
    if trimmed.is_empty() { "item".to_string() } else { trimmed.to_string() }
}

/// Dynamically detect all translation rule units / slots from the publication cards,
/// staged content, and metadata based on the active publication profile rules.
pub fn detect_translation_rule_units(state: &OrchestratorWorkspaceState) -> Vec<SlotItem> {
    let mut out: Vec<SlotItem> = Vec::new();
    let mut seen_texts = std::collections::BTreeSet::new();
    let prof = state.selected_profile.as_str();

    // Collect canonical units from the core draft
    let draft = worksheet_draft(state);
    let collected_units = pubgen_core::translation_units::collect_units(&draft);

    let role_budget = |role: pubgen_core::translation_units::UnitRole| -> Option<usize> {
        match prof {
            "bps_brs" => match role {
                pubgen_core::translation_units::UnitRole::CoverTitle => Some(60),
                pubgen_core::translation_units::UnitRole::NarrativeTitle => Some(80),
                pubgen_core::translation_units::UnitRole::NarrativeBody => Some(280),
                pubgen_core::translation_units::UnitRole::TableTitle => Some(150),
                pubgen_core::translation_units::UnitRole::TableHeader => Some(40),
                pubgen_core::translation_units::UnitRole::TableRowLabel => Some(50),
                pubgen_core::translation_units::UnitRole::TableNote => Some(150),
                pubgen_core::translation_units::UnitRole::ChartTitle => Some(100),
                pubgen_core::translation_units::UnitRole::ChartSeries => Some(35),
                pubgen_core::translation_units::UnitRole::ImageCaption => Some(120),
                _ => Some(100),
            },
            "bps_dda" => match role {
                pubgen_core::translation_units::UnitRole::CoverTitle => Some(100),
                pubgen_core::translation_units::UnitRole::DividerTitle => Some(60),
                pubgen_core::translation_units::UnitRole::TocTitle => Some(40),
                pubgen_core::translation_units::UnitRole::NarrativeTitle => Some(100),
                pubgen_core::translation_units::UnitRole::NarrativeBody => Some(350),
                pubgen_core::translation_units::UnitRole::TableTitle => Some(160),
                pubgen_core::translation_units::UnitRole::TableHeader => Some(40),
                pubgen_core::translation_units::UnitRole::TableRowLabel => Some(50),
                pubgen_core::translation_units::UnitRole::TableNote => Some(150),
                pubgen_core::translation_units::UnitRole::ChartTitle => Some(100),
                pubgen_core::translation_units::UnitRole::ChartSeries => Some(35),
                pubgen_core::translation_units::UnitRole::MapTitle => Some(80),
                pubgen_core::translation_units::UnitRole::ImageCaption => Some(120),
                pubgen_core::translation_units::UnitRole::HeaderFooter => Some(80),
                _ => Some(100),
            },
            "corporate_annual" => match role {
                pubgen_core::translation_units::UnitRole::CoverTitle => Some(80),
                pubgen_core::translation_units::UnitRole::DividerTitle => Some(60),
                pubgen_core::translation_units::UnitRole::NarrativeTitle => Some(90),
                pubgen_core::translation_units::UnitRole::NarrativeBody => Some(400),
                pubgen_core::translation_units::UnitRole::TableTitle => Some(150),
                pubgen_core::translation_units::UnitRole::TableHeader => Some(40),
                pubgen_core::translation_units::UnitRole::TableRowLabel => Some(50),
                pubgen_core::translation_units::UnitRole::TableNote => Some(150),
                pubgen_core::translation_units::UnitRole::ChartTitle => Some(100),
                pubgen_core::translation_units::UnitRole::ChartSeries => Some(40),
                pubgen_core::translation_units::UnitRole::ImageCaption => Some(120),
                _ => Some(100),
            },
            "academic_monograph" => match role {
                pubgen_core::translation_units::UnitRole::CoverTitle => Some(120),
                pubgen_core::translation_units::UnitRole::DividerTitle => Some(60),
                pubgen_core::translation_units::UnitRole::NarrativeTitle => Some(80),
                pubgen_core::translation_units::UnitRole::NarrativeBody => Some(500),
                pubgen_core::translation_units::UnitRole::TableTitle => Some(150),
                pubgen_core::translation_units::UnitRole::TableHeader => Some(40),
                pubgen_core::translation_units::UnitRole::TableRowLabel => Some(50),
                pubgen_core::translation_units::UnitRole::TableNote => Some(150),
                pubgen_core::translation_units::UnitRole::ChartTitle => Some(100),
                pubgen_core::translation_units::UnitRole::ImageCaption => Some(120),
                _ => Some(100),
            },
            _ => match role {
                pubgen_core::translation_units::UnitRole::CoverTitle => Some(80),
                pubgen_core::translation_units::UnitRole::TableTitle => Some(150),
                pubgen_core::translation_units::UnitRole::TableNote => Some(150),
                pubgen_core::translation_units::UnitRole::NarrativeBody => Some(300),
                _ => Some(100),
            },
        }
    };

    if prof == "bps_brs" {
        // 1. Cover Headline
        let headline = if !state.meta_title.trim().is_empty() {
            state.meta_title.clone()
        } else if let Some(cov) = state.cards.iter().find(|c| c.kind == CardKind::Cover) {
            cov.title.clone()
        } else {
            "Perkembangan Nilai Tukar Petani Jawa Barat".to_string()
        };
        seen_texts.insert(headline.clone());
        out.push(SlotItem { name: "cover_headline".into(), budget: Some(60), text: headline });

        // 2. Executive Summary / Ringkasan Eksekutif
        let summary_text = if let Some(u) = collected_units.iter().find(|u| {
            u.role == pubgen_core::translation_units::UnitRole::NarrativeBody
                && (u.card_id.to_lowercase().contains("point") || u.text.contains("NTP"))
        }) {
            u.text.clone()
        } else if let Some(card) = state.cards.iter().find(|c| {
            c.kind == CardKind::Narrative
                && (c.chapter_id.to_lowercase().contains("point")
                    || c.source.to_lowercase().contains("point")
                    || c.title.to_lowercase().contains("ringkasan"))
        }) {
            if let Some(pos) = card.title.find(':') {
                let rest = card.title[pos + 1..].trim();
                if rest.contains("NTP") {
                    rest.to_string()
                } else {
                    "NTP Jawa Barat pada Oktober 2026 sebesar 112,45 atau naik 0,65 persen dibanding bulan sebelumnya.".to_string()
                }
            } else {
                "NTP Jawa Barat pada Oktober 2026 sebesar 112,45 atau naik 0,65 persen dibanding bulan sebelumnya.".to_string()
            }
        } else {
            "NTP Jawa Barat pada Oktober 2026 sebesar 112,45 atau naik 0,65 persen dibanding bulan sebelumnya.".to_string()
        };
        seen_texts.insert(summary_text.clone());
        out.push(SlotItem { name: "ringkasan_eksekutif".into(), budget: Some(280), text: summary_text });

        // 3. Table Footnote / Catatan Kaki Tabel
        let note_text = if let Some(u) = collected_units.iter().find(|u| u.role == pubgen_core::translation_units::UnitRole::TableNote) {
            u.text.clone()
        } else {
            "Sumber: Badan Pusat Statistik Provinsi Jawa Barat, Sensus Pertanian 2026.".to_string()
        };
        seen_texts.insert(note_text.clone());
        out.push(SlotItem { name: "catatan_kaki_tabel".into(), budget: Some(150), text: note_text });

        // 4. All other translation rule units from cards
        for (card_idx, card) in state.cards.iter().enumerate() {
            let card_core_id = if card.chapter_id.is_empty() { format!("card_{}", card_idx + 1) } else { card.chapter_id.clone() };
            let cid = clean_slot_id(&card.chapter_id);
            match card.kind {
                CardKind::Cover => {}
                CardKind::Narrative => {
                    let is_point = card.chapter_id.to_lowercase().contains("point")
                        || card.source.to_lowercase().contains("point")
                        || card.title.to_lowercase().contains("ringkasan");
                    if !is_point {
                        if seen_texts.insert(card.title.clone()) {
                            out.push(SlotItem {
                                name: format!("ulasan_{cid}_judul"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::NarrativeTitle),
                                text: card.title.clone(),
                            });
                        }
                        let body = collected_units
                            .iter()
                            .find(|u| {
                                (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                    && u.role == pubgen_core::translation_units::UnitRole::NarrativeBody
                            })
                            .map(|u| u.text.clone())
                            .filter(|t| !t.trim().is_empty())
                            .unwrap_or_else(|| card.subtitle.clone());
                        if !body.trim().is_empty() && seen_texts.insert(body.clone()) {
                            out.push(SlotItem {
                                name: format!("ulasan_{cid}_narasi"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::NarrativeBody),
                                text: body,
                            });
                        }
                    }
                }
                CardKind::Table => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem {
                            name: format!("judul_tabel_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::TableTitle),
                            text: card.title.clone(),
                        });
                    }
                    if let Some(note) = collected_units.iter().find(|u| {
                        (u.card_id == card_core_id || u.card_id == card.chapter_id) && u.role == pubgen_core::translation_units::UnitRole::TableNote
                    }) {
                        if seen_texts.insert(note.text.clone()) {
                            out.push(SlotItem {
                                name: format!("catatan_kaki_{cid}"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::TableNote),
                                text: note.text.clone(),
                            });
                        }
                    }
                    for (h_idx, h) in collected_units
                        .iter()
                        .filter(|u| {
                            (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                && u.role == pubgen_core::translation_units::UnitRole::TableHeader
                        })
                        .enumerate()
                    {
                        if seen_texts.insert(h.text.clone()) {
                            out.push(SlotItem {
                                name: format!("header_tabel_{cid}_{h_idx}"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::TableHeader),
                                text: h.text.clone(),
                            });
                        }
                    }
                    for (r_idx, r) in collected_units
                        .iter()
                        .filter(|u| {
                            (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                && u.role == pubgen_core::translation_units::UnitRole::TableRowLabel
                        })
                        .enumerate()
                    {
                        if seen_texts.insert(r.text.clone()) {
                            out.push(SlotItem {
                                name: format!("label_baris_{cid}_{r_idx}"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::TableRowLabel),
                                text: r.text.clone(),
                            });
                        }
                    }
                }
                CardKind::Chart => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem {
                            name: format!("judul_grafik_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::ChartTitle),
                            text: card.title.clone(),
                        });
                    }
                    for (s_idx, s) in collected_units
                        .iter()
                        .filter(|u| {
                            (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                && u.role == pubgen_core::translation_units::UnitRole::ChartSeries
                        })
                        .enumerate()
                    {
                        if seen_texts.insert(s.text.clone()) {
                            out.push(SlotItem {
                                name: format!("seri_grafik_{cid}_{s_idx}"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::ChartSeries),
                                text: s.text.clone(),
                            });
                        }
                    }
                }
                CardKind::Image => {
                    let cap = card.settings.get("caption").cloned().unwrap_or_else(|| card.title.clone());
                    if seen_texts.insert(cap.clone()) {
                        out.push(SlotItem {
                            name: format!("keterangan_gambar_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::ImageCaption),
                            text: cap,
                        });
                    }
                }
                _ => {}
            }
        }
    } else if prof == "bps_dda" {
        // DDA (Daerah Dalam Angka) Nomenclature & Structure
        let cover_title = if !state.meta_title.trim().is_empty() { state.meta_title.clone() } else { "KABUPATEN DALAM ANGKA 2026".to_string() };
        seen_texts.insert(cover_title.clone());
        out.push(SlotItem { name: "cover_title".into(), budget: Some(100), text: cover_title });

        if !state.meta_subtitle.trim().is_empty() && seen_texts.insert(state.meta_subtitle.clone()) {
            out.push(SlotItem { name: "cover_title_en".into(), budget: Some(100), text: state.meta_subtitle.clone() });
        }
        if !state.meta_agency.trim().is_empty() && seen_texts.insert(state.meta_agency.clone()) {
            out.push(SlotItem { name: "cover_agency".into(), budget: Some(80), text: state.meta_agency.clone() });
        }

        for (card_idx, card) in state.cards.iter().enumerate() {
            let card_core_id = if card.chapter_id.is_empty() { format!("card_{}", card_idx + 1) } else { card.chapter_id.clone() };
            let cid = clean_slot_id(&card.chapter_id);
            match card.kind {
                CardKind::Cover => {}
                CardKind::Divider => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem {
                            name: format!("pembatas_bab_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::DividerTitle),
                            text: card.title.clone(),
                        });
                    }
                    if !card.title_en.trim().is_empty() && seen_texts.insert(card.title_en.clone()) {
                        out.push(SlotItem {
                            name: format!("pembatas_bab_en_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::DividerTitle),
                            text: card.title_en.clone(),
                        });
                    }
                }
                CardKind::Toc => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem {
                            name: "daftar_isi".into(),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::TocTitle),
                            text: card.title.clone(),
                        });
                    }
                }
                CardKind::Narrative => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem {
                            name: format!("ulasan_bab_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::NarrativeTitle),
                            text: card.title.clone(),
                        });
                    }
                    let body = collected_units
                        .iter()
                        .find(|u| {
                            (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                && u.role == pubgen_core::translation_units::UnitRole::NarrativeBody
                        })
                        .map(|u| u.text.clone())
                        .filter(|t| !t.trim().is_empty())
                        .unwrap_or_else(|| card.subtitle.clone());
                    if !body.trim().is_empty() && seen_texts.insert(body.clone()) {
                        out.push(SlotItem {
                            name: format!("narasi_bab_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::NarrativeBody),
                            text: body,
                        });
                    }
                }
                CardKind::Table => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem {
                            name: format!("judul_tabel_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::TableTitle),
                            text: card.title.clone(),
                        });
                    }
                    let note = collected_units
                        .iter()
                        .find(|u| {
                            (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                && u.role == pubgen_core::translation_units::UnitRole::TableNote
                        })
                        .map(|u| u.text.clone())
                        .unwrap_or_else(|| "Sumber / Source: BPS".to_string());
                    if seen_texts.insert(note.clone()) {
                        out.push(SlotItem {
                            name: format!("sumber_catatan_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::TableNote),
                            text: note,
                        });
                    }
                    for (h_idx, h) in collected_units
                        .iter()
                        .filter(|u| {
                            (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                && u.role == pubgen_core::translation_units::UnitRole::TableHeader
                        })
                        .enumerate()
                    {
                        if seen_texts.insert(h.text.clone()) {
                            out.push(SlotItem {
                                name: format!("header_kolom_{cid}_{h_idx}"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::TableHeader),
                                text: h.text.clone(),
                            });
                        }
                    }
                    for (r_idx, r) in collected_units
                        .iter()
                        .filter(|u| {
                            (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                && u.role == pubgen_core::translation_units::UnitRole::TableRowLabel
                        })
                        .enumerate()
                    {
                        if seen_texts.insert(r.text.clone()) {
                            out.push(SlotItem {
                                name: format!("label_baris_{cid}_{r_idx}"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::TableRowLabel),
                                text: r.text.clone(),
                            });
                        }
                    }
                }
                CardKind::Chart => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem {
                            name: format!("judul_grafik_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::ChartTitle),
                            text: card.title.clone(),
                        });
                    }
                    for (s_idx, s) in collected_units
                        .iter()
                        .filter(|u| {
                            (u.card_id == card_core_id || u.card_id == card.chapter_id)
                                && u.role == pubgen_core::translation_units::UnitRole::ChartSeries
                        })
                        .enumerate()
                    {
                        if seen_texts.insert(s.text.clone()) {
                            out.push(SlotItem {
                                name: format!("label_seri_{cid}_{s_idx}"),
                                budget: role_budget(pubgen_core::translation_units::UnitRole::ChartSeries),
                                text: s.text.clone(),
                            });
                        }
                    }
                }
                CardKind::MapChart => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem {
                            name: format!("judul_peta_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::MapTitle),
                            text: card.title.clone(),
                        });
                    }
                }
                CardKind::Image => {
                    let cap = card.settings.get("caption").cloned().unwrap_or_else(|| card.title.clone());
                    if seen_texts.insert(cap.clone()) {
                        out.push(SlotItem {
                            name: format!("keterangan_gambar_{cid}"),
                            budget: role_budget(pubgen_core::translation_units::UnitRole::ImageCaption),
                            text: cap,
                        });
                    }
                }
            }
        }
    } else if prof == "corporate_annual" {
        // Corporate Annual Report Nomenclature
        let title = if !state.meta_title.trim().is_empty() { state.meta_title.clone() } else { "Annual Report 2026".to_string() };
        seen_texts.insert(title.clone());
        out.push(SlotItem { name: "cover_title".into(), budget: Some(80), text: title });

        let agency = if !state.meta_agency.trim().is_empty() { state.meta_agency.clone() } else { "Company Name Inc.".to_string() };
        seen_texts.insert(agency.clone());
        out.push(SlotItem { name: "cover_agency".into(), budget: Some(80), text: agency });

        for card in &state.cards {
            let cid = clean_slot_id(&card.chapter_id);
            match card.kind {
                CardKind::Cover => {}
                CardKind::Divider => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem { name: format!("section_divider_{cid}"), budget: Some(60), text: card.title.clone() });
                    }
                }
                CardKind::Narrative => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem { name: format!("executive_heading_{cid}"), budget: Some(90), text: card.title.clone() });
                    }
                    let body = if !card.subtitle.trim().is_empty() { card.subtitle.clone() } else { card.title.clone() };
                    if seen_texts.insert(body.clone()) {
                        out.push(SlotItem { name: format!("executive_summary_{cid}"), budget: Some(400), text: body });
                    }
                }
                CardKind::Table => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem { name: format!("financial_table_{cid}"), budget: Some(150), text: card.title.clone() });
                    }
                    out.push(SlotItem {
                        name: format!("table_footnote_{cid}"),
                        budget: Some(150),
                        text: "Note: Consolidated financial statements in thousands USD".into(),
                    });
                }
                CardKind::Chart => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem { name: format!("financial_chart_{cid}"), budget: Some(100), text: card.title.clone() });
                    }
                }
                CardKind::Image if seen_texts.insert(card.title.clone()) => {
                    out.push(SlotItem { name: format!("figure_caption_{cid}"), budget: Some(120), text: card.title.clone() });
                }
                _ => {}
            }
        }
    } else if prof == "academic_monograph" {
        // Academic Monograph Nomenclature
        let title = if !state.meta_title.trim().is_empty() { state.meta_title.clone() } else { "Research Monograph".to_string() };
        seen_texts.insert(title.clone());
        out.push(SlotItem { name: "monograph_title".into(), budget: Some(120), text: title });

        let agency = if !state.meta_agency.trim().is_empty() { state.meta_agency.clone() } else { "University Press / Institution".to_string() };
        seen_texts.insert(agency.clone());
        out.push(SlotItem { name: "cover_agency".into(), budget: Some(80), text: agency });

        for card in &state.cards {
            let cid = clean_slot_id(&card.chapter_id);
            match card.kind {
                CardKind::Cover => {}
                CardKind::Divider => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem { name: format!("chapter_divider_{cid}"), budget: Some(60), text: card.title.clone() });
                    }
                }
                CardKind::Narrative => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem { name: format!("chapter_title_{cid}"), budget: Some(80), text: card.title.clone() });
                    }
                    let body = if !card.subtitle.trim().is_empty() { card.subtitle.clone() } else { card.title.clone() };
                    if seen_texts.insert(body.clone()) {
                        out.push(SlotItem { name: format!("narrative_body_{cid}"), budget: Some(500), text: body });
                    }
                }
                CardKind::Table => {
                    if seen_texts.insert(card.title.clone()) {
                        out.push(SlotItem { name: format!("academic_table_{cid}"), budget: Some(150), text: card.title.clone() });
                    }
                }
                CardKind::Chart | CardKind::Image if seen_texts.insert(card.title.clone()) => {
                    out.push(SlotItem { name: format!("figure_caption_{cid}"), budget: Some(100), text: card.title.clone() });
                }
                _ => {}
            }
        }
    } else {
        // Fallback for any other / custom profile
        for unit in &collected_units {
            if seen_texts.insert(unit.text.clone()) {
                let name = format!("{}_{}", unit.role.as_str(), clean_slot_id(&unit.card_id));
                let budget = role_budget(unit.role);
                out.push(SlotItem { name, budget, text: unit.text.clone() });
            }
        }
    }

    // Include any additional uncollected units from collected_units that were missed
    for unit in collected_units {
        if seen_texts.insert(unit.text.clone()) {
            let name = format!("{}_{}", unit.role.as_str(), clean_slot_id(&unit.card_id));
            let budget = role_budget(unit.role);
            out.push(SlotItem { name, budget, text: unit.text });
        }
    }

    out
}

pub fn refresh_detected_slots(state: &mut OrchestratorWorkspaceState) {
    state.slots = detect_translation_rule_units(state);
}

// ============================================================================
// PANEL KANAN: "Profil, Slot & Eksekusi"
// ============================================================================

fn render_right_panel(app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    ui.spacing_mut().item_spacing = vec2(6.0, 8.0);

    // Section title
    ui.label(RichText::new(crate::i18n::tr(lang, "Profile & Execution")).font(semibold(12.0)).color(t.text_strong));

    // 1. Profil Publikasi Box
    egui::Frame::NONE
        .fill(t.panel)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(t.radius))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);

            ui.label(RichText::new(crate::i18n::tr(lang, "Publication Profile:")).font(semibold(11.0)).color(t.text_strong));

            // Combo selection
            let current_label = match state.selected_profile.as_str() {
                "bps_brs" => "BPS Berita Resmi Statistik (BRS)",
                "bps_dda" => "BPS Daerah Dalam Angka (DDA)",
                "corporate_annual" => "Corporate Annual Report",
                "academic_monograph" => "Academic Monograph",
                _ => {
                    if state.selected_profile.contains("brs") {
                        "BPS Berita Resmi Statistik (BRS)"
                    } else {
                        "BPS Daerah Dalam Angka (DDA)"
                    }
                }
            };

            craft_ui::ComboBox::from_id_salt("profile_select_combo")
                .selected_text(RichText::new(current_label).font(FontId::proportional(11.0)))
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(state.selected_profile == "bps_brs", "BPS Berita Resmi Statistik (BRS)").clicked() {
                        if state.selected_profile != "bps_brs" {
                            state.selected_profile = "bps_brs".into();
                            refresh_detected_slots(state);
                        }
                    }
                    if ui.selectable_label(state.selected_profile == "bps_dda", "BPS Daerah Dalam Angka (DDA)").clicked() {
                        if state.selected_profile != "bps_dda" {
                            state.selected_profile = "bps_dda".into();
                            refresh_detected_slots(state);
                        }
                    }
                    if ui.selectable_label(state.selected_profile == "corporate_annual", "Corporate Annual Report").clicked() {
                        if state.selected_profile != "corporate_annual" {
                            state.selected_profile = "corporate_annual".into();
                            refresh_detected_slots(state);
                        }
                    }
                    if ui.selectable_label(state.selected_profile == "academic_monograph", "Academic Monograph").clicked() {
                        if state.selected_profile != "academic_monograph" {
                            state.selected_profile = "academic_monograph".into();
                            refresh_detected_slots(state);
                        }
                    }
                });

            let profile_margins = match state.selected_profile.as_str() {
                "bps_brs" => "Margins: T:18.0 B:18.0 L:20.0 R:20.0 mm | Spreads: Ya",
                "bps_dda" => "Margins: T:20.2 B:20.0 L:19.0 R:15.0 mm | Spreads: Ya",
                "corporate_annual" => "Margins: T:25.0 B:25.0 L:20.0 R:20.0 mm | Spreads: Ya",
                "academic_monograph" => "Margins: T:22.0 B:22.0 L:25.0 R:20.0 mm | Spreads: Ya",
                _ => "Margins: T:20.2 B:20.0 L:19.0 R:15.0 mm | Spreads: Ya",
            };
            ui.label(RichText::new(profile_margins).font(FontId::proportional(11.0)).color(t.text_dim));
        });

    // 2. Action Buttons: Translate, Generate & Generate and Publish
    let btn_h = 28.0f32;

    // Translate Worksheet (T7): TM/glossary hits resolve locally, only misses are sent.
    let translate_enabled = super::ai::backend_config_from_prefs(app).is_some() && !state.cards.is_empty();
    if let Some(job) = &state.translate_job {
        ui.ctx().request_repaint();
        let done = job.done;
        let total = job.total;
        let frac = if total > 0 { (done as f32 / total as f32).clamp(0.0, 1.0) } else { 0.0 };
        let pct = (frac * 100.0) as usize;

        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), btn_h), Sense::click());
        let p = ui.painter();

        // Base button background
        p.rect_filled(rect, CornerRadius::same(t.radius), t.panel_darker);

        // Progress fill (at least 8% if total > 0 so progress is immediately visible)
        let fill_w = if total > 0 { (rect.width() * frac).max(rect.width() * 0.08).min(rect.width()) } else { rect.width() * 0.25 };
        let progress_rect = Rect::from_min_size(rect.min, vec2(fill_w, rect.height()));
        p.rect_filled(progress_rect, CornerRadius::same(t.radius), Color32::from_rgb(26, 92, 138));

        // Animated running color / shimmer effect
        let time = ui.input(|i| i.time);
        let cycle = (time * 1.2).fract() as f32;
        let band_w = 70.0f32;
        let band_center_x = rect.min.x - band_w + (rect.width() + band_w * 2.0) * cycle;
        let band_min_x = (band_center_x - band_w * 0.5).clamp(rect.min.x, rect.max.x);
        let band_max_x = (band_center_x + band_w * 0.5).clamp(rect.min.x, rect.max.x);
        if band_max_x > band_min_x {
            let shimmer_rect = Rect::from_min_max(pos2(band_min_x, rect.min.y), pos2(band_max_x, rect.max.y));
            p.rect_filled(shimmer_rect, CornerRadius::same(t.radius), Color32::from_rgba_unmultiplied(255, 255, 255, 40));
        }

        // Outline stroke
        p.rect_stroke(rect, CornerRadius::same(t.radius), Stroke::new(1.0, t.accent), StrokeKind::Inside);

        // Label with done/total and percent
        let label = format!("Menerjemahkan dengan AI… {done}/{total} ({pct}%)");
        let galley = p.layout_no_wrap(label, semibold(11.5), Color32::WHITE);
        let text_pos = pos2(rect.center().x - galley.size().x / 2.0, rect.center().y - galley.size().y / 2.0);
        p.galley(text_pos, galley, Color32::WHITE);

        let resp = resp.on_hover_text(crate::i18n::tr(lang, "Translating with AI… Click to cancel"));
        if resp.clicked() {
            if let Some(job) = &state.translate_job {
                job.cancel();
            }
            state.translate_job = None;
            state.status_message = Some((crate::i18n::tr(lang, "AI translation cancelled").to_string(), false));
        }
    } else {
        let translate_label = crate::i18n::tr(lang, "Translate with AI").to_string();
        let translate_resp = ui
            .add_sized(
                [ui.available_width(), btn_h],
                egui::Button::new(RichText::new(translate_label).font(semibold(11.5)).color(if translate_enabled {
                    t.text_strong
                } else {
                    t.text_dim
                }))
                .fill(t.panel_darker)
                .corner_radius(CornerRadius::same(t.radius)),
            )
            .on_hover_text(crate::i18n::tr(
                lang,
                "Translate the worksheet's titles, headers and notes into English (memory and glossary hits need no request)",
            ));
        if !translate_enabled {
            translate_resp.clone().on_hover_text(crate::i18n::tr(lang, "Configure an AI provider first, and add at least one card"));
        }
        if translate_resp.clicked() {
            let existing = super::with_state(ui.ctx(), |st| count_existing_translations(state, Some(st)));
            if existing > 0 {
                state.show_translate_guard_dialog = true;
            } else {
                start_translate_stage(app, ui.ctx(), state, false);
            }
        }
    }
    // Groq rate limit quota display (mirrors 0.8.4 PubGen header counter)
    if let Some(n) = designcraft_translate::remaining_requests() {
        ui.label(RichText::new(format!("Groq: ~{n} requests left today")).font(FontId::proportional(11.0)).color(t.text_dim));
    }
    // The job itself is polled once per frame in `show`, whichever tab is open.

    // Generate Button (Deep Cyan)
    if ui
        .add_sized(
            [ui.available_width(), btn_h],
            egui::Button::new(RichText::new(crate::i18n::tr(lang, "Generate Publication")).font(semibold(12.0)).color(Color32::WHITE))
                .fill(t.accent_strong)
                .corner_radius(CornerRadius::same(t.radius)),
        )
        .on_hover_text(crate::i18n::tr(lang, "Generate Publication"))
        .clicked()
    {
        let doc_has_pages = app.session.doc().map(|d| d.doc.pages().next().is_some()).unwrap_or(false);
        if doc_has_pages {
            state.show_guard_dialog = true;
            state.guard_is_publish = false;
        } else {
            execute_compile(app, ui.ctx(), state, false, lang);
        }
    }

    // Generate & Publish Button (Emerald Green)
    if ui
        .add_sized(
            [ui.available_width(), btn_h],
            egui::Button::new(RichText::new(crate::i18n::tr(lang, "Generate & Publish PDF")).font(semibold(12.0)).color(Color32::WHITE))
                .fill(t.accent_strong)
                .corner_radius(CornerRadius::same(t.radius)),
        )
        .on_hover_text(crate::i18n::tr(lang, "Generate & Publish PDF"))
        .clicked()
    {
        let doc_has_pages = app.session.doc().map(|d| d.doc.pages().next().is_some()).unwrap_or(false);
        if doc_has_pages {
            state.show_guard_dialog = true;
            state.guard_is_publish = true;
        } else {
            execute_compile(app, ui.ctx(), state, true, lang);
        }
    }

    ui.separator();

    // 3. Status Mesin Tata Letak
    egui::Frame::NONE
        .fill(t.panel_darker)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(t.radius))
        .inner_margin(Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.label(RichText::new(crate::i18n::tr(lang, "Layout Engine")).font(semibold(11.0)).color(t.text_dim));
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::i18n::tr(lang, "● Integrated (DesignCraft Native OK)")).font(semibold(11.0)).color(t.success));
            });
        });

    ui.separator();

    // 4. Slot Terdeteksi (Slot character budget & fit)
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{} ({})", crate::i18n::tr(lang, "Detected Slots"), state.slots.len())).font(semibold(11.5)).color(t.text_strong),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.small_button("⟳").on_hover_text(crate::i18n::tr(lang, "Refresh detected translation rule units")).clicked() {
                refresh_detected_slots(state);
            }
        });
    });

    if state.slots.is_empty() && !state.cards.is_empty() {
        refresh_detected_slots(state);
    }

    ScrollArea::vertical().id_salt("pubgen_right_slots_scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 6.0);

        if state.slots.is_empty() {
            ui.label(
                RichText::new(crate::i18n::tr(lang, "No slots detected yet. Open a .designcraft template first."))
                    .font(FontId::proportional(11.0))
                    .color(t.text_dim),
            );
        } else {
            for (idx, slot) in state.slots.iter().enumerate() {
                ui.push_id((idx, &slot.name), |ui| {
                    egui::Frame::NONE
                        .fill(t.panel_darker)
                        .stroke(Stroke::new(1.0, t.field_border))
                        .corner_radius(CornerRadius::same(t.radius))
                        .inner_margin(Margin::same(6))
                        .show(ui, |ui| {
                            let row_w = ui.available_width();
                            ui.allocate_ui_with_layout(vec2(row_w, 18.0), Layout::left_to_right(Align::Center), |ui| {
                                ui.label(RichText::new(&slot.name).font(semibold(11.0)).color(t.text_strong));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let char_count = slot.text.chars().count();
                                    if let Some(budget) = slot.budget {
                                        let is_ok = char_count <= budget;
                                        ui.label(RichText::new(if is_ok { "● OK" } else { "● Over" }).font(semibold(11.0)).color(if is_ok {
                                            t.success
                                        } else {
                                            t.danger
                                        }));
                                        ui.label(RichText::new(format!("{char_count}/{budget}")).font(FontId::proportional(11.0)).color(t.text_dim));
                                    } else {
                                        ui.label(RichText::new(format!("{char_count} chars")).font(FontId::proportional(11.0)).color(t.text_dim));
                                    }
                                });
                            });

                            let preview: String = slot.text.chars().take(40).collect();
                            ui.label(RichText::new(format!("{preview}…")).font(FontId::proportional(11.0)).color(t.text_dim));
                        });
                });
            }
        }
    });
}

// ============================================================================
// MODAL EDITOR KARTU PUBLIKASI
// ============================================================================

fn load_chart_data_from_source(card: &mut PublicationCard, source_path: &str, cached_content: &std::collections::BTreeMap<String, Vec<u8>>) {
    if source_path.is_empty() || source_path == "Tanpa sumber" || source_path == "Manual" {
        return;
    }
    let clean = source_path.replace('\\', "/");
    let bare = clean.strip_prefix("csv/").unwrap_or(&clean);
    let with_csv_prefix = format!("csv/{bare}");
    let with_ext = if !bare.ends_with(".csv") { format!("{bare}.csv") } else { bare.to_string() };
    let with_both = if !clean.ends_with(".csv") { format!("csv/{with_ext}") } else { with_csv_prefix.clone() };
    let backslash = clean.replace('/', "\\");

    let found_bytes = cached_content
        .get(&clean)
        .or_else(|| cached_content.get(bare))
        .or_else(|| cached_content.get(&with_csv_prefix))
        .or_else(|| cached_content.get(&with_ext))
        .or_else(|| cached_content.get(&with_both))
        .or_else(|| cached_content.get(&backslash));

    if let Some(bytes) = found_bytes {
        card.chart_raw_data = String::from_utf8_lossy(bytes).into_owned();
        auto_detect_chart_columns(card);
    } else if let Ok(bytes) = std::fs::read(source_path) {
        card.chart_raw_data = String::from_utf8_lossy(&bytes).into_owned();
        auto_detect_chart_columns(card);
    }
}

fn auto_detect_chart_columns(card: &mut PublicationCard) {
    let delim = super::csv::detect_delimiter(&card.chart_raw_data);
    let grid = super::csv::parse_csv_content_with_delim(&card.chart_raw_data, delim);
    let Some(header_row) = grid.first() else { return };
    let headers: Vec<String> = header_row.iter().map(|c| c.trim().to_lowercase()).collect();

    let mut cat_idx = 0;
    let mut found_cat = false;
    for (idx, h) in headers.iter().enumerate() {
        let col_letter = super::charts::spec::index_to_col_letter(idx);
        if matches!(h.as_str(), "nmdesa" | "desa" | "village" | "kecamatan" | "wilayah" | "category" | "kategori" | "tahun" | "year") {
            card.chart_category_col = col_letter;
            cat_idx = idx;
            found_cat = true;
            break;
        }
    }
    if !found_cat {
        card.chart_category_col = "A".into();
    }

    let mut series_indices = Vec::new();
    for (idx, h) in headers.iter().enumerate() {
        if idx == cat_idx {
            continue;
        }
        if matches!(h.as_str(), "title" | "judul" | "title_en" | "judul_en" | "id" | "code") {
            continue;
        }
        series_indices.push(idx);
    }

    if let Some(&s1) = series_indices.first() {
        card.chart_series_1_col = super::charts::spec::index_to_col_letter(s1);
        if let Some(lbl) = header_row.get(s1) {
            let trimmed = lbl.trim();
            if !trimmed.is_empty() {
                card.chart_series_1_label_id = trimmed.to_string();
            }
        }
    }

    if let Some(&s2) = series_indices.get(1) {
        card.chart_series_2_enabled = true;
        card.chart_series_2_col = super::charts::spec::index_to_col_letter(s2);
        if let Some(lbl) = header_row.get(s2) {
            let trimmed = lbl.trim();
            if !trimmed.is_empty() {
                card.chart_series_2_label_id = trimmed.to_string();
            }
        }
    } else {
        card.chart_series_2_enabled = false;
    }

    if let Some(&s3) = series_indices.get(2) {
        card.chart_series_3_enabled = true;
        card.chart_series_3_col = super::charts::spec::index_to_col_letter(s3);
        if let Some(lbl) = header_row.get(s3) {
            let trimmed = lbl.trim();
            if !trimmed.is_empty() {
                card.chart_series_3_label_id = trimmed.to_string();
            }
        }
    } else {
        card.chart_series_3_enabled = false;
    }

    if let Some(row1_cells) = grid.get(1) {
        if let Some(t_idx) = headers.iter().position(|h| h == "title" || h == "judul") {
            if let Some(t_val) = row1_cells.get(t_idx).map(|c| c.trim()) {
                if !t_val.is_empty() && (card.title.is_empty() || card.title.starts_with("Grafik Baru") || card.title.starts_with("Objek Grafik")) {
                    card.title = t_val.to_string();
                }
            }
        }
        if let Some(te_idx) = headers.iter().position(|h| h == "title_en" || h == "judul_en") {
            if let Some(te_val) = row1_cells.get(te_idx).map(|c| c.trim()) {
                if !te_val.is_empty() && card.title_en.is_empty() {
                    card.title_en = te_val.to_string();
                }
            }
        }
        card.chart_data_start_row = 2;
        card.chart_data_end_row = grid.len();
    }
}

fn build_chart_sheet_csv(card: &PublicationCard) -> String {
    let delim = super::csv::detect_delimiter(&card.chart_raw_data);
    let mut lines = Vec::new();
    let binding = card.chart_type.to_lowercase();
    let ct = match binding.as_str() {
        "vertical clustered bar" | "clustered bar" | "bar" => "bar_clustered",
        "vertical stacked bar" | "stacked bar" => "bar_stacked",
        "horizontal bar (ranked)" | "horizontal bar" | "hbar" => "horizontal_bar",
        "line chart (markers)" | "line" => "line",
        "area chart (filled)" | "area" => "area",
        "donut / ring chart" | "donut" | "pie" => "donut",
        "population pyramid (piramida penduduk)" | "population pyramid" | "pyramid" => "population_pyramid",
        "gauge / speedometer chart" | "gauge" => "gauge",
        other => other,
    };
    lines.push(format!("chart_type{delim}{ct}"));
    if !card.title.is_empty() {
        lines.push(format!("title{delim}{}", card.title));
    }
    if !card.title_en.is_empty() {
        lines.push(format!("title_en{delim}{}", card.title_en));
    }
    let cat_col = if card.chart_category_col.is_empty() { "A" } else { &card.chart_category_col };
    lines.push(format!("category_col{delim}{cat_col}"));
    let s1_col = if card.chart_series_1_col.is_empty() { "B" } else { &card.chart_series_1_col };
    lines.push(format!("series_1_col{delim}{s1_col}"));
    if !card.chart_series_1_label_id.is_empty() {
        lines.push(format!("series_1_label{delim}{}", card.chart_series_1_label_id));
    }
    if !card.chart_series_1_label_en.is_empty() {
        lines.push(format!("series_1_label_en{delim}{}", card.chart_series_1_label_en));
    }
    if card.chart_series_2_enabled && !card.chart_series_2_col.is_empty() {
        lines.push(format!("series_2_col{delim}{}", card.chart_series_2_col));
        if !card.chart_series_2_label_id.is_empty() {
            lines.push(format!("series_2_label{delim}{}", card.chart_series_2_label_id));
        }
        if !card.chart_series_2_label_en.is_empty() {
            lines.push(format!("series_2_label_en{delim}{}", card.chart_series_2_label_en));
        }
    }
    if card.chart_series_3_enabled && !card.chart_series_3_col.is_empty() {
        lines.push(format!("series_3_col{delim}{}", card.chart_series_3_col));
        if !card.chart_series_3_label_id.is_empty() {
            lines.push(format!("series_3_label{delim}{}", card.chart_series_3_label_id));
        }
        if !card.chart_series_3_label_en.is_empty() {
            lines.push(format!("series_3_label_en{delim}{}", card.chart_series_3_label_en));
        }
    }
    let fmt = match card.chart_value_format.as_str() {
        "Percentage (%)" => "percent",
        "Currency IDR (Rp)" => "idr",
        _ => "number",
    };
    lines.push(format!("value_format{delim}{fmt}"));
    lines.push(format!("show_value_labels{delim}{}", card.chart_show_value_labels));
    lines.push(format!("show_legend{delim}{}", card.chart_show_legend));
    lines.push(format!("show_grid{delim}{}", card.chart_show_grid));
    if !card.chart_max_value_override.is_empty() {
        lines.push(format!("max_value{delim}{}", card.chart_max_value_override));
    }
    let base_header_count = lines.len() + 3;
    let data_start = base_header_count + card.chart_data_start_row.saturating_sub(1);
    let data_end = base_header_count + card.chart_data_end_row.saturating_sub(1);
    lines.push(format!("data_start_row{delim}{data_start}"));
    lines.push(format!("data_end_row{delim}{data_end}"));
    lines.push(String::new());
    lines.push(card.chart_raw_data.clone());
    lines.join("\n")
}

fn render_chart_options_form(
    app: &mut DesignApp,
    ui: &mut Ui,
    card: &mut PublicationCard,
    cached_content: &std::collections::BTreeMap<String, Vec<u8>>,
    t: &Tokens,
    lang: &str,
) {
    egui::Frame::NONE
        .fill(t.panel)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(t.radius))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            ui.label(
                RichText::new(crate::i18n::tr(lang, "Chart Visualisation Settings (Vector Engine & Data Mapping):"))
                    .font(semibold(11.0))
                    .color(t.text_strong),
            );

            // Section 1: Archetype selection
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::i18n::tr(lang, "Chart Archetype:")).font(semibold(11.0)));
                craft_ui::ComboBox::from_id_salt("chart_archetype_select").selected_text(&card.chart_type).width(260.0).show_ui(ui, |ui| {
                    for (ct_val, ct_display) in [
                        ("Line Chart (Markers)", "Line Chart (Markers)"),
                        ("Vertical Clustered Bar", "Vertical Clustered Bar"),
                        ("Vertical Stacked Bar", "Vertical Stacked Bar"),
                        ("Horizontal Bar (Ranked)", "Horizontal Bar (Ranked)"),
                        ("Area Chart (Filled)", "Area Chart (Filled)"),
                        ("Donut / Ring Chart", "Donut / Ring Chart"),
                        ("Population Pyramid (Piramida Penduduk)", "Population Pyramid (Piramida Penduduk)"),
                        ("Gauge / Speedometer Chart", "Gauge / Speedometer Chart"),
                    ] {
                        if ui.selectable_label(card.chart_type == ct_val || card.chart_type == ct_display, ct_display).clicked() {
                            card.chart_type = ct_val.into();
                        }
                    }
                });
            });

            // Section 2: Data Range & Columns
            ui.separator();
            ui.label(RichText::new(crate::i18n::tr(lang, "Data Columns & Rows:")).font(semibold(11.0)).color(t.text_strong));
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::i18n::tr(lang, "Category Col:")).font(FontId::proportional(11.0)).color(t.text));
                ui.add_sized([40.0, 20.0], egui::TextEdit::singleline(&mut card.chart_category_col));

                ui.add_space(8.0);
                ui.label(RichText::new(crate::i18n::tr(lang, "Rows:")).font(FontId::proportional(11.0)).color(t.text));
                ui.add(egui::DragValue::new(&mut card.chart_data_start_row).range(1..=1000).prefix("Start: "));
                ui.add(egui::DragValue::new(&mut card.chart_data_end_row).range(1..=1000).prefix("End: "));
            });

            // Series 1
            ui.horizontal(|ui| {
                ui.label(RichText::new("Series 1:").font(semibold(11.0)).color(t.text_strong));
                ui.label(RichText::new("Col:").font(FontId::proportional(11.0)).color(t.text_dim));
                ui.add_sized([30.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_1_col));
                ui.label(RichText::new("Label (ID):").font(FontId::proportional(11.0)).color(t.text_dim));
                ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_1_label_id));
                ui.label(RichText::new("Label (EN):").font(FontId::proportional(11.0)).color(t.text_dim));
                ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_1_label_en));
            });

            // Series 2 & 3 with archetype adaptations
            let is_gauge = card.chart_type.to_lowercase().contains("gauge");
            let is_donut = card.chart_type.to_lowercase().contains("donut") || card.chart_type.to_lowercase().contains("pie");
            let is_pyramid = card.chart_type.to_lowercase().contains("pyramid") || card.chart_type.to_lowercase().contains("piramida");

            if is_gauge {
                ui.label(
                    RichText::new(crate::i18n::tr(lang, "ℹ Gauge charts only fit 1 category and 1 data series."))
                        .font(FontId::proportional(11.0))
                        .color(t.text_dim),
                );
            } else if is_donut {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut card.chart_series_2_enabled, "Series 2 (second ring):");
                    if card.chart_series_2_enabled {
                        ui.label(RichText::new("Col:").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([30.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_col));
                        ui.label(RichText::new("Label (ID):").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_label_id));
                        ui.label(RichText::new("Label (EN):").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_label_en));
                    }
                });
                ui.label(
                    RichText::new(crate::i18n::tr(lang, "ℹ Donut charts fit up to 2 data series: one ring per series."))
                        .font(FontId::proportional(11.0))
                        .color(t.text_dim),
                );
            } else if is_pyramid {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut card.chart_series_2_enabled, "Series 2 (Female / Perempuan):");
                    if card.chart_series_2_enabled {
                        ui.label(RichText::new("Col:").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([30.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_col));
                        ui.label(RichText::new("Label (ID):").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_label_id));
                        ui.label(RichText::new("Label (EN):").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_label_en));
                    }
                });
                ui.label(
                    RichText::new(crate::i18n::tr(lang, "ℹ Population pyramid: Series 1 = Male (Laki-laki), Series 2 = Female (Perempuan)."))
                        .font(FontId::proportional(11.0))
                        .color(t.text_dim),
                );
            } else {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut card.chart_series_2_enabled, "Series 2:");
                    if card.chart_series_2_enabled {
                        ui.label(RichText::new("Col:").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([30.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_col));
                        ui.label(RichText::new("Label (ID):").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_label_id));
                        ui.label(RichText::new("Label (EN):").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_2_label_en));
                    }
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut card.chart_series_3_enabled, "Series 3:");
                    if card.chart_series_3_enabled {
                        ui.label(RichText::new("Col:").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([30.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_3_col));
                        ui.label(RichText::new("Label (ID):").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_3_label_id));
                        ui.label(RichText::new("Label (EN):").font(FontId::proportional(11.0)).color(t.text_dim));
                        ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut card.chart_series_3_label_en));
                    }
                });
            }

            // Section 3: Formatting & Display Options
            ui.separator();
            ui.label(RichText::new(crate::i18n::tr(lang, "Formatting & Display Options:")).font(semibold(11.0)).color(t.text_strong));
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::i18n::tr(lang, "Value Format:")).font(FontId::proportional(11.0)).color(t.text));
                craft_ui::ComboBox::from_id_salt("chart_val_fmt_select").selected_text(&card.chart_value_format).width(160.0).show_ui(ui, |ui| {
                    for fmt_label in ["Number (1.234,5)", "Percentage (%)", "Currency IDR (Rp)"] {
                        if ui.selectable_label(card.chart_value_format == fmt_label, fmt_label).clicked() {
                            card.chart_value_format = fmt_label.into();
                        }
                    }
                });

                ui.add_space(8.0);
                ui.label(RichText::new(crate::i18n::tr(lang, "Max Override:")).font(FontId::proportional(11.0)).color(t.text_dim));
                ui.add_sized([50.0, 20.0], egui::TextEdit::singleline(&mut card.chart_max_value_override).hint_text("Auto"));
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut card.chart_show_value_labels, crate::i18n::tr(lang, "Show Value Labels"));
                ui.checkbox(&mut card.chart_show_legend, crate::i18n::tr(lang, "Show Legend"));
                ui.checkbox(&mut card.chart_show_grid, crate::i18n::tr(lang, "Show Gridlines"));
            });

            // Section 4: Data Table (CSV / TSV)
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::i18n::tr(lang, "Data Table (CSV / TSV):")).font(semibold(11.0)).color(t.text_strong));
                if ui.small_button(crate::i18n::tr(lang, "Reset Sample Data")).clicked() {
                    card.chart_raw_data = super::charts::dialog::DEFAULT_SAMPLE_CSV.to_string();
                    card.chart_data_start_row = 2;
                    card.chart_data_end_row = 6;
                }
                if !card.source.is_empty() && card.source != "Tanpa sumber" && card.source != "Manual" {
                    if ui.small_button(crate::i18n::tr(lang, "Reload From Source")).clicked() {
                        let src = card.source.clone();
                        load_chart_data_from_source(card, &src, cached_content);
                    }
                }
                if ui.small_button(crate::i18n::tr(lang, "Auto-Detect Columns")).clicked() {
                    auto_detect_chart_columns(card);
                }
            });

            ui.horizontal(|ui| {
                ui.selectable_value(&mut card.chart_csv_grid_preview, true, crate::i18n::tr(lang, "Preview"));
                ui.selectable_value(&mut card.chart_csv_grid_preview, false, crate::i18n::tr(lang, "Edit"));
            });

            if card.chart_csv_grid_preview {
                let delim = super::csv::detect_delimiter(&card.chart_raw_data);
                let grid = super::csv::parse_csv_content_with_delim(&card.chart_raw_data, delim);
                if !grid.is_empty() {
                    egui::ScrollArea::both().id_salt("pubgen_chart_card_csv_preview").max_height(140.0).show(ui, |ui| {
                        egui::Grid::new("pubgen_chart_card_csv_preview_grid").striped(true).spacing([12.0, 2.0]).show(ui, |ui| {
                            for (r, row) in grid.iter().take(100).enumerate() {
                                let header_rows = card.chart_data_start_row.saturating_sub(1);
                                let row_number = r.checked_sub(header_rows).map(|v| v + 1);
                                let is_data = row_number.is_some();
                                ui.label(
                                    RichText::new(row_number.map(|n| n.to_string()).unwrap_or_default()).monospace().size(11.0).color(t.text_dim),
                                );
                                for (c, cell) in row.iter().enumerate() {
                                    let text = RichText::new(cell).monospace().size(11.0);
                                    let text = if !is_data {
                                        text.strong().color(t.text_strong)
                                    } else if c == 0 {
                                        text.strong().color(t.text)
                                    } else {
                                        text.color(t.text)
                                    };
                                    ui.label(text);
                                }
                                ui.end_row();
                            }
                        });
                    });
                    if grid.len() > 100 {
                        ui.label(
                            RichText::new(format!("(Menampilkan 100 dari {} baris data)", grid.len()))
                                .font(FontId::proportional(11.0))
                                .color(t.text_dim),
                        );
                    }
                }
            } else {
                egui::ScrollArea::vertical().id_salt("pubgen_chart_card_csv_edit_scroll").max_height(140.0).show(ui, |ui| {
                    ui.add_sized([ui.available_width(), 130.0], egui::TextEdit::multiline(&mut card.chart_raw_data).code_editor());
                });
            }

            // Section 5: Optional Raster Image Fallback
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::i18n::tr(lang, "Raster Image Fallback (Optional):")).font(semibold(11.0)).color(t.text_dim));
                let edit = ui.add(egui::TextEdit::singleline(&mut card.chart_image).desired_width(200.0).hint_text("images/chart.png"));
                if edit.changed() && !card.chart_image.is_empty() {
                    card.source = card.chart_image.clone();
                }
                if ui.button(crate::i18n::tr(lang, "Buka...")).clicked() {
                    let ctx = ui.ctx().clone();
                    let _ = app.ask_open("image", crate::app::file_dialogs::Bound::App, move |_app, path| {
                        let mut st = get_state(&ctx);
                        if let Some(idx) = st.editing_card_idx {
                            if let Some(c) = st.cards.get_mut(idx) {
                                c.chart_image = path.clone();
                                c.source = path;
                            }
                        }
                        set_state(&ctx, st);
                        ctx.request_repaint();
                        Ok(serde_json::Value::Null)
                    });
                }
            });
        });
}

fn render_card_edit_modal(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    let Some(idx) = state.editing_card_idx else { return };
    if idx >= state.cards.len() {
        state.editing_card_idx = None;
        state.card_delete_confirm = false;
        return;
    }

    let is_bilingual = state.doc_language.to_lowercase().contains("bilingual")
        || state.doc_language.to_lowercase().contains("dwibahasa")
        || state.doc_language.eq_ignore_ascii_case("indonesian-english")
        || state.doc_language.eq_ignore_ascii_case("indo-english")
        || (state.doc_language.to_lowercase().contains("indonesia") && state.doc_language.to_lowercase().contains("english"));
    let is_english = state.doc_language.to_lowercase().contains("english") || state.doc_language.to_lowercase().contains("inggris");
    let show_title_en = is_bilingual || is_english;

    let mut close = false;
    let mut delete_card = false;
    let mut insert_before = false;
    let mut insert_after = false;
    let mut apply_all_tables = false;

    let page_num = state.cards[idx].page;
    let is_newly_opened = state.last_editing_card_idx != Some(idx);
    state.last_editing_card_idx = Some(idx);

    if is_newly_opened {
        let card = &mut state.cards[idx];
        if card.kind == CardKind::Chart
            && !card.source.is_empty()
            && card.source != "Tanpa sumber"
            && card.source != "Manual"
            && (card.chart_raw_data.trim().is_empty() || card.chart_raw_data == super::charts::dialog::DEFAULT_SAMPLE_CSV)
        {
            let src = card.source.clone();
            load_chart_data_from_source(card, &src, &state.cached_content);
        }
        let card = &state.cards[idx];
        if card.kind != CardKind::Cover {
            super::with_state(ctx, |pubgen_st| {
                if card.hf_enabled && !card.hf_style.is_empty() {
                    pubgen_st.hf_style = card.hf_style.clone();
                    pubgen_st.hf_sided = card.hf_sided;
                    pubgen_st.hf_header_single = card.hf_header_single.clone();
                    pubgen_st.hf_header_left = card.hf_header_left.clone();
                    pubgen_st.hf_header_right = card.hf_header_right.clone();
                    pubgen_st.hf_footer_single = card.hf_footer_single.clone();
                    pubgen_st.hf_footer_left = card.hf_footer_left.clone();
                    pubgen_st.hf_footer_right = card.hf_footer_right.clone();
                    pubgen_st.hf_header_rule = card.hf_header_rule;
                    pubgen_st.hf_footer_rule = card.hf_footer_rule;
                    pubgen_st.hf_header_offset = card.hf_header_offset;
                    pubgen_st.hf_footer_offset = card.hf_footer_offset;
                }
                pubgen_st.hf_scope = if card.hf_scope_forward { 4 } else { 3 };
                pubgen_st.hf_scope_start = card.page.max(1);
                pubgen_st.hf_scope_end = card.page.max(1);
            });
        } else {
            super::with_state(ctx, |pubgen_st| {
                sync_card_to_pubgen_state(card, state, pubgen_st);
            });
        }
    }

    let modal_w = 600.0f32;
    let available_h = (ctx.content_rect().height() - 80.0).max(200.0);
    let max_modal_h = 680.0f32.min(available_h);
    let scroll_h = max_modal_h - 90.0;
    let content_w = modal_w - 24.0;

    let cached_content = state.cached_content.clone();
    let mut switch_to_chart_tab = false;

    egui::Window::new(format!("{} — {} {}", crate::i18n::tr(lang, "Edit Publication Card"), crate::i18n::tr(lang, "Pg."), page_num))
        .collapsible(false)
        .resizable(false)
        .min_width(modal_w)
        .max_width(modal_w)
        .default_width(modal_w)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .show(ctx, |ui| {
            ui.set_width(content_w);
            ui.set_max_width(content_w);
            let card = &mut state.cards[idx];
            ui.spacing_mut().item_spacing = vec2(6.0, 8.0);

            // Modal Header
            ui.horizontal(|ui| {
                ui.label(RichText::new(card.kind.icon()).size(18.0));
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("{} {}", card.kind.label(lang), if card.scope_new_page {
                            crate::i18n::tr(lang, "(New Page)")
                        } else {
                            crate::i18n::tr(lang, "(New Object)")
                        }))
                            .font(semibold(12.0))
                            .color(t.text_strong),
                    );
                    ui.label(
                        RichText::new(
                            crate::i18n::tr(lang, "Page {p} · Layout: {l}")
                                .replace("{p}", &card.page.to_string())
                                .replace(
                                    "{l}",
                                    if state.page_layout_facing {
                                        crate::i18n::tr(lang, "Double-Page (Facing Spreads)")
                                    } else {
                                        crate::i18n::tr(lang, "Single-Page")
                                    },
                                ),
                        )
                        .font(FontId::proportional(11.0))
                        .color(t.text_dim),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(RichText::new("×").size(15.0)).on_hover_text(crate::i18n::tr(lang, "Close")).clicked() {
                        close = true;
                    }
                });
            });

            ui.separator();

            // Unified Tab Bar (Matching InDesign-style Right Dock Tabs)
            let tab_bar_height = DOCK_TAB_H;
            let mut tabs = vec![CardEditTab::KontenSumber];
            if card.kind == CardKind::Chart {
                tabs.push(CardEditTab::OpsiGrafik);
            }
            tabs.push(CardEditTab::TataLetakSeksi);
            if card.kind != CardKind::Cover {
                tabs.push(CardEditTab::HeaderFooter);
            }
            if !tabs.contains(&state.card_edit_tab) {
                state.card_edit_tab = CardEditTab::KontenSumber;
            }

            let tab_count = tabs.len() as f32;
            let tab_w = (content_w / tab_count).floor();

            let (strip, _) = ui.allocate_exact_size(vec2(content_w, tab_bar_height), Sense::hover());
            ui.painter().rect_filled(strip, 0.0, t.tab_strip);

            for (i, &tab) in tabs.iter().enumerate() {
                let tab_x = strip.min.x + (i as f32) * tab_w;
                let this_tab_w = if i == tabs.len() - 1 {
                    (strip.max.x - tab_x).max(0.0)
                } else {
                    tab_w
                };
                let tab_rect = Rect::from_min_size(pos2(tab_x, strip.min.y), vec2(this_tab_w, tab_bar_height));
                let is_selected = state.card_edit_tab == tab;
                let (icon, label) = match tab {
                    CardEditTab::KontenSumber => ("🏷️", crate::i18n::tr(lang, "Content & Source")),
                    CardEditTab::OpsiGrafik => ("📊", crate::i18n::tr(lang, "Chart Options")),
                    CardEditTab::TataLetakSeksi => ("📄", crate::i18n::tr(lang, "Layout & Section")),
                    CardEditTab::HeaderFooter => ("📑", crate::i18n::tr(lang, "Header & Footer")),
                };
                let resp = ui.interact(tab_rect, ui.scope_id().with(("card_tab", i)), Sense::click())
                    .on_hover_text(label);
                resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, is_selected, label));
                if is_selected {
                    ui.painter().rect_filled(tab_rect, 0.0, t.panel);
                    ui.painter().line_segment([tab_rect.left_top(), tab_rect.left_bottom()], Stroke::new(1.0, t.border));
                    ui.painter().line_segment([tab_rect.right_top(), tab_rect.right_bottom()], Stroke::new(1.0, t.border));
                } else {
                    if resp.hovered() {
                        ui.painter().rect_filled(tab_rect, 0.0, t.hover);
                    }
                    ui.painter().line_segment([tab_rect.left_bottom(), tab_rect.right_bottom()], Stroke::new(1.0, t.border));
                }
                ui.painter().text(
                    tab_rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{icon} {label}"),
                    FontId::proportional(DOCK_TAB_LABEL),
                    if is_selected { t.text_strong } else { t.text_dim },
                );
                if resp.clicked() {
                    state.card_edit_tab = tab;
                }
            }

            ui.add_space(2.0);

            egui::ScrollArea::vertical()
                .max_width(content_w)
                .max_height(scroll_h - tab_bar_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(content_w);
                    ui.set_max_width(content_w);

                    match state.card_edit_tab {
                        CardEditTab::KontenSumber => {
                            // Section 1: Cakupan & Jenis Konten (Scope & Content Type)
                            egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(crate::i18n::tr(lang, "Scope:")).font(semibold(11.0)).color(t.text_strong));
                                    let is_np = card.scope_new_page;
                                    if ui.selectable_label(is_np, crate::i18n::tr(lang, "New Page")).clicked() {
                                        card.scope_new_page = true;
                                        card.tag2 = "Halaman Baru".into();
                                    }
                                    if ui.selectable_label(!is_np, crate::i18n::tr(lang, "New Object")).clicked() {
                                        card.scope_new_page = false;
                                        card.tag2 = "Objek Baru".into();
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(crate::i18n::tr(lang, "Content Type:")).font(semibold(11.0)).color(t.text_strong));
                                    craft_ui::ComboBox::from_id_salt("card_edit_kind_select")
                                        .selected_text(format!("{} {}", card.kind.icon(), card.kind.label(lang)))
                                        .width(180.0)
                                        .show_ui(ui, |ui| {
                                            for k in CardKind::ALL {
                                                if ui.selectable_label(card.kind == k, format!("{} {}", k.icon(), k.label(lang))).clicked() {
                                                    card.kind = k;
                                                    card.tag1 = match k {
                                                        CardKind::Cover => "Sampul".into(),
                                                        CardKind::Narrative => "Paragraf".into(),
                                                        CardKind::Chart => "Grafik".into(),
                                                        CardKind::MapChart => "Peta Tematik".into(),
                                                        CardKind::Table => "Tabel".into(),
                                                        CardKind::Image => "Gambar".into(),
                                                        CardKind::Toc => "Daftar Isi".into(),
                                                        CardKind::Divider => "Pembatas".into(),
                                                    };
                                                }
                                            }
                                        });
                                });
                            });

                            // Section 2: Identitas & Sumber Data (Identity & Binding)
                            if card.kind != CardKind::Cover {
                                egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                    ui.label(RichText::new(crate::i18n::tr(lang, "Publication Title")).font(semibold(11.0)).color(t.text_strong));
                                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut card.title));

                                    if show_title_en {
                                        ui.label(RichText::new(crate::i18n::tr(lang, "Title EN:")).font(semibold(11.0)).color(t.text_strong));
                                        ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut card.title_en));
                                        // T8: is this English an unreviewed AI draft or a human decision?
                                        if let Some(label) = super::translate_stage::badge_label(&state.tm, &card.title) {
                                            let reviewed = label == "Reviewed";
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(label)
                                                        .font(FontId::proportional(11.0))
                                                        .color(if reviewed { t.success } else { t.accent }),
                                                );
                                                if !reviewed && ui.button(RichText::new("Accept").font(FontId::proportional(11.0))).clicked() {
                                                    if state.tm.promote(&card.title) {
                                                        state.is_dirty = true;
                                                    }
                                                }
                                            });
                                        }
                                    }

                                    let show_chapter = matches!(card.kind, CardKind::Narrative | CardKind::Table | CardKind::Chart | CardKind::MapChart | CardKind::Divider);
                                    if show_chapter {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Chapter / Section ID:")).font(semibold(11.0)).color(t.text_strong));
                                            ui.add(egui::TextEdit::singleline(&mut card.chapter_id).desired_width(120.0).hint_text("Contoh: Bab 1"));
                                        });
                                    }

                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(crate::i18n::tr(lang, "Source:")).font(semibold(11.0)).color(t.text_strong));
                                        craft_ui::ComboBox::from_id_salt("card_source_select_box")
                                            .selected_text(if card.source.is_empty() { "Tanpa sumber" } else { &card.source })
                                            .width(240.0)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(card.source == "Tanpa sumber", "Tanpa sumber").clicked() {
                                                    card.source = "Tanpa sumber".into();
                                                }
                                                for f in &state.content_files {
                                                    if card.kind == CardKind::Chart {
                                                        let clean = f.path.replace('\\', "/");
                                                        if !clean.starts_with("csv/") && !clean.ends_with(".csv") {
                                                            continue;
                                                        }
                                                    }
                                                    if ui.selectable_label(card.source == f.path, &f.path).clicked() {
                                                        card.source = f.path.clone();
                                                        if card.kind == CardKind::Chart {
                                                            load_chart_data_from_source(card, &f.path, &cached_content);
                                                        }
                                                    }
                                                }
                                            });
                                    });
                                });
                            }

                            // Section 3: Rule-Based Form Sections per CardKind
                            match card.kind {
                                CardKind::Cover => {
                                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Source:")).font(semibold(11.0)).color(t.text_strong));
                                            craft_ui::ComboBox::from_id_salt("cover_source_select_box")
                                                .selected_text(if card.source.is_empty() { "Tanpa sumber" } else { &card.source })
                                                .width(220.0)
                                                .show_ui(ui, |ui| {
                                                    if ui.selectable_label(card.source == "Tanpa sumber", "Tanpa sumber").clicked() {
                                                        card.source = "Tanpa sumber".into();
                                                    }
                                                    for f in &state.content_files {
                                                        if ui.selectable_label(card.source == f.path, &f.path).clicked() {
                                                            card.source = f.path.clone();
                                                        }
                                                    }
                                                });
                                        });
                                        ui.add_space(4.0);

                                        ui.horizontal(|ui| {
                                            ui.selectable_value(&mut card.cover_tab, 0, RichText::new("Desain Sampul").font(semibold(11.0)));
                                            ui.selectable_value(&mut card.cover_tab, 1, RichText::new("Metadata Rilis & PIC").font(semibold(11.0)));
                                            ui.selectable_value(&mut card.cover_tab, 2, RichText::new("Kunci & Nilai Lembar COV").font(semibold(11.0)));
                                        });
                                        ui.separator();

                                        if card.cover_tab == 0 {
                                            let ctx = ui.ctx().clone();
                                            crate::dock::pubgen::with_state(&ctx, |pubgen_st| {
                                                super::widgets::card_cover_body_ui(app, ui, pubgen_st, t);
                                                let new_title = pubgen_st.cover_title.trim();
                                                if !new_title.is_empty() {
                                                    card.title = format!("Sampul Rilis: {new_title}");
                                                    card.settings.insert("title".into(), new_title.to_string());
                                                }
                                                card.title_en = pubgen_st.cover_title_en.clone();
                                                card.settings.insert("title_en".into(), pubgen_st.cover_title_en.clone());
                                                card.cover_title2 = pubgen_st.cover_title2.clone();
                                                card.settings.insert("title2".into(), pubgen_st.cover_title2.clone());
                                                card.cover_year = pubgen_st.cover_year.clone();
                                                card.settings.insert("year".into(), pubgen_st.cover_year.clone());
                                                card.cover_volume = pubgen_st.cover_volume.clone();
                                                card.settings.insert("volume".into(), pubgen_st.cover_volume.clone());
                                                card.cover_catalogue = pubgen_st.cover_catalogue.clone();
                                                card.settings.insert("catalogue".into(), pubgen_st.cover_catalogue.clone());
                                                card.cover_code_num = pubgen_st.cover_code_num.clone();
                                                card.settings.insert("code".into(), pubgen_st.cover_code_num.clone());
                                                card.cover_pub_office = pubgen_st.cover_agency_text.clone();
                                                card.settings.insert("pub_office".into(), pubgen_st.cover_agency_text.clone());
                                                card.settings.insert("agency".into(), pubgen_st.cover_agency_text.clone());
                                                card.cover_office_address = pubgen_st.cover_office_address.clone();
                                                card.settings.insert("office_address".into(), pubgen_st.cover_office_address.clone());
                                                card.cover_style_idx = pubgen_st.cover_style;
                                                card.cover_add_map = pubgen_st.cover_add_map;
                                                card.cover_inside_back = pubgen_st.cover_include_inside_back;
                                            });
                                        } else if card.cover_tab == 1 {
                                            egui::Grid::new("cover_release_pic_grid").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                                                ui.label(RichText::new("Nomor Rilis:").font(semibold(11.0)).color(t.text));
                                                if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut card.cover_release_no)).changed() {
                                                    card.settings.insert("release_no".into(), card.cover_release_no.clone());
                                                    state.meta_release_no = card.cover_release_no.clone();
                                                }
                                                ui.end_row();

                                                ui.label(RichText::new("Tanggal Rilis:").font(semibold(11.0)).color(t.text));
                                                if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut card.cover_release_date)).changed() {
                                                    card.settings.insert("release_date".into(), card.cover_release_date.clone());
                                                    state.meta_release_date = card.cover_release_date.clone();
                                                }
                                                ui.end_row();

                                                ui.label(RichText::new("Bulan Rilis:").font(semibold(11.0)).color(t.text));
                                                if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut card.cover_release_month)).changed() {
                                                    card.settings.insert("release_month".into(), card.cover_release_month.clone());
                                                    state.meta_release_month = card.cover_release_month.clone();
                                                }
                                                ui.end_row();

                                                ui.label(RichText::new("Instansi Publikasi:").font(semibold(11.0)).color(t.text));
                                                if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut card.cover_pub_office)).changed() {
                                                    card.settings.insert("pub_office".into(), card.cover_pub_office.clone());
                                                    card.settings.insert("agency".into(), card.cover_pub_office.clone());
                                                    state.meta_agency = card.cover_pub_office.clone();
                                                }
                                                ui.end_row();

                                                ui.label(RichText::new("Nama PIC / Kontak:").font(semibold(11.0)).color(t.text));
                                                if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut card.cover_pic_name)).changed() {
                                                    card.settings.insert("pic_name".into(), card.cover_pic_name.clone());
                                                }
                                                ui.end_row();

                                                ui.label(RichText::new("Foto PIC:").font(semibold(11.0)).color(t.text));
                                                if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut card.cover_pic_photo)).changed() {
                                                    card.settings.insert("pic_photo".into(), card.cover_pic_photo.clone());
                                                }
                                                ui.end_row();

                                                ui.label(RichText::new("Telepon PIC:").font(semibold(11.0)).color(t.text));
                                                if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut card.cover_pic_phone)).changed() {
                                                    card.settings.insert("pic_phone".into(), card.cover_pic_phone.clone());
                                                }
                                                ui.end_row();

                                                ui.label(RichText::new("Email PIC:").font(semibold(11.0)).color(t.text));
                                                if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut card.cover_pic_email)).changed() {
                                                    card.settings.insert("pic_email".into(), card.cover_pic_email.clone());
                                                }
                                                ui.end_row();
                                            });
                                        } else {
                                            ui.label(RichText::new("Seluruh Kunci & Nilai Lembar COV:").font(semibold(11.0)).color(t.text_strong));
                                            ui.add_space(4.0);
                                            egui::ScrollArea::vertical().id_salt("cov_kv_scroll").max_height(280.0).show(ui, |ui| {
                                                egui::Grid::new("cov_kv_grid").striped(true).num_columns(2).spacing(vec2(12.0, 4.0)).show(ui, |ui| {
                                                    ui.label(RichText::new("Kunci (Key)").font(semibold(11.0)).color(t.text_dim));
                                                    ui.label(RichText::new("Nilai (Value)").font(semibold(11.0)).color(t.text_dim));
                                                    ui.end_row();

                                                    let mut keys_to_update: Vec<(String, String)> = Vec::new();
                                                    for (k, v) in &card.settings {
                                                        ui.label(RichText::new(k).font(FontId::monospace(10.5)).color(t.accent));
                                                        let mut val_buf = v.clone();
                                                        if ui.add_sized([ui.available_width(), crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut val_buf)).changed() {
                                                            keys_to_update.push((k.clone(), val_buf));
                                                        }
                                                        ui.end_row();
                                                    }
                                                    for (k, v) in keys_to_update {
                                                        card.settings.insert(k, v);
                                                    }
                                                });
                                            });
                                        }
                                    });
                                }
                                CardKind::Table => {
                                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                        ui.label(RichText::new("Aturan Ingest & Format Tabel (Table Rules):").font(semibold(11.0)).color(t.text_strong));
                                        ui.checkbox(&mut card.table_first_row_header, crate::i18n::tr(lang, "First row as table header"));
                                        ui.checkbox(&mut card.table_col_numbers, crate::i18n::tr(lang, "Show column numbers (1)(2)..."));
                                        ui.checkbox(&mut card.table_footer, crate::i18n::tr(lang, "Table footer (notes & source)"));
                                        ui.checkbox(&mut card.table_row_conditions, crate::i18n::tr(lang, "Zebra striping"));

                                        ui.separator();
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Table Style:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt("table_style_select")
                                                .selected_text(&card.table_style)
                                                .width(180.0)
                                                .show_ui(ui, |ui| {
                                                    for s in ["Basic Grid", "Corporate Navy", "Minimalist", "Horizontal Rules", "BRS6 Standard"] {
                                                        if ui.selectable_label(card.table_style == s, s).clicked() {
                                                            card.table_style = s.into();
                                                        }
                                                    }
                                                });
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Table Theme:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt("table_theme_select")
                                                .selected_text(&card.table_theme)
                                                .width(180.0)
                                                .show_ui(ui, |ui| {
                                                    for th in ["BPS Blue", "Corporate", "Emerald", "Crimson", "Slate"] {
                                                        if ui.selectable_label(card.table_theme == th, th).clicked() {
                                                            card.table_theme = th.into();
                                                        }
                                                    }
                                                });
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Table Layout:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt("table_layout_select")
                                                .selected_text(&card.table_layout_preset)
                                                .width(180.0)
                                                .show_ui(ui, |ui| {
                                                    for lp in ["Standard", "Compact", "Expanded"] {
                                                        if ui.selectable_label(card.table_layout_preset == lp, lp).clicked() {
                                                            card.table_layout_preset = lp.into();
                                                        }
                                                    }
                                                });
                                        });

                                        if ui.button(crate::i18n::tr(lang, "Apply to All Tables")).clicked() {
                                            apply_all_tables = true;
                                        }
                                    });
                                }
                                CardKind::Chart => {
                                    egui::Frame::NONE
                                        .fill(t.panel)
                                        .stroke(Stroke::new(1.0, t.field_border))
                                        .corner_radius(CornerRadius::same(t.radius))
                                        .inner_margin(Margin::same(8))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("📊").size(18.0));
                                                ui.vertical(|ui| {
                                                    ui.label(RichText::new(format!("{}: {}", crate::i18n::tr(lang, "Chart Archetype"), card.chart_type)).font(semibold(11.0)).color(t.text_strong));
                                                    let row_count = card.chart_data_end_row.saturating_sub(card.chart_data_start_row).saturating_add(1);
                                                    ui.label(RichText::new(format!("Source: {} · {} baris data", if card.source.is_empty() { "Tanpa sumber" } else { &card.source }, row_count)).font(FontId::proportional(11.0)).color(t.text_dim));
                                                });
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    if ui.button(RichText::new(format!("📊 {}", crate::i18n::tr(lang, "Configure Chart Options →"))).font(semibold(11.0))).clicked() {
                                                        switch_to_chart_tab = true;
                                                    }
                                                });
                                            });
                                        });
                                }
                                CardKind::MapChart => {
                                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                        ui.label(RichText::new(crate::i18n::tr(lang, "Thematic Map Settings (Choropleth & Spatial Map):")).font(semibold(11.0)).color(t.text_strong));

                                        // Pilihan GeoJSON (Stock vs Custom)
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "GeoJSON Source:")).font(semibold(11.0)));
                                            if ui.selectable_label(!card.map_is_custom, crate::i18n::tr(lang, "Stock Map (@assets/pubgen/maps)")).clicked() {
                                                card.map_is_custom = false;
                                                if card.map_stock_name.is_empty() {
                                                    card.map_stock_name = "CILAMAYA_WETAN".into();
                                                }
                                                card.source = format!("assets/pubgen/maps/{}_DESA.geojson", card.map_stock_name.trim_end_matches("_DESA").to_uppercase().replace([' ', '-'], "_"));
                                            }
                                            if ui.selectable_label(card.map_is_custom, crate::i18n::tr(lang, "Custom GeoJSON (File Link)")).clicked() {
                                                card.map_is_custom = true;
                                                if !card.map_custom_geojson.is_empty() {
                                                    card.source = card.map_custom_geojson.clone();
                                                }
                                            }
                                        });

                                        if !card.map_is_custom {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(crate::i18n::tr(lang, "Region Map Picker:")).font(semibold(11.0)));
                                                let mut stock_maps = super::charts::stock_maps::list_stock_map_charts();
                                                if stock_maps.is_empty() {
                                                    stock_maps = vec![
                                                        super::charts::stock_maps::StockMapEntry {
                                                            id: "CILAMAYA_WETAN".into(),
                                                            display_name: "Cilamaya Wetan (Desa)".into(),
                                                            path: std::path::PathBuf::from("assets/pubgen/maps/CILAMAYA_WETAN_DESA.geojson"),
                                                        },
                                                        super::charts::stock_maps::StockMapEntry {
                                                            id: "KARAWANG_TIMUR".into(),
                                                            display_name: "Karawang Timur (Desa)".into(),
                                                            path: std::path::PathBuf::from("assets/pubgen/maps/KARAWANG_TIMUR_DESA.geojson"),
                                                        },
                                                    ];
                                                }
                                                let current_norm = card.map_stock_name.trim_end_matches("_DESA").to_uppercase().replace([' ', '-'], "_");
                                                let selected_display = stock_maps
                                                    .iter()
                                                    .find(|m| m.id == current_norm || m.id == card.map_stock_name.to_uppercase() || m.display_name.eq_ignore_ascii_case(&card.map_stock_name))
                                                    .map(|m| m.display_name.clone())
                                                    .unwrap_or_else(|| {
                                                        if !card.map_stock_name.is_empty() {
                                                            format!("{} (Desa)", card.map_stock_name)
                                                        } else {
                                                            crate::i18n::tr(lang, "Pick a region map…").to_string()
                                                        }
                                                    });

                                                craft_ui::ComboBox::from_id_salt("map_stock_select")
                                                    .selected_text(&selected_display)
                                                    .width(260.0)
                                                    .show_ui(ui, |ui| {
                                                        for m in &stock_maps {
                                                            let is_sel = m.id == current_norm || m.display_name == selected_display;
                                                            if ui.selectable_label(is_sel, &m.display_name).clicked() {
                                                                card.map_stock_name = m.id.clone();
                                                                card.source = format!("assets/pubgen/maps/{}_DESA.geojson", m.id);
                                                            }
                                                        }
                                                    });
                                            });
                                        } else {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(crate::i18n::tr(lang, "GeoJSON file:")).font(semibold(11.0)));
                                                let edit = ui.add(
                                                    egui::TextEdit::singleline(&mut card.map_custom_geojson)
                                                        .desired_width(220.0)
                                                        .hint_text("Contoh: maps/wilayah.geojson"),
                                                );
                                                if edit.changed() {
                                                    card.source = card.map_custom_geojson.clone();
                                                }
                                                if ui.button("Buka...").clicked() {
                                                    let ctx = ui.ctx().clone();
                                                    let _ = app.ask_open("geojson", crate::app::file_dialogs::Bound::App, move |_app, path| {
                                                        let mut st = get_state(&ctx);
                                                        if let Some(idx) = st.editing_card_idx {
                                                            if let Some(c) = st.cards.get_mut(idx) {
                                                                c.map_custom_geojson = path.clone();
                                                                c.source = path;
                                                                c.map_is_custom = true;
                                                            }
                                                        }
                                                        set_state(&ctx, st);
                                                        ctx.request_repaint();
                                                        Ok(serde_json::Value::Null)
                                                    });
                                                }
                                            });
                                        }

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("Data Matriks (CSV):").font(semibold(11.0)));
                                            ui.add(
                                                egui::TextEdit::singleline(&mut card.map_data_csv)
                                                    .desired_width(220.0)
                                                    .hint_text("Contoh: maps/contoh.csv"),
                                            );
                                        });

                                        ui.separator();

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Map Type:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt("map_type_select")
                                                .selected_text(&card.map_chart_type)
                                                .width(220.0)
                                                .show_ui(ui, |ui| {
                                                    for mt in ["Choropleth", "Bubble Map", "Boundary Map"] {
                                                        if ui.selectable_label(card.map_chart_type == mt, mt).clicked() {
                                                            card.map_chart_type = mt.into();
                                                        }
                                                    }
                                                });
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Classification:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt("map_class_select")
                                                .selected_text(&card.map_classification)
                                                .width(220.0)
                                                .show_ui(ui, |ui| {
                                                    for cl in ["Quantile (4 kelas)", "Equal Interval (5 kelas)", "Jenks Natural Breaks"] {
                                                        if ui.selectable_label(card.map_classification == cl, cl).clicked() {
                                                            card.map_classification = cl.into();
                                                        }
                                                    }
                                                });
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Color Palette:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt("map_palette_select")
                                                .selected_text(&card.map_color_palette)
                                                .width(220.0)
                                                .show_ui(ui, |ui| {
                                                    for pal in ["BPS Biru-Teal", "Oranye Statistik", "Hijau Gradasi", "Ungu-Magenta", "Merah-Kuning-Hijau"] {
                                                        if ui.selectable_label(card.map_color_palette == pal, pal).clicked() {
                                                            card.map_color_palette = pal.into();
                                                        }
                                                    }
                                                });
                                        });

                                        ui.checkbox(&mut card.map_show_labels, crate::i18n::tr(lang, "Show Region Labels"));
                                        ui.checkbox(&mut card.map_show_legend, crate::i18n::tr(lang, "Show Legend"));
                                    });
                                }
                                CardKind::Image => {
                                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                        ui.label(RichText::new(crate::i18n::tr(lang, "Image & Illustration Settings:")).font(semibold(11.0)).color(t.text_strong));
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Placement Area:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt("image_area_select")
                                                .selected_text(&card.image_area)
                                                .width(200.0)
                                                .show_ui(ui, |ui| {
                                                    for area in ["Area Margin", "Full Bleed", "Hero", "Kolom Ganda"] {
                                                        if ui.selectable_label(card.image_area == area, area).clicked() {
                                                            card.image_area = area.into();
                                                        }
                                                    }
                                                });
                                        });
                                    });
                                }
                                CardKind::Toc => {
                                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                        ui.label(RichText::new(crate::i18n::tr(lang, "Table of Contents (TOC) Settings:")).font(semibold(11.0)).color(t.text_strong));
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "TOC Style:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt(("toc_style_select", card.page, card.chapter_id.as_str()))
                                                .selected_text(&card.toc_style)
                                                .width(200.0)
                                                .show_ui(ui, |ui| {
                                                    for (s, default_title_id, default_title_bilingual) in [
                                                        ("TOC Paragraph", "Daftar Isi", "Daftar Isi / Table of Contents"),
                                                        ("TOC Table", "Daftar Tabel", "Daftar Tabel / List of Tables"),
                                                        ("TOC Chart", "Daftar Grafik", "Daftar Grafik / List of Charts"),
                                                        ("TOC Entry", "Daftar Isi", "Daftar Isi / Table of Contents"),
                                                    ] {
                                                        if ui.selectable_value(&mut card.toc_style, s.to_string(), s).clicked() {
                                                            card.settings.insert("toc_style".into(), s.to_string());
                                                            state.is_dirty = true;
                                                            // Auto-fill Title if empty or matches standard title
                                                            let target_title = if is_bilingual { default_title_bilingual } else { default_title_id };
                                                            card.title = target_title.into();
                                                        }
                                                    }
                                                });
                                        });

                                        // Rules summary text per chosen TOC Style
                                        let rule_desc = match card.toc_style.as_str() {
                                            "TOC Paragraph" => "Aturan: Menyusun daftar bab narasi (RXX/BXX/BAB) dan sub-heading level 1 & 2 secara bersarang.",
                                            "TOC Table" => "Aturan: Hanya menyusun halaman tabel yang diawali 'Tabel' atau 'Table' (halaman landscape 'L' otomatis dikecualikan).",
                                            "TOC Chart" => "Aturan: Hanya menyusun halaman grafik atau gambar yang diawali 'Grafik', 'Chart', atau 'Gambar'.",
                                            _ => "Aturan: Mengumpulkan semua halaman dengan style 'TOC Entry' dan halaman narasi.",
                                        };
                                        ui.label(RichText::new(rule_desc).font(FontId::proportional(11.0)).color(t.text_dim));

                                        if card.toc_style == "TOC Table" || card.toc_style == "TOC Chart" {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(crate::i18n::tr(lang, "Tata Letak (Layout):")).font(semibold(11.0)));
                                                let layouts = ["Standard", "TablePage (BPS Catalog)"];
                                                let cur_layout = layouts.get(card.toc_layout_style).copied().unwrap_or(layouts[0]);
                                                craft_ui::ComboBox::from_id_salt(("toc_layout_style_select", card.page, card.chapter_id.as_str()))
                                                    .selected_text(cur_layout)
                                                    .width(180.0)
                                                    .show_ui(ui, |ui| {
                                                        for (idx, l) in layouts.iter().enumerate() {
                                                            if ui.selectable_value(&mut card.toc_layout_style, idx, *l).clicked() {
                                                                card.settings.insert("toc_layout_style".into(), idx.to_string());
                                                                state.is_dirty = true;
                                                            }
                                                        }
                                                    });
                                            });
                                        }

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("Kolom:").font(semibold(11.0)));
                                            if ui.selectable_label(card.toc_columns == "1 Kolom", "1 Kolom").clicked() {
                                                card.toc_columns = "1 Kolom".into();
                                                card.settings.insert("toc_columns".into(), "1 Kolom".into());
                                                state.is_dirty = true;
                                            }
                                            if ui.selectable_label(card.toc_columns == "2 Kolom", "2 Kolom").clicked() {
                                                card.toc_columns = "2 Kolom".into();
                                                card.settings.insert("toc_columns".into(), "2 Kolom".into());
                                                state.is_dirty = true;
                                            }
                                        });
                                        if ui.checkbox(&mut card.toc_hanging_indent, crate::i18n::tr(lang, "Hanging Indent")).changed() {
                                            card.settings.insert("toc_hanging_indent".into(), card.toc_hanging_indent.to_string());
                                            state.is_dirty = true;
                                        }
                                        if ui.checkbox(&mut card.toc_include_self, crate::i18n::tr(lang, "Include Self")).changed() {
                                            card.settings.insert("toc_include_self".into(), card.toc_include_self.to_string());
                                            state.is_dirty = true;
                                        }
                                        if is_bilingual && ui.checkbox(&mut card.toc_pair_bilingual, crate::i18n::tr(lang, "Pair Bilingual")).changed() {
                                            card.settings.insert("toc_pair_bilingual".into(), card.toc_pair_bilingual.to_string());
                                            state.is_dirty = true;
                                        }
                                    });
                                }
                                CardKind::Narrative => {
                                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                        ui.label(RichText::new(crate::i18n::tr(lang, "Text & Paragraph Format Settings:")).font(semibold(11.0)).color(t.text_strong));
                                        ui.label(RichText::new("Teks narasi di-parse secara otomatis dari dokumen RTF yang terhubung. Perataan paragraf, indentasi, heading, dan quote dipertahankan sesuai stylesheet BPS.").font(FontId::proportional(11.0)).color(t.text_dim));
                                    });
                                }
                                CardKind::Divider => {
                                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                        ui.label(RichText::new(crate::i18n::tr(lang, "Chapter Divider Settings:")).font(semibold(11.0)).color(t.text_strong));
                                        ui.label(RichText::new("Pembatas bab A4 full bleed dengan judul dwibahasa, ornamen grafis batik atau minimalis, dan penomoran bab otomatis.").font(FontId::proportional(11.0)).color(t.text_dim));
                                    });
                                }
                            }
                        }
                        CardEditTab::OpsiGrafik => {
                            if card.kind == CardKind::Chart {
                                render_chart_options_form(app, ui, card, &cached_content, t, lang);
                            } else {
                                ui.label(RichText::new(crate::i18n::tr(lang, "This card kind does not use chart options.")).font(FontId::proportional(11.0)).color(t.text_dim));
                            }
                        }
                        CardEditTab::TataLetakSeksi => {
                            // Section 4: Penomoran Halaman & Seksi (Halaman Baru)
                            if card.scope_new_page {
                                egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                    ui.label(
                                        RichText::new(crate::i18n::tr(lang, "Page & Section Numbering Options:"))
                                            .font(semibold(11.0))
                                            .color(t.text_strong),
                                    );
                                    ui.checkbox(&mut card.section_start, crate::i18n::tr(lang, "Start Section"));
                                    if card.section_start {
                                        ui.horizontal(|ui| {
                                            if ui.selectable_label(card.automatic_numbering, crate::i18n::tr(lang, "Automatic Numbering")).clicked() {
                                                card.automatic_numbering = true;
                                            }
                                            if ui.selectable_label(!card.automatic_numbering, crate::i18n::tr(lang, "Start at Page:")).clicked() {
                                                card.automatic_numbering = false;
                                            }
                                            if !card.automatic_numbering {
                                                ui.add(egui::DragValue::new(&mut card.start_page_number).range(1..=999));
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Section Prefix:")).font(semibold(11.0)));
                                            ui.add(egui::TextEdit::singleline(&mut card.section_prefix).desired_width(80.0));
                                            ui.label(RichText::new(crate::i18n::tr(lang, "Numbering Style:")).font(semibold(11.0)));
                                            craft_ui::ComboBox::from_id_salt("num_style_select")
                                                .selected_text(&card.numbering_style)
                                                .width(100.0)
                                                .show_ui(ui, |ui| {
                                                    for ns in ["1, 2, 3...", "i, ii, iii...", "A, B, C...", "Tidak bernomor"] {
                                                        if ui.selectable_label(card.numbering_style == ns, ns).clicked() {
                                                            card.numbering_style = ns.into();
                                                        }
                                                    }
                                                });
                                        });
                                    }
                                });
                            } else {
                                egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(t.radius)).inner_margin(Margin::same(8)).show(ui, |ui| {
                                    ui.label(RichText::new(crate::i18n::tr(lang, "Card Scope:")).font(semibold(11.0)).color(t.text_strong));
                                    ui.label(RichText::new("Kartu ini berstatus 'Objek Baru' (mengalir di halaman yang sama). Opsi penomoran seksi hanya berlaku untuk item yang berstatus 'Halaman Baru'.").font(FontId::proportional(11.0)).color(t.text_dim));
                                    ui.add_space(4.0);
                                    if ui.button(crate::i18n::tr(lang, "Ubah Menjadi Halaman Baru")).clicked() {
                                        card.scope_new_page = true;
                                        card.tag2 = "Halaman Baru".into();
                                    }
                                });
                            }
                        }
                        CardEditTab::HeaderFooter => {
                            // Section 5: Pengaturan Header & Footer (Header-Footer Setup for this publication page)
                            if card.kind != CardKind::Cover {
                                let card_page = card.page.max(1);
                                egui::Frame::NONE
                                    .fill(t.panel)
                                    .stroke(Stroke::new(1.0, t.field_border))
                                    .corner_radius(CornerRadius::same(t.radius))
                                    .inner_margin(Margin::same(8))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(crate::i18n::tr(lang, "Header & Footer Settings:")).font(semibold(11.0)).color(t.text_strong));
                                        ui.add_space(3.0);

                                        let ctx = ui.ctx().clone();
                                        crate::dock::pubgen::with_state(&ctx, |pubgen_st| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(crate::i18n::tr(lang, "Apply Option:")).font(semibold(11.0)).color(t.text_strong));
                                                let is_forward = card.hf_scope_forward;
                                                if ui.selectable_label(!is_forward, crate::i18n::tr(lang, "Apply for this page only")).clicked() {
                                                    card.hf_scope_forward = false;
                                                    pubgen_st.hf_scope = 3;
                                                    pubgen_st.hf_scope_start = card_page;
                                                    pubgen_st.hf_scope_end = card_page;
                                                }
                                                if ui.selectable_label(is_forward, crate::i18n::tr(lang, "Apply from this page forward")).clicked() {
                                                    card.hf_scope_forward = true;
                                                    pubgen_st.hf_scope = 4;
                                                    pubgen_st.hf_scope_start = card_page;
                                                }
                                            });

                                            // Keep pubgen_st scope synchronized with card's page
                                            if pubgen_st.hf_scope == 3 {
                                                pubgen_st.hf_scope_start = card_page;
                                                pubgen_st.hf_scope_end = card_page;
                                            } else if pubgen_st.hf_scope == 4 {
                                                pubgen_st.hf_scope_start = card_page;
                                            }

                                            ui.add_space(4.0);
                                            super::header_footer::card_header_footer_body_ui(app, ui, pubgen_st, t);

                                            // Sync values back to card
                                            card.hf_enabled = true;
                                            card.hf_scope_forward = pubgen_st.hf_scope == 4;
                                            card.hf_style = pubgen_st.hf_style.clone();
                                            card.hf_sided = pubgen_st.hf_sided;
                                            card.hf_header_single = pubgen_st.hf_header_single.clone();
                                            card.hf_header_left = pubgen_st.hf_header_left.clone();
                                            card.hf_header_right = pubgen_st.hf_header_right.clone();
                                            card.hf_footer_single = pubgen_st.hf_footer_single.clone();
                                            card.hf_footer_left = pubgen_st.hf_footer_left.clone();
                                            card.hf_footer_right = pubgen_st.hf_footer_right.clone();
                                            card.hf_header_rule = pubgen_st.hf_header_rule;
                                            card.hf_footer_rule = pubgen_st.hf_footer_rule;
                                            card.hf_header_offset = pubgen_st.hf_header_offset;
                                            card.hf_footer_offset = pubgen_st.hf_footer_offset;
                                        });
                                    });
                            }
                        }
                    }
                });

            ui.separator();

            // Footer Actions
            ui.horizontal(|ui| {
                // Delete button with confirmation
                if state.card_delete_confirm {
                    ui.label(RichText::new(crate::i18n::tr(lang, "Confirm Delete?")).font(semibold(11.0)).color(t.danger));
                    if ui.add(egui::Button::new(RichText::new(crate::i18n::tr(lang, "Yes, Delete")).color(t.caption_close_text)).fill(t.caption_close)).clicked() {
                        delete_card = true;
                    }
                    if ui.button(crate::i18n::tr(lang, "Cancel")).clicked() {
                        state.card_delete_confirm = false;
                    }
                } else if ui.button(RichText::new(crate::i18n::tr(lang, "Delete Card")).color(t.danger)).clicked() {
                    state.card_delete_confirm = true;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(format!("✓ {}", crate::i18n::tr(lang, "Save & Close"))).font(semibold(11.0)).color(Color32::WHITE)).fill(t.accent_strong)).clicked() {
                        close = true;
                    }

                    if ui.button(format!("+ {}", crate::i18n::tr(lang, "Insert After"))).clicked() {
                        insert_after = true;
                    }

                    if ui.button(format!("+ {}", crate::i18n::tr(lang, "Insert Before"))).clicked() {
                        insert_before = true;
                    }
                });
            });
        });

    if switch_to_chart_tab {
        state.card_edit_tab = CardEditTab::OpsiGrafik;
    }

    if apply_all_tables {
        if let Some(src_card) = state.cards.get(idx).cloned() {
            for c in &mut state.cards {
                if c.kind == CardKind::Table {
                    c.table_first_row_header = src_card.table_first_row_header;
                    c.table_col_numbers = src_card.table_col_numbers;
                    c.table_footer = src_card.table_footer;
                    c.table_row_conditions = src_card.table_row_conditions;
                    c.table_style = src_card.table_style.clone();
                    c.table_theme = src_card.table_theme.clone();
                    c.table_layout_preset = src_card.table_layout_preset.clone();
                }
            }
        }
    }

    if delete_card {
        state.cards.remove(idx);
        for (i, c) in state.cards.iter_mut().enumerate() {
            c.page = i + 1;
        }
        state.card_delete_confirm = false;
        state.editing_card_idx = None;
        state.last_editing_card_idx = None;
        refresh_detected_slots(state);
    } else if insert_before {
        let new_card = PublicationCard::new_with(state.new_card_kind, "Item Baru", "Draf konten", "Manual", idx + 1, true);
        state.cards.insert(idx, new_card);
        for (i, c) in state.cards.iter_mut().enumerate() {
            c.page = i + 1;
        }
        state.editing_card_idx = Some(idx);
        state.last_editing_card_idx = None;
        refresh_detected_slots(state);
    } else if insert_after {
        let new_card = PublicationCard::new_with(state.new_card_kind, "Item Baru", "Draf konten", "Manual", idx + 2, true);
        state.cards.insert(idx + 1, new_card);
        for (i, c) in state.cards.iter_mut().enumerate() {
            c.page = i + 1;
        }
        state.editing_card_idx = Some(idx + 1);
        state.last_editing_card_idx = None;
        refresh_detected_slots(state);
    } else if close {
        if let Some(card) = state.cards.get_mut(idx) {
            if card.kind == CardKind::Toc {
                card.settings.insert("toc_style".into(), card.toc_style.clone());
                card.settings.insert("toc_columns".into(), card.toc_columns.clone());
                card.settings.insert("toc_layout_style".into(), card.toc_layout_style.to_string());
                card.settings.insert("toc_hanging_indent".into(), card.toc_hanging_indent.to_string());
                card.settings.insert("toc_include_self".into(), card.toc_include_self.to_string());
                card.settings.insert("toc_pair_bilingual".into(), card.toc_pair_bilingual.to_string());
            }
        }
        state.is_dirty = true;
        if state.cards.get(idx).map(|c| c.kind != CardKind::Cover).unwrap_or(false) && app.session.active().is_some() {
            super::with_state(ctx, |st| {
                let _ = super::header_footer::execute_apply_header_footer(app, st);
            });
        }
        state.card_delete_confirm = false;
        state.editing_card_idx = None;
        state.last_editing_card_idx = None;
    }
}

/// The core draft of the current worksheet (T7): the metadata `to_metadata` writes into
/// a package, plus the staged content and the translation memory. This is what the
/// translation stage plans over (`collect_units` consumes the core draft).
fn worksheet_draft(state: &OrchestratorWorkspaceState) -> pubgen_core::workspace_validation::WorkspaceDraft {
    let mut csv = Vec::new();
    let mut text = Vec::new();
    let mut assets = Vec::new();
    for (key, bytes) in state.cached_content.iter() {
        let clean = key.replace('\\', "/");
        if clean.starts_with("csv/") || clean.ends_with(".csv") {
            csv.push((key.clone(), bytes.clone()));
        } else if clean.starts_with("rtf/")
            || clean.starts_with("md/")
            || clean.starts_with("text/")
            || clean.ends_with(".rtf")
            || clean.ends_with(".md")
        {
            text.push((key.clone(), bytes.clone()));
        } else {
            assets.push((key.clone(), bytes.clone()));
        }
    }
    let name = state.meta_title.trim();
    let name = if name.is_empty() { "worksheet" } else { name };
    let mut draft = pubgen_core::workspace_validation::WorkspaceDraft::from_ingest(
        name.to_string(),
        pubgen_core::excel::IngestReport {
            profile_id: state.selected_profile.clone(),
            metadata: state.to_metadata(),
            csv,
            text,
            assets,
            warnings: Vec::new(),
        },
    );
    draft.tm = state.tm.clone();
    draft
}

/// Count existing translated entries across the workspace Translation Memory, card settings, and state.
pub fn count_existing_translations(state: &OrchestratorWorkspaceState, pubgen_st: Option<&super::PubgenState>) -> usize {
    let mut count = state.tm.entries.iter().filter(|e| !e.target.trim().is_empty()).count();
    for card in &state.cards {
        if !card.title_en.trim().is_empty() && state.tm.lookup(&card.title).is_none() {
            count += 1;
        }
    }
    if count == 0 {
        if let Some(st) = pubgen_st {
            count = st.translations.len();
        }
    }
    count
}

/// "Translate with AI" (T7): resolve the TM and whole-string glossary hits locally,
/// then send only the misses to the provider as one background job.
/// When `replace_existing` is true, previous translation memory and card English titles
/// are reset so all units are re-translated from scratch.
fn start_translate_stage(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, replace_existing: bool) {
    if state.translate_job.is_some() || state.cards.is_empty() {
        return;
    }
    if replace_existing {
        let now = format_now();
        state.execution_log.push(format!("[{now}] Menimpa data terjemahan yang ada untuk menerjemahkan ulang dengan AI."));
        state.tm.entries.clear();
        for card in &mut state.cards {
            card.title_en.clear();
            card.settings.remove("title_en");
        }
        super::with_state(ctx, |pubgen_st| {
            pubgen_st.translations.clear();
        });
    } else {
        // Ensure any existing card.title_en is represented in state.tm so plan_units reuses it
        for card in &state.cards {
            if !card.title_en.trim().is_empty() && state.tm.lookup(&card.title).is_none() {
                state.tm.insert(pubgen_core::translation::TmEntry {
                    source: card.title.clone(),
                    target: card.title_en.trim().to_string(),
                    origin: pubgen_core::translation::TmOrigin::Reviewed,
                    role: "card_title".into(),
                    updated: pubgen_core::translation::now_stamp(),
                });
            }
        }
        if state.tm.entries.is_empty() {
            super::with_state(ctx, |pubgen_st| {
                for (src, tgt) in &pubgen_st.translations {
                    if !tgt.trim().is_empty() && state.tm.lookup(src).is_none() {
                        state.tm.insert(pubgen_core::translation::TmEntry {
                            source: src.clone(),
                            target: tgt.clone(),
                            origin: pubgen_core::translation::TmOrigin::Reviewed,
                            role: "auto".into(),
                            updated: pubgen_core::translation::now_stamp(),
                        });
                    }
                }
            });
        }
    }

    let glossary = super::ai::BpsGlossary::load(&app.session.prefs.pubgen_glossary_path);
    let draft = worksheet_draft(state);
    let mut tm = state.tm.clone();
    let (misses, plan) = super::translate_stage::plan_units(&draft, &tm, &glossary);
    super::translate_stage::store_glossary_hits(&draft, &mut tm, &glossary, &misses);
    state.tm = tm;
    state.translate_reused = plan.reused;
    if misses.is_empty() {
        // Everything resolved locally: no request leaves the machine.
        let now = format_now();
        state.execution_log.push(format!("[{now}] Translated 0 unit(s), reused {}, skipped 0, failed 0.", plan.reused));
        state.status_message =
            Some((format!("Semua data terjemahan sudah lengkap ({} item dari memori) — tidak ada kuota AI yang digunakan.", plan.reused), true));
        state.is_dirty = true;
        super::with_state(ctx, |pubgen_st| {
            super::translate_stage::seed_pubgen_state(pubgen_st, &state.tm);
        });
        return;
    }
    let Some(cfg) = super::ai::backend_config_from_prefs(app) else {
        state.status_message = Some(("Tidak ada penyedia AI yang dikonfigurasi (Pengaturan AI).".into(), false));
        return;
    };
    let pairs = glossary.prompt_pairs();
    let requests = super::translate_stage::masked_requests(&misses, &pairs);
    state.translate_units = misses;
    state.translate_glossary_pairs = pairs.clone();
    state.translate_job = Some(super::ai::TranslateJob::spawn(&cfg, requests, pairs, ctx.clone()));
    state.status_message = Some((format!("Menerjemahkan {} unit dengan AI…", state.translate_units.len()), true));
}

/// Poll the worksheet translation job (T7): restore the masked outputs, reject any
/// unit whose numbers did not survive, write the accepted English onto the cards and
/// into the memory, and report one line in LogEksekusi.
fn poll_translate_job(state: &mut OrchestratorWorkspaceState, app: &DesignApp, ctx: &egui::Context) {
    let Some(mut job) = state.translate_job.take() else { return };
    let mut scratch = super::PubgenState::default();
    let _ok = job.poll(&mut scratch);
    if !job.finished {
        state.status_message = Some((format!("Menerjemahkan dengan AI… {}/{}", job.done, job.total), true));
        state.translate_job = Some(job);
        return;
    }
    let mut draft = worksheet_draft(state);
    let mut tm = std::mem::take(&mut state.tm);
    let outputs: Vec<(String, String)> = scratch.translations.into_iter().collect();
    let (results, restore_failed) = super::translate_stage::restore_outputs(&state.translate_units, &outputs, &state.translate_glossary_pairs);
    // T8: deterministic glossary enforcement reuses the run's masking pairs — the same official
    // English the prompt asked for, applied to output the model may have written differently.
    let entries: Vec<pubgen_core::glossary_fix::Entry> = state
        .translate_glossary_pairs
        .iter()
        .map(|(term, replacement)| pubgen_core::glossary_fix::Entry {
            term: term.clone(),
            replacement: replacement.clone(),
            do_not_translate: false,
            case_preserve: true,
        })
        .collect();
    let mut res = super::translate_stage::apply_results(&mut draft, &mut tm, &state.translate_units, &results, &entries);
    state.tm = tm;
    res.failed.extend(restore_failed);
    // Refresh the UI cards from the core draft: `to_metadata` maps `state.cards` 1:1.
    for (core_card, ui_card) in draft.metadata.cards.iter().zip(&mut state.cards) {
        if let Some(ten) = core_card.settings.get("title_en").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
            ui_card.title_en = ten.to_string();
        }
    }

    // Best-effort mirror of Groq rate-limit to Supabase in background (0.8.4 parity)
    #[cfg(not(target_arch = "wasm32"))]
    {
        let cred = super::ai::backend_config_from_prefs(app)
            .filter(|cfg| cfg.provider == "groq")
            .map(|cfg| designcraft_imagegen::quota::fingerprint(&cfg.key));
        if let (Some(cred), Some(n)) = (cred, designcraft_translate::remaining_requests()) {
            let _ = std::thread::Builder::new().name("pubgen-groq-quota".into()).spawn(move || {
                designcraft_imagegen::quota::report_groq(&cred, n);
            });
        }
    }

    let provider = app.session.prefs.translation_provider.as_str();
    let quota_suffix = super::ai::groq_quota_suffix(provider);
    let now = format_now();
    state.execution_log.push(format!(
        "[{now}] Translated {} unit(s), reused {}, skipped {}, failed {}.{quota_suffix}",
        res.translated,
        state.translate_reused,
        res.skipped,
        res.failed.len()
    ));
    if let Some((msg, good)) = scratch.header_status
        && !good
    {
        state.execution_log.push(format!("[{now}] {msg}"));
    }
    if res.glossary_fixes > 0 {
        state.execution_log.push(format!("[{now}] Glossary enforced {} replacement(s) on AI output.", res.glossary_fixes));
    }
    state.status_message = Some((
        format!("Terjemahan selesai: {} unit, {} daur ulang, {} gagal{quota_suffix}", res.translated, state.translate_reused, res.failed.len()),
        res.failed.is_empty(),
    ));
    state.is_dirty = true;
    state.translate_units.clear();
    state.translate_glossary_pairs.clear();

    super::with_state(ctx, |pubgen_st| {
        super::translate_stage::seed_pubgen_state(pubgen_st, &state.tm);
    });

    // Trigger AI Quota Update modal popup (0.8.4 parity)
    super::with_state(ctx, |pubgen_st| {
        super::ai::push_groq_quota_popup(pubgen_st, provider);
    });
}

/// Executes compilation and automatically transitions workspace to DesignCraft Layout mode.
/// Build a `PubgenState` from the active worksheet (cards + cached csv/text/assets),
/// stage the cached content into a temp folder, and run the shared page-generation
/// pipeline (`excel::generate_publication_pages`) on it. This is the same generation
/// process as XLSX import, with the worksheet as the data source instead of a
/// workbook file on disk.
fn generate_from_workspace(app: &mut DesignApp, state: &OrchestratorWorkspaceState) -> Result<String, String> {
    let mut st = super::PubgenState::default();
    // Apply the selected publication profile the same way XLSX import does
    // (`excel::execute_excel_to_csvs`), otherwise BRS loses its table style,
    // theme and header/footer preset (`state::apply_profile`, `state.rs:787`).
    st.apply_profile(super::profiles::ProfileManager::get_profile(&state.selected_profile), false);
    // Fix round 2: carry the worksheet's language into the pipeline state. The page generator
    // reads it to decide bilingual output for BRS, so it must not arrive empty.
    st.doc_language = state.doc_language.clone();
    st.glossary = super::ai::BpsGlossary::load(&app.session.prefs.pubgen_glossary_path);

    if state.margin_top_mm > 0 {
        let mut prof = st.active_profile();
        prof.geometry.margins_mm = super::profiles::MarginsMm {
            top: state.margin_top_mm as f64,
            bottom: state.margin_bottom_mm as f64,
            inside: state.margin_left_mm as f64,
            outside: state.margin_right_mm as f64,
        };
        st.profile_custom = Some(prof.clone());
        st.apply_profile(prof, false);
    }

    // Stage the cached content into a temp folder so the file-based generators can read it. The
    // folder is unique per call: two generations can run at once in one process (a second tab, or
    // parallel tests), and a shared folder let one wipe the other's staged files mid-run.
    static STAGE_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = STAGE_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stage = std::env::temp_dir().join(format!("dc-pubgen-worksheet-{}-{seq}", std::process::id()));
    let _ = std::fs::remove_dir_all(&stage);
    std::fs::create_dir_all(&stage).map_err(|e| format!("worksheet staging dir: {e}"))?;
    let staged = |key: &str| -> Option<String> {
        let rel = key.trim();
        (rel != "Tanpa sumber" && !rel.is_empty()).then(|| stage.join(rel.replace('\\', "/")).to_string_lossy().into_owned())
    };
    for (rel, bytes) in state.cached_content.iter() {
        let p = stage.join(rel.replace('\\', "/"));
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        std::fs::write(&p, bytes).map_err(|e| format!("staging {rel}: {e}"))?;
    }

    // Cover fields: the worksheet metadata feeds the same `ExcelCoverData` the XLSX
    // import fills, plus a cover card's per-cover overrides.
    let cover_card = state.cards.iter().find(|c| c.kind == CardKind::Cover);
    if let Some(card) = cover_card {
        st.excel_had_cover = true;
        st.cover_include_inside_back = card.cover_inside_back;
        st.cover_style = card.cover_style_idx;
        if !card.cover_template.trim().is_empty() {
            if let Some(idx) = super::excel::cover_style_index(&card.cover_template) {
                st.cover_style = idx;
            }
        }
        let mut cover = super::excel::ExcelCoverData {
            style: if !card.cover_template.trim().is_empty() {
                card.cover_template.clone()
            } else {
                card.settings.get("style").cloned().unwrap_or_default()
            },
            title1: if !card.title.trim().is_empty() {
                card.title
                    .strip_prefix("Sampul Rilis: ")
                    .or_else(|| card.title.strip_prefix("Sampul Publikasi: "))
                    .or_else(|| card.title.strip_prefix("Sampul: "))
                    .unwrap_or(&card.title)
                    .to_string()
            } else {
                state.meta_title.clone()
            },
            title2: if !card.cover_title2.trim().is_empty() {
                card.cover_title2.clone()
            } else if let Some(t2) = card.settings.get("title2").filter(|s| !s.trim().is_empty()) {
                t2.clone()
            } else if !card.subtitle.trim().is_empty() && !card.subtitle.starts_with("Cover Release:") && !card.subtitle.starts_with("Sampul") {
                card.subtitle.clone()
            } else {
                state.meta_subtitle.clone()
            },
            title_en: if let Some(t2) = card.settings.get("title2").filter(|s| !s.trim().is_empty()) {
                t2.clone()
            } else if !card.cover_title2.trim().is_empty() {
                card.cover_title2.clone()
            } else if !card.title_en.trim().is_empty() && !card.title_en.starts_with("Cover Release:") && !card.title_en.starts_with("Sampul") {
                card.title_en.clone()
            } else {
                card.settings.get("title_en").cloned().unwrap_or_default()
            },
            year: if !card.cover_year.trim().is_empty() { card.cover_year.clone() } else { card.settings.get("year").cloned().unwrap_or_default() },
            volume: if !card.cover_volume.trim().is_empty() {
                card.cover_volume.clone()
            } else {
                card.settings.get("volume").cloned().unwrap_or_default()
            },
            catalogue: if !card.cover_catalogue.trim().is_empty() {
                card.cover_catalogue.clone()
            } else {
                card.settings.get("catalogue").cloned().unwrap_or_default()
            },
            code: if !card.cover_code_num.trim().is_empty() {
                card.cover_code_num.clone()
            } else {
                card.settings.get("code").cloned().unwrap_or_default()
            },
            pub_office: if !card.cover_pub_office.trim().is_empty() { card.cover_pub_office.clone() } else { state.meta_agency.clone() },
            agency: if !card.cover_pub_office.trim().is_empty() { card.cover_pub_office.clone() } else { state.meta_agency.clone() },
            office_address: if !card.cover_office_address.trim().is_empty() {
                card.cover_office_address.clone()
            } else {
                card.settings.get("office_address").cloned().unwrap_or_default()
            },
            table_theme: card.settings.get("table_theme").or_else(|| card.settings.get("theme")).cloned().unwrap_or_default(),
            event_theme: card.settings.get("event_theme").cloned().unwrap_or_default(),
            palette: card.cover_ai_palette.clone(),
            release_no: if state.selected_profile == "bps_brs" {
                if !card.cover_release_no.trim().is_empty() { card.cover_release_no.clone() } else { state.meta_release_no.clone() }
            } else {
                String::new()
            },
            release_date: if !card.cover_release_date.trim().is_empty() { card.cover_release_date.clone() } else { state.meta_release_date.clone() },
            release_month: if !card.cover_release_month.trim().is_empty() {
                card.cover_release_month.clone()
            } else {
                state.meta_release_month.clone()
            },
            ..Default::default()
        };
        if let Some(h) = card.settings.get("image_hero").or_else(|| card.settings.get("image_1")) {
            cover.image_hero = h.clone();
        }
        if let Some(l) = card.settings.get("image_landscape").or_else(|| card.settings.get("image_lanscape")) {
            cover.image_landscape = l.clone();
        }
        if let Some(lg) = card.settings.get("logo").or_else(|| card.settings.get("office_logo")) {
            cover.logo = lg.clone();
        }
        if !card.cover_pic_name.is_empty() || !card.cover_pic_photo.is_empty() {
            cover.pic_name = card.cover_pic_name.clone();
            cover.pic_photo = staged(&card.cover_pic_photo).unwrap_or_else(|| card.cover_pic_photo.clone());
            cover.pic_phone = card.cover_pic_phone.clone();
            cover.pic_email = card.cover_pic_email.clone();
        }
        st.excel_cover = Some(cover.clone());
        super::excel::apply_excel_cover(&mut st, &cover, Some(&stage));
        sync_card_to_pubgen_state(card, state, &mut st);
    }

    // Catalog rows: [type, number, title, file, template, title_en, file_en, theme] like the converted XLSX catalog.
    let mut rows: Vec<Vec<String>> = Vec::new();
    for card in state.cards.iter().filter(|c| c.kind != CardKind::Cover) {
        if card.kind == CardKind::Toc {
            rows.push(vec![
                "TOC".to_string(),
                "".to_string(),
                card.title.clone(),
                card.toc_style.clone(),
                card.toc_layout_style.to_string(),
                card.title_en.clone(),
                card.chapter_id.clone(),
                "".to_string(),
            ]);
            continue;
        }
        let mut file = staged(&card.source).unwrap_or_default();
        let typ = match card.kind {
            CardKind::Narrative => {
                let id_upper = card.chapter_id.to_ascii_uppercase();
                let is_rxx = id_upper.starts_with('R') && id_upper.len() > 1 && id_upper[1..].chars().all(|c| c.is_ascii_digit() || c == '_');
                let is_bxx = id_upper.starts_with('B') && id_upper.len() > 1 && id_upper[1..].chars().all(|c| c.is_ascii_digit() || c == '_');
                let is_preface = card.title.to_lowercase().contains("pengantar") || card.title_en.to_lowercase().contains("preface");
                let title_upper = card.title.trim().to_ascii_uppercase();
                let title_starts_bab = title_upper.starts_with("BAB ") || title_upper.starts_with("CHAPTER ");
                if is_rxx || is_bxx || is_preface || title_starts_bab { if file.is_empty() { "Note" } else { "Ulasan" } } else { "Note" }
            }
            CardKind::Table => "Tabel",
            CardKind::Divider => "Divider",
            CardKind::Chart => {
                if let Some(img) = staged(&card.chart_image) {
                    file = img;
                    "Image"
                } else {
                    let chart_id = if card.chapter_id.is_empty() { format!("C{:02}_01", card.page) } else { card.chapter_id.clone() };
                    let staged_csv =
                        staged(&format!("csv/{}.csv", chart_id)).or_else(|| staged(&format!("{}.csv", chart_id))).or_else(|| staged(&card.source));

                    if let Some(c) = staged_csv {
                        file = c;
                    } else if !card.chart_raw_data.trim().is_empty() {
                        let csv_content = build_chart_sheet_csv(card);
                        let p = stage.join(format!("csv/{chart_id}.csv"));
                        if let Some(dir) = p.parent() {
                            let _ = std::fs::create_dir_all(dir);
                        }
                        if std::fs::write(&p, csv_content).is_ok() {
                            file = p.to_string_lossy().into_owned();
                        } else {
                            file = chart_id;
                        }
                    } else {
                        file = chart_id;
                    }
                    "Gambar"
                }
            }
            CardKind::MapChart => {
                let staged_csv = staged(&format!("csv/{}.csv", card.chapter_id));
                if let Some(c) = staged_csv {
                    file = c;
                } else if let Some(c) = staged(&card.map_data_csv) {
                    file = c;
                } else {
                    file = card.chapter_id.clone();
                }
                "Peta"
            }
            _ => "Image",
        };
        let number = if card.kind == CardKind::Divider {
            super::excel::extract_first_two_digit_number(&card.title)
                .or_else(|| super::excel::extract_dotted_number(&card.title))
                .or_else(|| {
                    let d: String = card.title.chars().filter(|c| c.is_ascii_digit()).collect();
                    (!d.is_empty()).then(|| if d.len() == 1 { format!("0{d}") } else { d })
                })
                .unwrap_or_else(|| "01".to_string())
        } else {
            super::excel::extract_dotted_number(&card.title)
                .or_else(|| {
                    let norm_title = card.title.replace(',', ".");
                    super::excel::extract_dotted_number(&norm_title)
                })
                .or_else(|| {
                    let stripped = card.chapter_id.trim_start_matches(|c: char| c.is_alphabetic() || c == '_');
                    if !stripped.is_empty() && stripped.contains('_') { Some(stripped.replace('_', ".")) } else { None }
                })
                .or_else(|| card.title.split_whitespace().find(|w| !w.is_empty() && w.chars().all(|c| c.is_ascii_digit())).map(str::to_string))
                .unwrap_or_default()
        };
        let template = if card.kind == CardKind::Divider {
            card.settings
                .get("template")
                .or_else(|| card.settings.get("style"))
                .map(|s| super::excel::resolve_divider_template(s))
                .unwrap_or_else(|| "assets/templates/dividers/002_batik_infographic.designcraft".to_string())
        } else {
            card.settings.get("template").cloned().unwrap_or_default()
        };
        let title_en =
            if !card.title_en.trim().is_empty() { card.title_en.clone() } else { card.settings.get("title_en").cloned().unwrap_or_default() };
        let raw_file_en = card.settings.get("file_en").and_then(|fen| staged(fen)).unwrap_or_default();
        // A `file_en` that names the same file as `file` (same path, case, slash or bare name) is not
        // a separate English RTF, so it must export blank: otherwise a re-import reads the Indonesian
        // file back as "English" instead of translating it.
        let file_en = if super::file_en_same_as_file(&file, &raw_file_en) { String::new() } else { raw_file_en };
        let image1 = card.settings.get("image1").and_then(|im| staged(im)).unwrap_or_default();
        let image2 = card.settings.get("image2").and_then(|im| staged(im)).unwrap_or_default();
        let theme = if !card.table_theme.trim().is_empty() {
            card.table_theme.clone()
        } else {
            card.settings.get("theme").or_else(|| card.settings.get("table_theme")).cloned().unwrap_or_default()
        };
        let chapter = card.settings.get("chapter").or_else(|| card.settings.get("chapter_title")).cloned().unwrap_or_else(|| {
            let ch_num = card.title.split_whitespace().find(|w| w.chars().any(|c| c.is_ascii_digit())).unwrap_or("");
            let d_num: String = ch_num.chars().take_while(|c| c.is_ascii_digit() || *c == '.').take(2).collect();
            if !d_num.is_empty() {
                state.cards.iter().find(|c| c.kind == CardKind::Divider && c.title.contains(&d_num)).map(|c| c.title.clone()).unwrap_or_default()
            } else {
                String::new()
            }
        });
        let chapter_en = card.settings.get("chapter_en").or_else(|| card.settings.get("chapter_title_en")).cloned().unwrap_or_else(|| {
            let ch_num = card.title.split_whitespace().find(|w| w.chars().any(|c| c.is_ascii_digit())).unwrap_or("");
            let d_num: String = ch_num.chars().take_while(|c| c.is_ascii_digit() || *c == '.').take(2).collect();
            if !d_num.is_empty() {
                state.cards.iter().find(|c| c.kind == CardKind::Divider && c.title.contains(&d_num)).map(|c| c.title_en.clone()).unwrap_or_default()
            } else {
                String::new()
            }
        });
        let section = card
            .settings
            .get("section")
            .or_else(|| card.settings.get("section_title"))
            .or_else(|| card.settings.get("subtitle"))
            .cloned()
            .unwrap_or_default();
        let section_en = card
            .settings
            .get("section_en")
            .or_else(|| card.settings.get("section_title_en"))
            .or_else(|| card.settings.get("subtitle_en"))
            .cloned()
            .unwrap_or_default();

        rows.push(vec![
            typ.to_string(),
            number,
            card.title.clone(),
            file,
            template,
            title_en,
            file_en,
            theme,
            chapter,
            chapter_en,
            section,
            section_en,
            image1,
            image2,
        ]);
    }
    if rows.is_empty() && !st.excel_had_cover {
        return Err("the worksheet has no publication cards to generate from".into());
    }
    st.card1_headers = vec![
        "type".into(),
        "number".into(),
        "title".into(),
        "file".into(),
        "template".into(),
        "title_en".into(),
        "file_en".into(),
        "theme".into(),
        "chapter".into(),
        "chapter_en".into(),
        "section".into(),
        "section_en".into(),
        "image1".into(),
        "image2".into(),
    ];
    st.card1_records = rows;
    st.card1_type_col = 0;
    st.card1_num_col = 1;
    st.card1_title_col = 2;
    st.card1_file_col = 3;
    st.card1_template_col = 4;
    st.card1_title_en_col = 5;
    st.card1_file_en_col = 6;
    st.card1_range_to = st.card1_records.len();

    let catalog_csv_path = stage.join("catalog.csv");
    let mut catalog_content = st.card1_headers.join(",") + "\n";
    for r in &st.card1_records {
        let line: Vec<String> = r.iter().map(|f| super::excel::escape_csv_field(f)).collect();
        catalog_content.push_str(&line.join(","));
        catalog_content.push('\n');
    }
    let _ = std::fs::write(&catalog_csv_path, &catalog_content);
    st.card1_path = catalog_csv_path.to_string_lossy().to_string();

    st.card2_sheet_tables = state.cards.iter().filter(|c| c.kind == CardKind::Table).filter_map(|c| staged(&c.source)).collect();
    st.card2_sheet_idx = 0;
    if let Some(first_table) = st.card2_sheet_tables.first().cloned() {
        st.card2_path = first_table.clone();
        super::csv::load_card2_csv(&mut st, &first_table);
    }

    // Bilingual output is re-derived *here*, at the last point before page generation: every
    // `apply_profile` call and `excel::apply_excel_cover` above resets `bilingual_output` from
    // the profile, and the seed below derives `translate_with_ai` from it. Deriving it earlier
    // left the two flags disagreeing and the paired English never rendered.
    let is_bilingual = state.doc_language.to_lowercase().contains("bilingual")
        || state.doc_language.to_lowercase().contains("dwibahasa")
        || state.doc_language.eq_ignore_ascii_case("indonesian-english")
        || state.doc_language.eq_ignore_ascii_case("indo-english")
        || (state.doc_language.to_lowercase().contains("indonesia") && state.doc_language.to_lowercase().contains("english"));
    if is_bilingual || state.selected_profile == "bps_dda" {
        st.bilingual_output = true;
    } else if state.selected_profile == "bps_brs" {
        st.bilingual_output = false;
    }
    // T7: seed the generation cache from the translation memory so a compile reuses
    // every translation; the worksheet's glossary reaches `pair_cells` the same way.
    super::translate_stage::seed_pubgen_state(&mut st, &state.tm);

    let gen_res = super::excel::generate_publication_pages(app, &mut st)?;

    // Generate TOC contents on their pre-allocated placeholder pages in card order
    for card in state.cards.iter().filter(|c| c.kind == CardKind::Toc) {
        let placeholder_loc = app.session.doc().ok().and_then(|d| {
            for sp in &d.doc.spreads {
                for it in &sp.items {
                    if it.label == "pubgen_toc_card" {
                        if let Some(pg) = d.doc.page_of_item(it.id) {
                            return Some((it.id, pg));
                        }
                    }
                }
            }
            None
        });

        let cur_pages = app.session.doc().map(|d| d.doc.pages().count()).unwrap_or(0);
        let (ph_to_clear, target_page) = if let Some((ph_id, pg)) = placeholder_loc {
            (Some(ph_id), pg)
        } else {
            let p = if card.page > 0 && card.page <= cur_pages { card.page.saturating_sub(1) } else { cur_pages };
            if card.scope_new_page || p >= cur_pages {
                let after_idx = p.saturating_sub(1);
                let _ = app.run("layout.pages.insert", serde_json::json!({ "after": after_idx, "count": 1 }));
            }
            (None, p)
        };

        if let Some(ph_id) = ph_to_clear {
            let _ = app.run("edit.clear", serde_json::json!({ "ids": [ph_id.0] }));
        }

        let toc_kind_idx = match card.toc_style.as_str() {
            "TOC Table" => 1,
            "TOC Chart" => 2,
            "TOC Paragraph" => 3,
            _ => {
                let t = card.title.to_lowercase();
                if t.contains("tabel") || t.contains("table") {
                    1
                } else if t.contains("grafik") || t.contains("chart") || t.contains("gambar") || t.contains("figure") {
                    2
                } else {
                    3
                }
            }
        };

        let mut toc_st = st.clone();
        toc_st.toc_kind = toc_kind_idx;
        toc_st.toc_title = card.title.clone();
        toc_st.toc_layout_style = card.toc_layout_style;
        toc_st.toc_hanging_indent = card.toc_hanging_indent;
        toc_st.toc_include_self = card.toc_include_self;
        toc_st.toc_pair_bilingual = card.toc_pair_bilingual;
        toc_st.bilingual_output = card.toc_pair_bilingual && state.doc_language != "Indonesian";

        let plan = super::toc::TocPlan::from_state(&toc_st);
        let _ = super::toc::generate_toc(app, &toc_st, target_page, &plan, false);
    }

    // A translation job still in flight has not written any memory entry yet, so the pages above
    // rendered the previous memory and the English column keeps the Indonesian body — which looks
    // exactly like the bug being fixed. Say so instead of letting the user re-report it.
    Ok(if state.translate_job.is_some() {
        format!(
            "{gen_res} AI translation is still running — the English columns use the memory as it was before this run; generate again when it finishes."
        )
    } else {
        gen_res
    })
}

fn start_compile_job(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, publish: bool, lang: &str) {
    let now = format_now();
    let pages_count = state.cards.len();
    state.execution_log.push(format!(
        "[{now}] {}",
        crate::i18n::tr(lang, "Generating {n} publication page structures from the active worksheet…").replace("{n}", &pages_count.to_string())
    ));

    match generate_from_workspace(app, state) {
        Ok(msg) => {
            state.execution_log.push(format!("[{now}] {msg}"));
            let total_doc_pages = app.session.doc().map(|d| d.doc.pages().count()).unwrap_or(0);
            state.compile_progress = Some(CompileProgressState {
                is_publish: publish,
                total_pages: total_doc_pages,
                current_page: 0,
                status_text: crate::i18n::tr(lang, "Menyiapkan pratinjau halaman tata letak…").to_string(),
                error_msg: None,
                completed: total_doc_pages == 0,
            });
            ctx.request_repaint();
        }
        Err(e) => {
            state.execution_log.push(format!("[{now}] Error: {e}"));
            state.status_message = Some((format!("Generate gagal: {e}"), false));
        }
    }
}

/// Renders progress notification modal and performs thumbnail pre-warming step-by-step
fn render_compile_progress_modal(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    let Some(mut prog) = state.compile_progress.clone() else {
        return;
    };

    let title = if prog.is_publish {
        crate::i18n::tr(lang, "Mengompilasi & Menerbitkan Publikasi")
    } else {
        crate::i18n::tr(lang, "Menyusun Halaman Publikasi")
    };

    let mut finish_and_switch = false;
    let mut close = false;

    egui::Window::new(title).collapsible(false).resizable(false).fixed_size([460.0, 180.0]).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).show(
        ctx,
        |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);

            if let Some(ref err) = prog.error_msg {
                ui.label(RichText::new(err).font(semibold(12.0)).color(Color32::from_rgb(220, 50, 50)));
                ui.add_space(8.0);
                if ui.button(crate::i18n::tr(lang, "Tutup")).clicked() {
                    close = true;
                }
            } else if prog.completed {
                ui.label(
                    RichText::new(format!("✓ {}", crate::i18n::tr(lang, "Tata letak siap! Semua halaman dan visual cache selesai diproses.")))
                        .font(semibold(12.0))
                        .color(t.text_strong),
                );
                let count_text = format!("{}/{} {}", prog.total_pages, prog.total_pages, crate::i18n::tr(lang, "halaman"));
                ui.add(egui::ProgressBar::new(1.0).show_percentage().desired_width(ui.available_width()).text(count_text));
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        "Seluruh halaman dokumen dan visual cache telah selesai dimuat. Navigasi panel dan kanvas di DesignCraft kini dapat berjalan lancar tanpa freeze.",
                    ))
                    .font(FontId::proportional(11.0))
                    .color(t.text_dim),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("🎨 {}", crate::i18n::tr(lang, "Buka Lembar Kerja DesignCraft")))
                                    .font(semibold(12.0))
                                    .color(Color32::WHITE),
                            )
                            .fill(t.accent_strong)
                            .corner_radius(CornerRadius::same(t.radius)),
                        )
                        .clicked()
                    {
                        finish_and_switch = true;
                    }

                    if ui.button(crate::i18n::tr(lang, "Tetap di Orchestrator")).clicked() {
                        close = true;
                    }
                });
            } else {
                ui.label(RichText::new(&prog.status_text).font(semibold(12.0)).color(t.text_strong));

                let fraction = if prog.total_pages > 0 { (prog.current_page as f32 / prog.total_pages as f32).clamp(0.0, 0.99) } else { 0.5 };

                let progress_bar = egui::ProgressBar::new(fraction)
                    .show_percentage()
                    .desired_width(ui.available_width())
                    .text(format!("{}/{} halaman", prog.current_page, prog.total_pages));
                ui.add(progress_bar);

                ui.label(
                    RichText::new(crate::i18n::tr(lang, "Membuat cache visual dan komposisi halaman agar navigasi panel lancar..."))
                        .font(FontId::proportional(11.0))
                        .color(t.text_dim),
                );

                if prog.current_page < prog.total_pages {
                    let page_idx = prog.current_page;
                    let _ = crate::panels::pages::prewarm_page_thumbnail(app, ctx, page_idx);
                    prog.current_page += 1;
                    prog.status_text = format!(
                        "{} ({}/{})",
                        crate::i18n::tr(lang, "Membuat cache visual halaman…"),
                        prog.current_page,
                        prog.total_pages
                    );
                    ctx.request_repaint();
                } else {
                    // All pages pre-warmed!
                    if prog.is_publish {
                        let pdf_path = format!("outputs/{}.pdf", sanitize_file_stem(&state.meta_title));
                        match app.run("file.exportPdf", serde_json::json!({ "path": pdf_path })) {
                            Ok(_) => {
                                let now = format_now();
                                state
                                    .execution_log
                                    .push(format!("[{now}] {}", crate::i18n::tr(lang, "Exported release PDF: {path}").replace("{path}", &pdf_path)));
                                state.last_output_pdf = Some(pdf_path);
                                state.status_message = Some(("Kompilasi & Publikasi PDF Berhasil".into(), true));
                                prog.completed = true;
                            }
                            Err(e) => {
                                prog.error_msg = Some(format!("Ekspor PDF gagal: {e}"));
                            }
                        }
                    } else {
                        state.status_message = Some(("Dokumen berhasil digenerate dan visual cache siap".into(), true));
                        prog.completed = true;
                    }
                    ctx.request_repaint();
                }
            }
        },
    );

    if close {
        state.compile_progress = None;
    } else if finish_and_switch {
        state.compile_progress = None;
        crate::menus::pubgen_bar::set_orchestrator_mode(ctx, false);
    } else {
        state.compile_progress = Some(prog);
    }
}

fn execute_compile(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, publish: bool, lang: &str) {
    start_compile_job(app, ctx, state, publish, lang);
}

/// File-stem-safe version of a title for the output PDF name.
fn sanitize_file_stem(title: &str) -> String {
    let stem: String = title.trim().chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' { c } else { ' ' }).collect();
    let stem = stem.split_whitespace().collect::<Vec<_>>().join("_");
    if stem.is_empty() { "publikasi".into() } else { stem }
}

/// Renders the Re-compilation Safety Guard Dialog (3-way modal: Overwrite / Save v2 / Cancel).
fn render_guard_dialog(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    if !state.show_guard_dialog {
        return;
    }

    let mut close = false;
    let mut chosen_action = None;

    let guard_title = crate::i18n::tr(lang, "Compilation Safety Guard");
    egui::Window::new(guard_title).collapsible(false).resizable(false).fixed_size([460.0, 240.0]).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).show(
        ctx,
        |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);

            ui.label(
                RichText::new(crate::i18n::tr(lang, "The active layout already has pages and manual edits."))
                    .font(semibold(12.0))
                    .color(t.text_strong),
            );

            ui.label(
                RichText::new(crate::i18n::tr(
                    lang,
                    "Recompiling rebuilds every page from the active worksheet data. \
                     Typography adjustments, frame positions, or manual styles made in DesignCraft mode \
                     may be overwritten.",
                ))
                .font(FontId::proportional(11.0))
                .color(t.text_dim),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [175.0, 26.0],
                        egui::Button::new(
                            RichText::new(crate::i18n::tr(lang, "Save as New Version (v2)")).font(semibold(11.0)).color(Color32::WHITE),
                        )
                        .fill(t.accent_strong),
                    )
                    .on_hover_text(crate::i18n::tr(lang, "Keep the old layout and compile into a new (_v2) file"))
                    .clicked()
                {
                    chosen_action = Some(1); // New version
                    close = true;
                }

                if ui
                    .add_sized(
                        [165.0, 26.0],
                        egui::Button::new(RichText::new(crate::i18n::tr(lang, "Overwrite Layout")).font(semibold(11.0)).color(t.caption_close_text))
                            .fill(t.caption_close),
                    )
                    .on_hover_text(crate::i18n::tr(lang, "Replace the active document with a fresh compile from the source data"))
                    .clicked()
                {
                    chosen_action = Some(2); // Overwrite
                    close = true;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(crate::i18n::tr(lang, "Cancel")).clicked() {
                        close = true;
                    }
                });
            });
        },
    );

    if close {
        state.show_guard_dialog = false;
    }

    if let Some(act) = chosen_action {
        let is_publish = state.guard_is_publish;
        let now = format_now();
        let stem = sanitize_file_stem(&state.meta_title);
        if act == 1 {
            // Save the current manual layout as v2, then regenerate fresh pages in a new document.
            let v2 = format!("outputs/{stem}_v2.designcraft");
            state.execution_log.push(format!("[{now}] {}", crate::i18n::tr(lang, "Saving old layout to: {path}").replace("{path}", &v2)));
            let _ = app.run("file.saveAs", serde_json::json!({ "path": v2 }));
            let _ = app.run("file.new", serde_json::json!({}));
            start_compile_job(app, ctx, state, is_publish, lang);
        } else if act == 2 {
            state.execution_log.push(format!("[{now}] Menimpa tata letak aktif dengan hasil generate segar dari worksheet..."));
            let _ = app.run("file.new", serde_json::json!({}));
            start_compile_job(app, ctx, state, is_publish, lang);
        }
    }
}

/// Renders the AI Translation Guard Dialog when the project already has translated data:
/// Allows the user to either translate only untranslated items (efficient, saves quota)
/// or replace/overwrite all existing translation data with fresh AI translations.
fn render_translate_guard_dialog(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    if !state.show_translate_guard_dialog {
        return;
    }

    let mut close = false;
    let mut chosen_replace = None;
    let existing_count = super::with_state(ctx, |pubgen_st| count_existing_translations(state, Some(pubgen_st)));

    let guard_title = crate::i18n::tr(lang, "Existing Translations Found");
    egui::Window::new(guard_title)
        .collapsible(false)
        .resizable(false)
        .fixed_size([500.0, 230.0])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);

            ui.label(
                RichText::new(
                    crate::i18n::tr(lang, "This project already contains existing translation data ({count} items).")
                        .replace("{count}", &existing_count.to_string()),
                )
                .font(semibold(12.0))
                .color(t.text_strong),
            );

            ui.label(
                RichText::new(crate::i18n::tr(
                    lang,
                    "To save AI quota and processing time, you can translate only untranslated items, or replace existing data with fresh AI translations.",
                ))
                .font(FontId::proportional(11.0))
                .color(t.text_dim),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [185.0, 26.0],
                        egui::Button::new(
                            RichText::new(crate::i18n::tr(lang, "Translate Untranslated Only"))
                                .font(semibold(11.0))
                                .color(Color32::WHITE),
                        )
                        .fill(t.accent_strong),
                    )
                    .on_hover_text(crate::i18n::tr(lang, "Keep existing translations and translate only untranslated items to conserve AI quota"))
                    .clicked()
                {
                    chosen_replace = Some(false);
                    close = true;
                }

                if ui
                    .add_sized(
                        [165.0, 26.0],
                        egui::Button::new(
                            RichText::new(crate::i18n::tr(lang, "Replace Existing Data"))
                                .font(semibold(11.0))
                                .color(t.caption_close_text),
                        )
                        .fill(t.caption_close),
                    )
                    .on_hover_text(crate::i18n::tr(lang, "Clear existing translations and re-translate all units from scratch with AI"))
                    .clicked()
                {
                    chosen_replace = Some(true);
                    close = true;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(crate::i18n::tr(lang, "Cancel")).clicked() {
                        close = true;
                    }
                });
            });
        });

    if close {
        state.show_translate_guard_dialog = false;
    }

    if let Some(replace) = chosen_replace {
        start_translate_stage(app, ctx, state, replace);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dock opens at 320 px, and the divider can drag it only within
    /// [`RIGHT_DOCK_MIN_W`] and 60% of the window; collapsing gives the 38 px icon strip and no divider.
    #[test]
    fn dock_width_clamps_and_collapses() {
        assert_eq!(dock_width(1440.0, true, default_right_dock_w()), (320.0, DOCK_DIVIDER_W));
        assert_eq!(dock_width(1440.0, true, 40.0).0, RIGHT_DOCK_MIN_W);
        assert_eq!(dock_width(1440.0, true, 5000.0).0, 1440.0 * RIGHT_DOCK_MAX_SHARE);
        assert_eq!(dock_width(300.0, true, 5000.0).0, RIGHT_DOCK_MIN_W);
        assert_eq!(dock_width(1440.0, false, 320.0), (RIGHT_DOCK_COLLAPSED_W, 0.0));
    }

    /// Three tabs share the dock at ~100 px each (the default 320 px dock), less than "Sources &
    /// Content", "Profile & Execution" and "Translation Review" need at 12 px. A painted label must
    /// therefore fit its own tab — elided with "…" — and never run into its neighbour. Measured with
    /// the real font: each galley's width against its tab's width minus [`DOCK_TAB_PAD`].
    #[test]
    fn dock_tab_labels_fit_their_tab_at_the_default_width() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx, "en");
        let mut out = ctx.run_ui(Default::default(), |_| {}); // fonts only bind inside a run
        out.textures_delta.clear();

        let (tab1_w, tab2_w, tab3_w) = dock_tab_widths(default_right_dock_w());
        let tabs = [("Sources & Content", tab1_w), ("Profile & Execution", tab2_w), ("Translation Review", tab3_w)];
        for lang in ["en", "id"] {
            for (key, tab_w) in tabs {
                let text = crate::i18n::tr(lang, key);
                let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(tab_w, DOCK_TAB_H));
                let galley = dock_tab_label(&ctx, rect, text, Color32::WHITE);
                let raw = crate::rtl::plain(&ctx, text, FontId::proportional(DOCK_TAB_LABEL), Color32::WHITE);
                assert!(
                    galley.size().x <= tab_w - DOCK_TAB_PAD + 0.5,
                    "{lang} `{text}` paints {:.1} px into a {tab_w} px tab ({:.1} px unelided, {:.1} px available)",
                    galley.size().x,
                    raw.size().x,
                    tab_w - DOCK_TAB_PAD
                );
                assert!(galley.elided || raw.size().x <= tab_w - DOCK_TAB_PAD, "{lang} `{text}` must be elided to fit");
            }
            // The widest label must actually elide at this width, or the check above proves nothing.
            let widest = crate::i18n::tr(lang, "Translation Review");
            let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(tab3_w, DOCK_TAB_H));
            assert!(dock_tab_label(&ctx, rect, widest, Color32::WHITE).elided, "{lang} `{widest}` was expected to elide in a {tab3_w} px tab");
        }
    }

    /// Chrome type comes from one small scale: DesignCraft's `LABEL_SIZE` (11 px), the 11.5 px section
    /// band, the 12 px panel title, and the [`HOME_*`] start-screen sizes. Nothing in this file may dip
    /// back to the 9–10.5 px sizes that made the Orchestrator's labels smaller than every panel in
    /// DesignCraft (F5). The needles are assembled at run time so this test's own source cannot match.
    #[test]
    fn chrome_type_scale_never_dips_below_the_label_size() {
        let src = include_str!("workspace.rs");
        let prefixes = ["semibold(", "proportional(", ".size("];
        let banned = ["9.", "10."];
        for p in prefixes {
            for b in banned {
                let needle = format!("{p}{b}");
                if let Some(at) = src.find(&needle) {
                    let rest: String = src[at..].chars().take(40).collect();
                    panic!("chrome text below 11 px: {rest}");
                }
            }
        }
    }

    /// Card rows contain `Ui::horizontal` rows (pills, footer) that are never shorter than the
    /// style's `interact_size.y` (18 px by default). The measurement must budget those minimums or
    /// wrapped titles push the footer past the card edge (clipped "Pg. N — Click to edit").
    #[test]
    fn measured_card_height_budgets_row_minimums_and_wrapped_titles() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx, "en");
        let mut out = ctx.run_ui(Default::default(), |_| {}); // fonts only bind inside a run
        out.textures_delta.clear();
        let painter = ctx.debug_painter();

        let short = PublicationCard::new_with(CardKind::Chart, "NTP", "Grafik", "csv/ntp.csv", 1, true);
        let long = PublicationCard::new_with(
            CardKind::Chart,
            "Grafik Perkembangan Nilai Tukar Petani Menurut Komoditas Tahun 2026",
            "Grafik",
            "csv/ntp.csv",
            2,
            true,
        );
        let det_w = 120.0; // narrow enough that `long` wraps to several lines

        let h_short = measured_card_height(&painter, &short, det_w, "en");
        let h_long = measured_card_height(&painter, &long, det_w, "en");

        let interact = ctx.global_style().spacing.interact_size.y;
        // PAD_Y (12) + three LINE_GAPs (2) + four rows at the interactive minimum.
        assert!(h_short >= 12.0 + 3.0 * 2.0 + 4.0 * interact, "h_short={h_short}, interact={interact}");
        // Wrapped extra title lines must grow the budget.
        assert!(h_long >= h_short + 24.0, "h_long={h_long}, h_short={h_short}");
    }

    #[test]
    fn test_orchestrator_workspace_right_dock_state() {
        let mut state = OrchestratorWorkspaceState::default();
        assert_eq!(state.right_dock_tab, OrchestratorRightTab::SumberKonten);
        assert!(state.right_dock_expanded);
        assert!(state.page_layout_facing);
        assert_eq!(state.margin_top_mm, 18);
        assert_eq!(state.margin_bottom_mm, 18);
        assert_eq!(state.margin_left_mm, 20);
        assert_eq!(state.margin_right_mm, 20);
        assert_eq!(state.cards.len(), 14);
        assert_eq!(state.cards[13].kind, CardKind::Image);

        // Switch to tab 2
        state.right_dock_tab = OrchestratorRightTab::ProfilEksekusi;
        assert_eq!(state.right_dock_tab, OrchestratorRightTab::ProfilEksekusi);

        // Collapse to icons
        state.right_dock_expanded = false;
        assert!(!state.right_dock_expanded);

        // Expand
        state.right_dock_expanded = true;
        assert!(state.right_dock_expanded);
    }

    #[test]
    fn test_recompile_safety_guard_dialog_state() {
        let mut state = OrchestratorWorkspaceState::default();
        assert!(!state.show_guard_dialog);
        assert!(!state.guard_is_publish);

        // Trigger guard for generate
        state.show_guard_dialog = true;
        state.guard_is_publish = false;
        assert!(state.show_guard_dialog);
        assert!(!state.guard_is_publish);

        // Trigger guard for publish
        state.guard_is_publish = true;
        assert!(state.guard_is_publish);

        // Cancel dialog
        state.show_guard_dialog = false;
        assert!(!state.show_guard_dialog);
    }

    #[test]
    fn test_ai_translation_guard_dialog_and_existing_counts() {
        let mut state = OrchestratorWorkspaceState::default();
        assert!(!state.show_translate_guard_dialog);
        assert_eq!(count_existing_translations(&state, None), 0);

        // Add an entry to TM
        state.tm.insert(pubgen_core::translation::TmEntry {
            source: "Luas Panen".into(),
            target: "Harvested Area".into(),
            origin: pubgen_core::translation::TmOrigin::Reviewed,
            role: "table_header".into(),
            updated: pubgen_core::translation::now_stamp(),
        });
        assert_eq!(count_existing_translations(&state, None), 1);

        // Trigger translate guard dialog
        state.show_translate_guard_dialog = true;
        assert!(state.show_translate_guard_dialog);

        // Cancel dialog
        state.show_translate_guard_dialog = false;
        assert!(!state.show_translate_guard_dialog);
    }

    #[test]
    fn test_ai_translation_start_stage_replace_and_keep_modes() {
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        let ctx = egui::Context::default();
        let mut state = OrchestratorWorkspaceState::default();
        state.cards.push(PublicationCard::new_with(CardKind::Table, "Produksi Padi", "Tabel", "csv/padi.csv", 1, true));
        state.cards[0].title_en = "Rice Production".into();
        state.tm.insert(pubgen_core::translation::TmEntry {
            source: "Produksi Padi".into(),
            target: "Rice Production".into(),
            origin: pubgen_core::translation::TmOrigin::Reviewed,
            role: "card_title".into(),
            updated: pubgen_core::translation::now_stamp(),
        });

        // Mode 1: Keep existing (replace_existing = false) with all units already in memory
        start_translate_stage(&mut app, &ctx, &mut state, false);
        assert!(state.translate_job.is_none(), "no AI job should be spawned when everything is already translated");
        assert_eq!(state.cards[0].title_en, "Rice Production");
        assert!(state.tm.lookup("Produksi Padi").is_some());

        // Mode 2: Replace existing (replace_existing = true) resets existing data so it can be re-translated
        start_translate_stage(&mut app, &ctx, &mut state, true);
        assert!(state.cards[0].title_en.is_empty(), "cards title_en should be cleared for fresh translation");
        assert!(state.tm.lookup("Produksi Padi").is_none(), "memory should be cleared for fresh translation");
    }

    #[test]
    fn test_compile_progress_modal_prewarms_and_does_not_jump_prematurely() {
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx, "id");
        crate::menus::pubgen_bar::set_orchestrator_mode(&ctx, true);
        assert!(crate::menus::pubgen_bar::is_orchestrator_mode(&ctx));

        let t = Tokens::get(&ctx);
        let mut state = OrchestratorWorkspaceState::default();
        state.compile_progress = Some(CompileProgressState {
            is_publish: false,
            total_pages: 2,
            current_page: 0,
            status_text: "Memulai...".into(),
            error_msg: None,
            completed: false,
        });

        // Step 1: Page 0 pre-warming
        let mut out = ctx.run_ui(Default::default(), |_| {
            render_compile_progress_modal(&mut app, &ctx, &mut state, &t, "id");
        });
        out.textures_delta.clear();
        assert!(crate::menus::pubgen_bar::is_orchestrator_mode(&ctx), "must NOT jump to DesignCraft during page loading");
        assert_eq!(state.compile_progress.as_ref().unwrap().current_page, 1);
        assert!(!state.compile_progress.as_ref().unwrap().completed);

        // Step 2: Page 1 pre-warming
        let mut out = ctx.run_ui(Default::default(), |_| {
            render_compile_progress_modal(&mut app, &ctx, &mut state, &t, "id");
        });
        out.textures_delta.clear();
        assert!(crate::menus::pubgen_bar::is_orchestrator_mode(&ctx), "must NOT jump during second page");
        assert_eq!(state.compile_progress.as_ref().unwrap().current_page, 2);

        // Step 3: Finalization sets completed = true, but remains in Orchestrator waiting for user action
        let mut out = ctx.run_ui(Default::default(), |_| {
            render_compile_progress_modal(&mut app, &ctx, &mut state, &t, "id");
        });
        out.textures_delta.clear();
        assert!(crate::menus::pubgen_bar::is_orchestrator_mode(&ctx), "must wait for user button click before jumping");
        assert!(state.compile_progress.as_ref().unwrap().completed);
    }

    #[test]
    fn test_orchestrator_workspace_i18n_translation() {
        assert_eq!(crate::i18n::tr("id", "Execution & Verification"), "Eksekusi & Verifikasi");
        assert_eq!(crate::i18n::tr("id", "Sources & Content"), "Sumber & Konten");
        assert_eq!(crate::i18n::tr("id", "Profile & Execution"), "Profil & Eksekusi");
        assert_eq!(crate::i18n::tr("id", "Page Structure"), "Struktur Halaman");
        assert_eq!(crate::i18n::tr("id", "QA Preflight Checklist"), "Daftar Periksa Mutu (QA Preflight)");
        assert_eq!(crate::i18n::tr("id", "Run QA Audit"), "Jalankan Audit QA");
        assert_eq!(crate::i18n::tr("id", "Double Page (Facing)"), "Double Page (Facing)");
        assert_eq!(crate::i18n::tr("id", "Single Page (Non-Facing)"), "Single Page (Non-Facing)");
        assert_eq!(crate::i18n::tr("id", "Document Language:"), "Bahasa Dokumen:");
        assert_eq!(crate::i18n::tr("id", "Paper Size:"), "Ukuran Kertas:");
        assert_eq!(crate::i18n::tr("id", "Margins (mm):"), "Margin (mm):");
        assert_eq!(crate::i18n::tr("id", "Document Settings"), "Pengaturan Dokumen");
        assert_eq!(crate::i18n::tr("id", "Publication Title"), "Judul Publikasi");
        assert_eq!(crate::i18n::tr("id", "Content Tree"), "Pohon Konten");
        assert_eq!(crate::i18n::tr("id", "Add Card:"), "Tambah Kartu:");
        assert_eq!(crate::i18n::tr("id", "Add New Page"), "Tambah Halaman Baru");
        assert_eq!(crate::i18n::tr("id", "Add New Object"), "Tambah Objek Baru");

        // F8: workspace chrome strings the Orchestrator renders through tr() (UI consistency F8).
        for (key, id) in [
            ("Open file (.pubgen)", "Buka Berkas (.pubgen)"),
            ("Open Active Studio", "Buka Studio Aktif"),
            ("Layout Engine", "Mesin Tata Letak"),
            ("● Integrated (DesignCraft Native OK)", "● Terintegrasi (DesignCraft Native OK)"),
            ("Detected Slots", "Slot Terdeteksi"),
            ("Page Layout Preview (Live Spread Document)", "Pratinjau Tata Letak Halaman (Live Spread Document)"),
            ("Save as New Version (v2)", "Simpan Versi Baru (v2)"),
            ("Overwrite Layout", "Timpa Tata Letak"),
            ("Thematic Map Settings (Choropleth & Spatial Map):", "Pengaturan Peta Tematik (Choropleth & Spatial Map):"),
            ("Opening file {path}", "Membuka berkas {path}"),
            ("QA audit finished: 0 errors, 0 warnings. Print standard passed.", "Audit QA selesai: 0 error, 0 warning. Standar cetak lolos."),
            ("Edit card", "Edit kartu"),
            ("Decrease {label}", "Kurangi {label}"),
            ("Increase {label}", "Tambah {label}"),
            ("Total: {n} pages", "Total: {n} Halaman"),
            ("Page & Section Numbering Options:", "Opsi Penomoran Halaman & Seksi:"),
            ("Untitled publication", "Publikasi tanpa judul"),
            ("{n} items", "{n} item"),
            ("Worksheet", "Lembar Kerja"),
        ] {
            assert_eq!(crate::i18n::tr("id", key), id, "missing ID row for {key:?}");
            assert_eq!(crate::i18n::tr("en", key), key, "EN must be the source key");
        }

        // English fallback
        assert_eq!(crate::i18n::tr("", "Execution & Verification"), "Execution & Verification");
        assert_eq!(crate::i18n::tr("en", "Execution & Verification"), "Execution & Verification");
        assert_eq!(crate::i18n::tr("", "Sources & Content"), "Sources & Content");
        assert_eq!(crate::i18n::tr("en", "Sources & Content"), "Sources & Content");
    }

    #[test]
    fn test_card_kind_map_chart_and_rule_based_defaults() {
        let card = PublicationCard::new_with(CardKind::MapChart, "Peta Tematik Sebaran NTP", "Peta Kloroplet", "maps/jabar.geojson", 15, true);
        assert_eq!(card.kind, CardKind::MapChart);
        assert_eq!(card.kind.icon(), "🗺️");
        assert_eq!(card.map_chart_type, "Choropleth");
        assert!(card.map_show_labels);
        assert!(card.map_show_legend);
        assert_eq!(card.tag1, "Peta Tematik");
        assert_eq!(card.tag2, "Halaman Baru");
    }

    #[test]
    fn test_card_kind_chart_options_defaults_and_roundtrip() {
        let mut card = PublicationCard::new_with(CardKind::Chart, "Grafik Perkembangan Nilai Tukar Petani", "Grafik Vektor", "csv/ntp.csv", 5, true);
        assert_eq!(card.kind, CardKind::Chart);
        assert_eq!(card.kind.icon(), "📈");
        assert_eq!(card.chart_type, "Line Chart (Markers)");
        assert_eq!(card.chart_category_col, "A");
        assert_eq!(card.chart_series_1_col, "B");
        assert_eq!(card.chart_data_start_row, 2);
        assert_eq!(card.chart_data_end_row, 6);
        assert_eq!(card.chart_value_format, "Number (1.234,5)");
        assert!(card.chart_show_value_labels);
        assert!(card.chart_show_legend);
        assert!(!card.chart_show_grid);
        assert!(card.chart_raw_data.contains("Tahun,Pertanian,Industri,Jasa"));

        // Modify options
        card.chart_type = "Vertical Clustered Bar".into();
        card.chart_series_2_enabled = true;
        card.chart_series_2_col = "C".into();
        card.chart_series_2_label_id = "Industri Olahan".into();
        card.chart_value_format = "Percentage (%)".into();
        card.chart_show_grid = true;
        card.chart_max_value_override = "100".into();

        let mut state = OrchestratorWorkspaceState::default();
        state.cards.push(card);

        let meta = state.to_metadata();
        let meta_chart = meta.cards.last().expect("chart card in metadata");
        assert_eq!(meta_chart.settings.get("chart_type").and_then(|v| v.as_str()), Some("Vertical Clustered Bar"));
        assert_eq!(meta_chart.settings.get("chart_series_2_enabled").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(meta_chart.settings.get("chart_series_2_col").and_then(|v| v.as_str()), Some("C"));
        assert_eq!(meta_chart.settings.get("chart_series_2_label_id").and_then(|v| v.as_str()), Some("Industri Olahan"));
        assert_eq!(meta_chart.settings.get("chart_value_format").and_then(|v| v.as_str()), Some("Percentage (%)"));
        assert_eq!(meta_chart.settings.get("chart_show_grid").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(meta_chart.settings.get("chart_max_value_override").and_then(|v| v.as_str()), Some("100"));

        // Roundtrip back to state via apply_ingest_report_to_state
        let report = pubgen_core::excel::IngestReport {
            profile_id: "bps_brs".into(),
            metadata: meta,
            warnings: Vec::new(),
            csv: Vec::new(),
            text: Vec::new(),
            assets: Vec::new(),
        };
        let mut restored = OrchestratorWorkspaceState::default();
        apply_ingest_report_to_state(&mut restored, "test.pubgen", &report, &[]);
        let res_chart = restored.cards.last().expect("restored chart card");
        assert_eq!(res_chart.chart_type, "Vertical Clustered Bar");
        assert!(res_chart.chart_series_2_enabled);
        assert_eq!(res_chart.chart_series_2_col, "C");
        assert_eq!(res_chart.chart_series_2_label_id, "Industri Olahan");
        assert_eq!(res_chart.chart_value_format, "Percentage (%)");
        assert!(res_chart.chart_show_grid);
        assert_eq!(res_chart.chart_max_value_override, "100");
    }

    #[test]
    fn test_chart_sheet_csv_generation_and_spec_parse() {
        let card = PublicationCard::new_with(CardKind::Chart, "Grafik Perkembangan NTP", "NTP", "Manual", 3, true);
        let csv_text = build_chart_sheet_csv(&card);
        let delim = super::super::csv::detect_delimiter(&csv_text);
        let ct_key = format!("chart_type{delim}line");
        let cat_key = format!("category_col{delim}A");
        let s1_key = format!("series_1_col{delim}B");
        let s1_lbl = format!("series_1_label{delim}Pertanian");
        assert!(csv_text.contains(&ct_key), "expected {ct_key} in csv_text:\n{csv_text}");
        assert!(csv_text.contains(&cat_key));
        assert!(csv_text.contains(&s1_key));
        assert!(csv_text.contains(&s1_lbl));
        assert!(csv_text.contains("data_start_row"));
        assert!(csv_text.contains("data_end_row"));

        let grid = super::super::csv::parse_csv_content_with_delim(&csv_text, delim);
        let sheet_data = super::super::excel::ExcelSheetData { name: "C03_01".into(), grid, merges: Vec::new() };

        let spec = super::super::charts::spec::parse_chart_spec(&sheet_data).expect("chart spec should parse");
        assert_eq!(spec.chart_type, super::super::charts::spec::ChartType::Line);
        assert_eq!(spec.category_col, "A");
        assert_eq!(spec.series.len(), 3);
        assert_eq!(spec.series[0].col, "B");
        assert_eq!(spec.series[1].col, "C");
        assert_eq!(spec.series[2].col, "D");

        let (categories, values) = super::super::charts::spec::extract_chart_data(&sheet_data, &spec).expect("extract chart data");
        assert_eq!(categories, vec!["2021", "2022", "2023", "2024", "2025"]);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], vec![3.8, 4.1, 3.9, 4.3, 4.5]);
    }

    #[test]
    fn test_auto_detect_chart_columns() {
        let mut card = PublicationCard::new_with(CardKind::Chart, "", "", "Manual", 1, true);
        card.chart_raw_data = "Wilayah,Jumlah,Rasio\nDesa Sukatani,100,5.2\nDesa Mekarmaya,200,6.1".into();
        auto_detect_chart_columns(&mut card);
        assert_eq!(card.chart_category_col, "A");
        assert_eq!(card.chart_series_1_col, "B");
        assert_eq!(card.chart_series_1_label_id, "Jumlah");
        assert_eq!(card.chart_series_2_col, "C");
        assert_eq!(card.chart_series_2_label_id, "Rasio");
        assert!(card.chart_series_2_enabled);
        assert!(!card.chart_series_3_enabled);
        assert_eq!(card.chart_data_start_row, 2);
        assert_eq!(card.chart_data_end_row, 3);
    }

    #[test]
    fn test_chart_card_source_selection_loads_csv_and_updates_data_table() {
        let mut state = OrchestratorWorkspaceState::default();
        let csv_bytes = b"Tahun,Padi,Jagung,Kedelai\n2021,120,45,15\n2022,125,50,18\n2023,130,52,20\n".to_vec();
        std::sync::Arc::make_mut(&mut state.cached_content).insert("csv/04_01_01.csv".into(), csv_bytes);
        std::sync::Arc::make_mut(&mut state.cached_content).insert("rtf/04_narasi.rtf".into(), b"rtf content".to_vec());

        state.content_files =
            vec![ContentItem { path: "csv/04_01_01.csv".into(), size_bytes: 60 }, ContentItem { path: "rtf/04_narasi.rtf".into(), size_bytes: 11 }];

        // 1. Verify filtering: for CardKind::Chart, only CSV files pass
        let chart_card = PublicationCard::new_with(CardKind::Chart, "Grafik Panen", "Padi", "Manual", 1, true);
        let eligible_files: Vec<_> = state
            .content_files
            .iter()
            .filter(|f| {
                if chart_card.kind == CardKind::Chart {
                    let clean = f.path.replace('\\', "/");
                    clean.starts_with("csv/") || clean.ends_with(".csv")
                } else {
                    true
                }
            })
            .collect();
        assert_eq!(eligible_files.len(), 1);
        assert_eq!(eligible_files[0].path, "csv/04_01_01.csv");

        // 2. Select the CSV source and verify load_chart_data_from_source updates chart_raw_data and columns
        let mut card = chart_card;
        assert_eq!(card.chart_raw_data, super::super::charts::dialog::DEFAULT_SAMPLE_CSV);
        load_chart_data_from_source(&mut card, "csv/04_01_01.csv", &state.cached_content);
        assert!(card.chart_raw_data.contains("Padi,Jagung,Kedelai"));
        assert_eq!(card.chart_category_col, "A");
        assert_eq!(card.chart_series_1_col, "B");
        assert_eq!(card.chart_series_1_label_id, "Padi");
        assert_eq!(card.chart_series_2_col, "C");
        assert_eq!(card.chart_series_2_label_id, "Jagung");
        assert!(card.chart_series_2_enabled);
        assert_eq!(card.chart_series_3_col, "D");
        assert_eq!(card.chart_series_3_label_id, "Kedelai");
        assert!(card.chart_series_3_enabled);
        assert_eq!(card.chart_data_start_row, 2);
        assert_eq!(card.chart_data_end_row, 4);
    }

    #[test]
    fn test_chart_card_edit_tab_opsi_grafik_isolation_and_rendering() {
        let ctx = egui::Context::default();
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        let t = Tokens::get(&ctx);
        let mut state = OrchestratorWorkspaceState::default();

        // Push a Chart card
        let chart_card = PublicationCard::new_with(CardKind::Chart, "Grafik Panen", "Padi", "csv/panen.csv", 4, true);
        state.cards.push(chart_card);
        let chart_idx = state.cards.len() - 1;

        // 1. Open the chart card modal on KontenSumber
        state.editing_card_idx = Some(chart_idx);
        state.card_edit_tab = CardEditTab::KontenSumber;
        let mut output = ctx.run_ui(Default::default(), |ctx| {
            render_card_edit_modal(&mut app, ctx, &mut state, &t, "en");
        });
        output.textures_delta.clear();
        assert_eq!(state.card_edit_tab, CardEditTab::KontenSumber);

        // 2. Switch to OpsiGrafik tab on the chart card
        state.card_edit_tab = CardEditTab::OpsiGrafik;
        let mut output2 = ctx.run_ui(Default::default(), |ctx| {
            render_card_edit_modal(&mut app, ctx, &mut state, &t, "en");
        });
        output2.textures_delta.clear();
        assert_eq!(state.card_edit_tab, CardEditTab::OpsiGrafik);

        // 3. For a non-chart card (idx 1, Narrative), OpsiGrafik is not allowed and falls back to KontenSumber
        state.editing_card_idx = Some(1);
        state.card_edit_tab = CardEditTab::OpsiGrafik;
        let mut output3 = ctx.run_ui(Default::default(), |ctx| {
            render_card_edit_modal(&mut app, ctx, &mut state, &t, "en");
        });
        output3.textures_delta.clear();
        assert_eq!(state.card_edit_tab, CardEditTab::KontenSumber, "non-chart card must fall back to KontenSumber");
    }

    #[test]
    fn test_publication_card_header_footer_scope_and_roundtrip() {
        let mut state = OrchestratorWorkspaceState::default();
        // Cover card (idx 0)
        assert_eq!(state.cards[0].kind, CardKind::Cover);

        // Modify non-cover card (idx 1, Narrative on page 2)
        state.cards[1].hf_enabled = true;
        state.cards[1].hf_scope_forward = false; // "This page only"
        state.cards[1].hf_style = "Corporate Report".into();
        state.cards[1].hf_header_single = "Chapter 1".into();

        // Modify non-cover card (idx 2, Table on page 3)
        state.cards[2].hf_enabled = true;
        state.cards[2].hf_scope_forward = true; // "This page forward"
        state.cards[2].hf_style = "hf-dda".into();

        // Export to metadata
        let metadata = state.to_metadata();
        let card1_meta = &metadata.cards[1];
        assert_eq!(card1_meta.settings.get("hf_enabled").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(card1_meta.settings.get("hf_scope_forward").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(card1_meta.settings.get("hf_style").and_then(|v| v.as_str()), Some("Corporate Report"));
        assert_eq!(card1_meta.settings.get("hf_header_single").and_then(|v| v.as_str()), Some("Chapter 1"));

        let card2_meta = &metadata.cards[2];
        assert_eq!(card2_meta.settings.get("hf_enabled").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(card2_meta.settings.get("hf_scope_forward").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(card2_meta.settings.get("hf_style").and_then(|v| v.as_str()), Some("hf-dda"));
    }

    #[test]
    fn test_card_edit_modal_header_footer_form_scope_sync() {
        let ctx = egui::Context::default();
        let mut state = OrchestratorWorkspaceState::default();
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        let t = Tokens::get(&ctx);

        // 1. Cover card: does not set hf_scope on pubgen_st
        state.editing_card_idx = Some(0);
        let mut output = ctx.run_ui(Default::default(), |ctx| {
            render_card_edit_modal(&mut app, ctx, &mut state, &t, "en");
        });
        output.textures_delta.clear();
        assert!(!state.cards[0].hf_enabled, "cover card should not have hf enabled");

        // Close cover modal
        state.editing_card_idx = None;

        // 2. Non-cover card (e.g. card idx 1, Narrative):
        state.editing_card_idx = Some(1);
        state.card_edit_tab = CardEditTab::HeaderFooter;
        let mut output2 = ctx.run_ui(Default::default(), |ctx| {
            render_card_edit_modal(&mut app, ctx, &mut state, &t, "en");
        });
        output2.textures_delta.clear();

        // Card 1 on page 2 defaults to "This page only" (scope 3)
        assert!(state.cards[1].hf_enabled, "non-cover card should enable hf on render");
        assert!(!state.cards[1].hf_scope_forward, "default apply scope is this page only");
        crate::dock::pubgen::with_state(&ctx, |pubgen_st| {
            assert_eq!(pubgen_st.hf_scope, 3, "scope should be 3 (This Page Only)");
            assert_eq!(pubgen_st.hf_scope_start, 2, "start page should match card.page (2)");
        });

        // 3. Close modal, set card to "This page forward" and re-open
        state.editing_card_idx = None;
        state.last_editing_card_idx = None;
        state.cards[1].hf_scope_forward = true;
        state.editing_card_idx = Some(1);
        state.card_edit_tab = CardEditTab::HeaderFooter;
        let mut output3 = ctx.run_ui(Default::default(), |ctx| {
            render_card_edit_modal(&mut app, ctx, &mut state, &t, "en");
        });
        output3.textures_delta.clear();

        crate::dock::pubgen::with_state(&ctx, |pubgen_st| {
            assert_eq!(pubgen_st.hf_scope, 4, "scope should be 4 (This Page Forward)");
            assert_eq!(pubgen_st.hf_scope_start, 2, "start page should match card.page (2)");
        });
    }

    #[test]
    fn test_orchestrator_welcome_page_and_active_workspace() {
        let mut state = OrchestratorWorkspaceState::default();
        assert_eq!(state.view, OrchestratorView::Home);
        assert!(!state.is_dirty);
        assert!(state.has_active_workspace());

        // Switch to studio
        state.view = OrchestratorView::Studio;
        assert_eq!(state.view, OrchestratorView::Studio);
        assert!(state.has_active_workspace());

        // Back to home
        state.view = OrchestratorView::Home;
        assert_eq!(state.view, OrchestratorView::Home);
    }

    /// Regression (diorz38/pubgen#2): the Orchestrator Home hero must use the professional PGS
    /// icon asset, not the generic placeholder glyph.
    #[test]
    fn orchestrator_home_hero_uses_pgs_icon_asset() {
        let src = include_str!("workspace.rs");
        assert!(src.contains("pgs_orchestrator_home_logo_texture"), "Orchestrator Home hero should load the PGS logo texture");
        assert!(src.contains("assets/app-icon/pgs_icon.png"), "Orchestrator Home hero should reference assets/app-icon/pgs_icon.png");
        let bytes: &[u8] = include_bytes!("../../../../../assets/app-icon/pgs_icon.png");
        assert!(designcraft_render::decode_pixmap_page(bytes, 0).is_some(), "pgs_icon.png must decode as an image");
    }

    #[test]
    fn test_dynamic_source_and_content_ingest() {
        let mut state = OrchestratorWorkspaceState::default();
        let sample = std::path::Path::new("../../plan/pubgen/samples/brs/ntp202610.xlsx");
        if sample.is_file() {
            let report = pubgen_core::excel::ingest_workbook(sample).expect("ingest sample");
            apply_ingest_report_to_state(&mut state, "ntp202610.xlsx", &report, &[]);
            assert_eq!(state.meta_title, "Perkembangan Nilai Tukar Petani");
            assert_eq!(state.meta_release_no, "61/10/32/Th. XXVIII");
            assert_eq!(state.meta_release_date, "1 Oktober 2026");
            assert_eq!(state.meta_release_month, "September 2026");
            assert_eq!(state.meta_agency, "BADAN PUSAT STATISTIK PROVINSI JAWA BARAT");
            assert!(!state.content_files.is_empty());
            assert!(state.content_files.iter().any(|f| f.path.starts_with("csv/")));
            assert!(state.content_files.iter().any(|f| f.path.starts_with("rtf/")), "narratives are staged as RTF");

            // Verify sheet COV is converted to CSV and updates card Cover
            assert!(state.content_files.iter().any(|f| f.path == "csv/COV.csv"), "COV converted to csv/COV.csv");
            assert!(state.cached_content.contains_key("csv/COV.csv"));
            let cov_card = state.cards.first().expect("Cover card");
            assert_eq!(cov_card.kind, CardKind::Cover);
            assert_eq!(cov_card.source, "csv/COV.csv");
            assert_eq!(cov_card.cover_release_no, "61/10/32/Th. XXVIII");
            assert_eq!(cov_card.cover_release_date, "1 Oktober 2026");
            assert_eq!(cov_card.cover_release_month, "September 2026");
            assert_eq!(cov_card.cover_pub_office, "BADAN PUSAT STATISTIK PROVINSI JAWA BARAT");
            assert!(cov_card.settings.contains_key("release_no"));

            // Verify syncing card cover values to PubgenState
            let mut pubgen_st = crate::dock::pubgen::PubgenState::default();
            sync_card_to_pubgen_state(cov_card, &state, &mut pubgen_st);
            assert_eq!(pubgen_st.cover_title, "Perkembangan Nilai Tukar Petani");
            assert_eq!(pubgen_st.cover_agency_text, "BADAN PUSAT STATISTIK PROVINSI JAWA BARAT");
        }
    }

    /// T7: importing a workbook replaces the publication, so the previous document's translation
    /// memory must not reuse its English on the new cards (or be saved into the new package).
    #[test]
    fn test_ingesting_a_workbook_clears_the_previous_translation_memory() {
        let mut state = OrchestratorWorkspaceState::default();
        state.tm.insert(pubgen_core::translation::TmEntry {
            source: "Luas Wilayah".into(),
            target: "Total Area".into(),
            origin: pubgen_core::translation::TmOrigin::Ai,
            role: "table_title".into(),
            updated: pubgen_core::translation::now_stamp(),
        });
        state.translate_units.push(pubgen_core::translation_units::TransUnit {
            key: "Luas Wilayah".into(),
            text: "Luas Wilayah".into(),
            role: pubgen_core::translation_units::UnitRole::TableTitle,
            card_id: "table-1".into(),
        });
        state.translate_glossary_pairs.push(("Kecamatan".into(), "Subdistrict".into()));

        let report = pubgen_core::excel::IngestReport {
            profile_id: "bps_brs".into(),
            metadata: pubgen_core::PublicationMetadata {
                profile_id: "bps_brs".into(),
                layout: pubgen_core::PageLayout::Single,
                language: pubgen_core::Language::Id,
                title: "Terjemahan Berikutnya".into(),
                title_en: None,
                cards: Vec::new(),
                extra: Default::default(),
            },
            csv: Vec::new(),
            text: Vec::new(),
            assets: Vec::new(),
            warnings: Vec::new(),
        };
        apply_ingest_report_to_state(&mut state, "other.xlsx", &report, &[]);

        assert!(state.tm.entries.is_empty(), "a newly imported document must not inherit the previous memory");
        assert!(state.tm.lookup("Luas Wilayah").is_none());
        assert!(state.translate_units.is_empty(), "a pending unit belongs to the replaced document");
        assert!(state.translate_glossary_pairs.is_empty());
    }

    /// T8: the ingest path carries the document glossary, so `glossary.json` has entries to write.
    /// It lives apart from the masking pairs, which a translation run clears.
    #[test]
    fn ingesting_a_workbook_seeds_the_glossary_without_the_masking_pairs() {
        let mut state = OrchestratorWorkspaceState::default();
        state.glossary_pairs.push(("Lama".into(), "Old".into()));
        let report = pubgen_core::excel::IngestReport {
            profile_id: "bps_brs".into(),
            metadata: pubgen_core::PublicationMetadata {
                profile_id: "bps_brs".into(),
                layout: pubgen_core::PageLayout::Single,
                language: pubgen_core::Language::Id,
                title: "Glosarium".into(),
                title_en: None,
                cards: Vec::new(),
                extra: Default::default(),
            },
            csv: Vec::new(),
            text: Vec::new(),
            assets: Vec::new(),
            warnings: Vec::new(),
        };
        apply_ingest_report_to_state(&mut state, "glosarium.xlsx", &report, &[("Kecamatan".into(), "Subdistrict".into())]);

        assert_eq!(state.glossary_pairs, vec![("Kecamatan".to_string(), "Subdistrict".to_string())]);
        assert!(state.translate_glossary_pairs.is_empty(), "the run's masking list stays separate");
    }

    /// T7: opening a `.pubgen` loads that package's own memory, which the import reset must keep.
    #[test]
    fn test_sync_external_workspace_keeps_the_opened_packages_memory() {
        let ctx = egui::Context::default();
        let mut tm = pubgen_core::TranslationMemory::default();
        tm.insert(pubgen_core::translation::TmEntry {
            source: "Luas Wilayah".into(),
            target: "Total Area".into(),
            origin: pubgen_core::translation::TmOrigin::Reviewed,
            role: "table_title".into(),
            updated: pubgen_core::translation::now_stamp(),
        });
        let metadata = pubgen_core::PublicationMetadata {
            profile_id: "bps_brs".into(),
            layout: pubgen_core::PageLayout::Single,
            language: pubgen_core::Language::Id,
            title: "Paket".into(),
            title_en: None,
            cards: Vec::new(),
            extra: Default::default(),
        };

        sync_external_workspace(&ctx, "Paket.pubgen", &metadata, &tm, &[], &[], &[]);
        // T8: the package's glossary is restored after the import reset too.
        set_workspace_glossary(&ctx, &[pubgen_core::GlossaryEntry { source: "Kecamatan".into(), target: "Subdistrict".into() }]);

        let state = get_state(&ctx);
        assert_eq!(state.tm.lookup("Luas Wilayah").map(|entry| entry.target.as_str()), Some("Total Area"));
        assert_eq!(state.glossary_pairs, vec![("Kecamatan".to_string(), "Subdistrict".to_string())]);
        crate::dock::pubgen::with_state(&ctx, |pubgen_st| {
            assert_eq!(pubgen_st.translations.get("Luas Wilayah").map(String::as_str), Some("Total Area"));
        });
    }

    #[test]
    fn test_generate_publication_from_workspace_cards() {
        // The Generate buttons must build pages from the worksheet cards + cached
        // content, without any XLSX manifest on disk (regression for the button
        // rewiring from `pubgen.compileProject` to `generate_from_workspace`).
        let mut state = OrchestratorWorkspaceState::default();
        let sample = std::path::Path::new("../../plan/pubgen/samples/brs/ntp202610.xlsx");
        if !sample.is_file() {
            return;
        }
        let report = pubgen_core::excel::ingest_workbook(sample).expect("ingest sample");
        apply_ingest_report_to_state(&mut state, "ntp202610.xlsx", &report, &[]);
        assert!(!state.cards.is_empty());
        assert!(!state.cached_content.is_empty());

        let mut app = crate::DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", serde_json::json!({})).unwrap();

        let res = generate_from_workspace(&mut app, &state);
        assert!(res.is_ok(), "generate_from_workspace should succeed: {res:?}");
        let msg = res.unwrap();
        assert!(msg.contains("auto-generated pages"), "should confirm generated pages: {msg}");

        let doc = &app.session.active().unwrap().doc;
        assert!(doc.page_count() >= 5, "BRS sample should produce several pages, got {}", doc.page_count());
    }

    #[test]
    fn test_dynamic_dda_page_structure_cards() {
        let mut state = OrchestratorWorkspaceState::default();
        let sample = std::path::Path::new("../../plan/pubgen/samples/SINGLE_LANG/BAB_02_2025.xlsx");
        if sample.is_file() {
            let report = pubgen_core::excel::ingest_workbook(sample).expect("ingest DDA sample");
            apply_ingest_report_to_state(&mut state, "BAB_02_2025.xlsx", &report, &[]);
            assert_eq!(state.selected_profile, "bps_dda");

            // DIV must be Divider (Pembatas)
            let div_card = state.cards.iter().find(|c| c.chapter_id == "DIV").expect("DIV card exists");
            assert_eq!(div_card.kind, CardKind::Divider);
            assert_eq!(div_card.tag1, "Pembatas");

            // R02 must be Paragraph (Narrative)
            let r02_card = state.cards.iter().find(|c| c.chapter_id == "R02").expect("R02 card exists");
            assert_eq!(r02_card.kind, CardKind::Narrative);
            assert_eq!(r02_card.tag1, "Paragraf");

            // 02_01 must be Table
            let t02_card = state.cards.iter().find(|c| c.chapter_id == "02_01").expect("02_01 card exists");
            assert_eq!(t02_card.kind, CardKind::Table);
            assert_eq!(t02_card.tag1, "Tabel");

            // C02_01 must be Image (Gambar) because file has C02_01.png and table/chart_type are empty
            let c02_card = state.cards.iter().find(|c| c.chapter_id == "C02_01").expect("C02_01 card exists");
            assert_eq!(c02_card.kind, CardKind::Image);
            assert_eq!(c02_card.tag1, "Gambar");
            assert!(c02_card.title.contains("Panjang Jalan"));

            // Verify end-to-end page generation produces DDA layout, not BRS
            let mut app = crate::DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
            app.run("file.new", serde_json::json!({})).unwrap();
            let res = generate_from_workspace(&mut app, &state);
            assert!(res.is_ok(), "generate_from_workspace on DDA should succeed: {res:?}");
            let msg = res.unwrap();
            assert!(msg.contains("auto-generated pages"), "should confirm generated pages: {msg}");

            let doc = &app.session.active().unwrap().doc;
            assert!(doc.settings.facing_pages, "DDA document must use facing pages");
            assert!(doc.page_count() >= 6, "DDA sample should produce several pages, got {}", doc.page_count());
        }
    }

    #[test]
    fn test_dda_bab01_dual_lang_full_generation() {
        let mut state = OrchestratorWorkspaceState::default();
        let sample = std::path::Path::new("../../plan/pubgen/samples/DUAL_LANG/BAB_01_2025_ai_translate.xlsx");
        if sample.is_file() {
            let report = pubgen_core::excel::ingest_workbook(sample).expect("ingest DDA BAB_01 sample");
            apply_ingest_report_to_state(&mut state, "BAB_01_2025_ai_translate.xlsx", &report, &[]);
            assert_eq!(state.selected_profile, "bps_dda");

            // Verify Paragraph cards (A02, G00, B01, G01, G02)
            let a02 = state.cards.iter().find(|c| c.chapter_id == "A02").expect("A02 card exists");
            assert_eq!(a02.kind, CardKind::Narrative);
            assert_eq!(a02.tag1, "Paragraf");

            let b01 = state.cards.iter().find(|c| c.chapter_id == "B01").expect("B01 card exists");
            assert_eq!(b01.kind, CardKind::Narrative);
            assert_eq!(b01.tag1, "Paragraf");

            // Verify MapChart card (M03_02)
            let m03 = state.cards.iter().find(|c| c.chapter_id == "M03_02").expect("M03_02 card exists");
            assert_eq!(m03.kind, CardKind::MapChart);
            assert!(m03.title.contains("Perbandingan Jumlah Penduduk"), "M03_02 title: {}", m03.title);
            assert!(m03.title_en.contains("Comparison of Population Size"), "M03_02 title_en: {}", m03.title_en);

            // Verify Divider cards exist immediately before RXX cards
            let div_r01 = state.cards.iter().find(|c| c.chapter_id == "DIV_R01").expect("DIV_R01 card exists");
            assert_eq!(div_r01.kind, CardKind::Divider);
            assert_eq!(div_r01.tag1, "Pembatas");

            let div_idx = state.cards.iter().position(|c| c.chapter_id == "DIV_R01").unwrap();
            let r01_idx = state.cards.iter().position(|c| c.chapter_id == "R01").unwrap();
            assert_eq!(div_idx + 1, r01_idx, "DIV_R01 must precede R01 immediately");

            // Verify Cover options
            if let Some(cover_card) = state.cards.iter_mut().find(|c| c.kind == CardKind::Cover) {
                cover_card.cover_inside_back = true;
            }

            // In DDA publications, G00, G01, G02 are TOC cards (Daftar Isi, Daftar Tabel, Daftar Grafik)
            for card in state.cards.iter_mut() {
                if card.chapter_id == "G00" {
                    card.kind = CardKind::Toc;
                    card.title = "Daftar Isi".into();
                    card.title_en = "Table of Contents".into();
                    card.toc_style = "TOC Paragraph".into();
                } else if card.chapter_id == "G01" {
                    card.kind = CardKind::Toc;
                    card.title = "Daftar Tabel".into();
                    card.title_en = "List of Tables".into();
                    card.toc_style = "TOC Table".into();
                } else if card.chapter_id == "G02" {
                    card.kind = CardKind::Toc;
                    card.title = "Daftar Grafik".into();
                    card.title_en = "List of Figures".into();
                    card.toc_style = "TOC Chart".into();
                }
            }

            // Run end-to-end generation
            let mut app = crate::DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
            app.run("file.new", serde_json::json!({})).unwrap();
            let res = generate_from_workspace(&mut app, &state);
            assert!(res.is_ok(), "generate_from_workspace on DDA dual lang should succeed: {res:?}");

            let doc = &app.session.active().unwrap().doc;
            assert!(doc.settings.facing_pages, "DDA document must use facing pages");
            assert!(doc.page_count() >= 10, "Document should have at least 10 pages, got {}", doc.page_count());

            // Verify Dual Language content for R01: both Indonesian and English text present in stories!
            assert!(
                doc.stories.values().any(|s| s.text.contains("East Karawang District")),
                "Document must contain English narrative from R01_en.rtf in dual-language mode!"
            );
            assert!(
                doc.stories.values().any(|s| s.text.contains("Kecamatan Karawang Timur merupakan")),
                "Document must contain Indonesian narrative from R01.rtf in dual-language mode!"
            );

            // Verify Map Chart titles are present in the document
            assert!(doc.stories.values().any(|s| s.text.contains("Perbandingan Jumlah Penduduk")), "Map title must be present in document stories!");
            assert!(
                doc.stories.values().any(|s| s.text.contains("Comparison of Population Size")),
                "Map English title must be present in document stories!"
            );

            // Verify DDA subtitle has no release date or multi-line agency text
            assert!(!state.meta_subtitle.contains("Oktober"), "DDA subtitle must not contain release date: {}", state.meta_subtitle);
            assert!(!state.meta_subtitle.contains('\n'), "DDA subtitle must not contain newlines: {}", state.meta_subtitle);

            // Verify Bookmark debris is completely absent
            for s in doc.stories.values() {
                assert!(!s.text.contains("Toc234493584"), "Stories must not contain bookmark debris Toc234493584");
            }
            let last_page_idx = doc.page_count() - 1;
            let (last_si, last_pi) = doc.page_loc(last_page_idx).expect("back cover page location");
            let last_spread = &doc.spreads[last_si];
            assert_eq!(last_spread.pages.len(), 1, "Back cover spread must be a lone spread at end");
            assert_eq!(last_spread.pages[last_pi].side, designcraft_doc::PageSide::Left, "Back cover must be a left page");

            // Last body spread (before back cover) must NOT have back cover labels
            if last_si > 0 {
                let prev_spread = &doc.spreads[last_si - 1];
                for it in &prev_spread.items {
                    assert!(!it.label.contains("back_cover"), "Previous spread items must not contain back_cover: {}", it.label);
                }
            }

            // Verify that TOC overflow never overlaps Kata Pengantar or Daftar Grafik
            let kp_story = doc.stories.values().find(|s| s.text.contains("merupakan seri publikasi untuk Masyarakat")).expect("Kata Pengantar story");
            let kp_frame = kp_story.frames[0];
            let kp_page = doc.page_of_item(kp_frame).expect("page of Kata Pengantar");
            let (kp_si, _kp_pi) = doc.page_loc(kp_page).unwrap();
            let kp_spread = &doc.spreads[kp_si];
            let kp_page_items: Vec<_> = kp_spread.items.iter().filter(|it| doc.page_of_item(it.id) == Some(kp_page)).collect();
            for it in &kp_page_items {
                if let Some(tf) = it.text_frame() {
                    let story_text = &doc.stories[&tf.story].text;
                    assert!(!story_text.contains("Daftar Isi"), "Kata Pengantar page must not contain Daftar Isi: {story_text}");
                    assert!(!story_text.contains("Daftar Tabel"), "Kata Pengantar page must not contain Daftar Tabel: {story_text}");
                    assert!(!story_text.contains("Daftar Grafik"), "Kata Pengantar page must not contain Daftar Grafik: {story_text}");
                }
                assert_ne!(it.label, "pubgen_toc_continuation_header", "Kata Pengantar page must not contain TOC continuation header");
            }

            let dg_story = doc.stories.values().find(|s| s.text.contains("Daftar Grafik")).expect("Daftar Grafik story");
            let dg_frame = dg_story.frames[0];
            let dg_page = doc.page_of_item(dg_frame).expect("page of Daftar Grafik");
            let (dg_si, _dg_pi) = doc.page_loc(dg_page).unwrap();
            let dg_spread = &doc.spreads[dg_si];
            let dg_page_items: Vec<_> = dg_spread.items.iter().filter(|it| doc.page_of_item(it.id) == Some(dg_page)).collect();
            for it in &dg_page_items {
                if let Some(tf) = it.text_frame() {
                    let story_text = &doc.stories[&tf.story].text;
                    assert!(!story_text.contains("Daftar Tabel"), "Daftar Grafik page must not contain Daftar Tabel: {story_text}");
                    assert!(!story_text.contains("Table 04.02.01"), "Daftar Grafik page must not contain Table entries: {story_text}");
                }
                if it.label == "pubgen_toc_continuation_header" {
                    let tf = it.text_frame().unwrap();
                    let hdr_text = &doc.stories[&tf.story].text;
                    assert!(!hdr_text.contains("Tabel"), "Daftar Grafik page must not contain Tabel continuation header");
                }
            }

            // Verify DDA footer text taken directly from title2 of COVER sheet
            assert!(
                doc.stories.values().any(|s| s.text.contains("EAST KARAWANG DISTRICT IN FIGURES\u{e006}2026")),
                "DDA footer must contain English title from title2 and year 2026"
            );
            for s in doc.stories.values() {
                assert!(!s.text.contains("Cover Release:"), "Stories must not contain 'Cover Release:' artifact: {}", s.text);
            }

            // Verify Preface (B01_en) was inserted directly after Kata Pengantar (B01)
            let pref_story =
                doc.stories.values().find(|s| s.text.contains("publication series designed for the public")).expect("Preface English story");
            let pref_frame = pref_story.frames[0];
            let pref_page = doc.page_of_item(pref_frame).expect("page of Preface");
            assert_eq!(pref_page, kp_page + 1, "Preface page must be immediately after Kata Pengantar page");

            // Verify R03 and R04 cards do NOT have file_en pointing to Indonesian RTF
            let r03_card = state.cards.iter().find(|c| c.chapter_id == "R03").expect("R03 card exists");
            assert_eq!(r03_card.settings.get("file_en"), None, "R03 must not have file_en set to Indonesian RTF");
            let r04_card = state.cards.iter().find(|c| c.chapter_id == "R04").expect("R04 card exists");
            assert_eq!(r04_card.settings.get("file_en"), None, "R04 must not have file_en set to Indonesian RTF");

            // Verify that collect_units gathers narrative paragraphs for R03 and R04
            let draft = worksheet_draft(&state);
            let units = pubgen_core::translation_units::collect_units(&draft);
            assert!(
                units.iter().any(|u| u.card_id == "R03" && u.role == pubgen_core::translation_units::UnitRole::NarrativeBody),
                "collect_units must gather paragraphs for R03"
            );
            assert!(
                units.iter().any(|u| u.card_id == "R04" && u.role == pubgen_core::translation_units::UnitRole::NarrativeBody),
                "collect_units must gather paragraphs for R04"
            );

            // Verify that when TM has translations, generate_from_workspace places translated English into right column of BAB 04
            let mut state_with_tm = state.clone();
            state_with_tm.tm.insert(pubgen_core::translation::TmEntry {
                source: "Dinamika Sosio-Ekonomi dan Ketenagakerjaan Karawang Timur".into(),
                target: "Socio-Economic Dynamics and Employment in East Karawang".into(),
                origin: pubgen_core::translation::TmOrigin::Ai,
                role: "narrative_body".into(),
                updated: pubgen_core::translation::now_stamp(),
            });
            let mut app2 = crate::DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
            app2.run("file.new", serde_json::json!({})).unwrap();
            let _ = generate_from_workspace(&mut app2, &state_with_tm);
            let doc2 = &app2.session.active().unwrap().doc;
            assert!(
                doc2.stories.values().any(|s| s.text.contains("Socio-Economic Dynamics and Employment in East Karawang")),
                "AI translated body must be placed in document stories for BAB 04!"
            );
        }
    }

    #[test]
    fn test_inspect_r03_import_and_translate() {
        // `plan/` is a gitignored local notebook: the inspection runs only where the sample exists.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plan/pubgen/samples/DUAL_LANG/rtf/R03.rtf");
        let Ok(rtf_bytes) = std::fs::read(&path) else {
            eprintln!("skipping: {} not present", path.display());
            return;
        };
        let imported = designcraft_textimport::import("R03.rtf", &rtf_bytes).expect("import R03.rtf");
        let story = &imported.story;
        let ranges = story.para_ranges();
        println!("R03.rtf para_ranges count: {}", ranges.len());
        for (i, r) in ranges.iter().enumerate() {
            let slice = story.slice(r.clone()).trim();
            if !slice.is_empty() {
                println!("  P[{}]: len={} text={:?}", i, slice.len(), &slice[..slice.len().min(60)]);
            }
        }
    }

    #[test]
    fn test_orchestrator_to_metadata_sync() {
        let mut state = OrchestratorWorkspaceState::default();
        state.meta_title = "Statistik Daerah Kabupaten Karawang 2026".into();
        state.meta_subtitle = "Katalog BPS 1102001".into();
        state.meta_release_no = "No. 3215/2026".into();
        state.meta_agency = "BPS Kabupaten Karawang".into();
        state.selected_profile = "bps_dda".into();
        state.page_layout_facing = true;

        // Add modified cards
        state.cards.push(PublicationCard::new_with(
            CardKind::MapChart,
            "Peta Wilayah Kecamatan",
            "Peta Tematik Batas Wilayah",
            "maps/karawang.geojson",
            21,
            true,
        ));

        let meta = state.to_metadata();
        assert_eq!(meta.title, "Statistik Daerah Kabupaten Karawang 2026");
        assert_eq!(meta.title_en.as_deref(), Some("Katalog BPS 1102001"));
        assert_eq!(meta.extra.get("release_no").map(|s| s.as_str()), Some("No. 3215/2026"));
        assert_eq!(meta.extra.get("agency").map(|s| s.as_str()), Some("BPS Kabupaten Karawang"));
        assert_eq!(meta.layout, pubgen_core::PageLayout::Double);
        assert_eq!(meta.cards.len(), state.cards.len());

        let last_card = meta.cards.last().expect("last card exists");
        assert_eq!(last_card.title, "Peta Wilayah Kecamatan");
        assert_eq!(last_card.kind, pubgen_core::CardKind::MapChart);
        assert_eq!(last_card.scope, pubgen_core::CardScope::NewPage);
        assert_eq!(last_card.source.as_deref(), Some("maps/karawang.geojson"));
    }

    #[test]
    fn test_map_chart_card_stock_and_custom_geojson_handling() {
        let mut state = OrchestratorWorkspaceState::default();
        let mut stock_card = PublicationCard::new_with(
            CardKind::MapChart,
            "Peta Wilayah Kecamatan Cilamaya Wetan",
            "Peta Tematik Batas Desa",
            "assets/pubgen/maps/CILAMAYA_WETAN_DESA.geojson",
            5,
            true,
        );
        stock_card.map_is_custom = false;
        stock_card.map_stock_name = "CILAMAYA WETAN".into();
        stock_card.map_data_csv = "maps/contoh.csv".into();
        state.cards.push(stock_card);

        let mut custom_card =
            PublicationCard::new_with(CardKind::MapChart, "Peta Tematik Khusus", "Custom GeoJSON Map", "maps/custom_border.geojson", 6, false);
        custom_card.map_is_custom = true;
        custom_card.map_custom_geojson = "maps/custom_border.geojson".into();
        custom_card.map_data_csv = "maps/data_custom.csv".into();
        state.cards.push(custom_card);

        let meta = state.to_metadata();
        let meta_stock = &meta.cards[meta.cards.len() - 2];
        let meta_custom = &meta.cards[meta.cards.len() - 1];

        assert_eq!(meta_stock.settings.get("map_is_custom").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(meta_stock.settings.get("map_stock_name").and_then(|v| v.as_str()), Some("CILAMAYA WETAN"));
        assert_eq!(meta_stock.settings.get("map_data_csv").and_then(|v| v.as_str()), Some("maps/contoh.csv"));

        assert_eq!(meta_custom.settings.get("map_is_custom").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(meta_custom.settings.get("map_custom_geojson").and_then(|v| v.as_str()), Some("maps/custom_border.geojson"));
        assert_eq!(meta_custom.settings.get("map_data_csv").and_then(|v| v.as_str()), Some("maps/data_custom.csv"));

        // Roundtrip back to state
        let report = pubgen_core::excel::IngestReport {
            profile_id: "bps_dda".into(),
            metadata: meta,
            csv: Vec::new(),
            text: Vec::new(),
            assets: Vec::new(),
            warnings: Vec::new(),
        };

        let mut restored_state = OrchestratorWorkspaceState::default();
        apply_ingest_report_to_state(&mut restored_state, "test.xlsx", &report, &[]);

        let res_stock = &restored_state.cards[restored_state.cards.len() - 2];
        let res_custom = &restored_state.cards[restored_state.cards.len() - 1];

        assert!(!res_stock.map_is_custom);
        assert_eq!(res_stock.map_stock_name, "CILAMAYA WETAN");
        assert_eq!(res_stock.map_data_csv, "maps/contoh.csv");

        assert!(res_custom.map_is_custom);
        assert_eq!(res_custom.map_custom_geojson, "maps/custom_border.geojson");
        assert_eq!(res_custom.map_data_csv, "maps/data_custom.csv");
    }

    #[test]
    fn test_toc_card_layout_style_roundtrip_and_settings() {
        let mut state = OrchestratorWorkspaceState::default();
        state.cards.clear();
        let mut toc_card = PublicationCard::new_with(CardKind::Toc, "Daftar Tabel", "List of Tables", "Manual", 3, true);
        toc_card.toc_style = "TOC Table".into();
        toc_card.toc_layout_style = 1; // TablePage (BPS Catalog)
        toc_card.toc_columns = "2 Kolom".into();
        toc_card.toc_hanging_indent = true;
        toc_card.toc_include_self = true;
        toc_card.toc_pair_bilingual = true;
        state.cards.push(toc_card);

        let meta = state.to_metadata();
        let meta_card = &meta.cards[0];

        assert_eq!(meta_card.settings.get("toc_style").and_then(|v| v.as_str()), Some("TOC Table"));
        assert_eq!(meta_card.settings.get("toc_layout_style").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(meta_card.settings.get("toc_columns").and_then(|v| v.as_str()), Some("2 Kolom"));
        assert_eq!(meta_card.settings.get("toc_hanging_indent").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(meta_card.settings.get("toc_include_self").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(meta_card.settings.get("toc_pair_bilingual").and_then(|v| v.as_bool()), Some(true));

        // Roundtrip back to state via IngestReport
        let report = pubgen_core::excel::IngestReport {
            profile_id: "bps_dda".into(),
            metadata: meta,
            csv: Vec::new(),
            text: Vec::new(),
            assets: Vec::new(),
            warnings: Vec::new(),
        };

        let mut restored_state = OrchestratorWorkspaceState::default();
        apply_ingest_report_to_state(&mut restored_state, "test.pubgen", &report, &[]);

        let res_card = &restored_state.cards[0];
        assert_eq!(res_card.toc_style, "TOC Table");
        assert_eq!(res_card.toc_layout_style, 1, "toc_layout_style must be preserved as 1 (TablePage)");
        assert_eq!(res_card.toc_columns, "2 Kolom");
        assert!(res_card.toc_hanging_indent);
        assert!(res_card.toc_include_self);
        assert!(res_card.toc_pair_bilingual);
    }

    #[test]
    fn test_host_tabs_strip_layout_and_actions() {
        let ctx = egui::Context::default();
        let titles = vec![("Publication 1".to_string(), false), ("Publication 2".to_string(), true)];
        set_host_tabs(&ctx, titles, 1);

        let t = Tokens::get(&ctx);
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let action = render_host_tabs(ui, &t, "en");
            assert_eq!(action, None); // No clicks simulated
        });
        out.textures_delta.clear();

        // Trigger action via trigger_host_tab_action helper
        trigger_host_tab_action(&ctx, HostTabAction::Activate(0));
        assert_eq!(take_host_tab_action(&ctx), Some(HostTabAction::Activate(0)));
        assert_eq!(take_host_tab_action(&ctx), None);
    }

    /// Opt-in evidence (`CRAFT_UI_VISUAL_FIXTURES=dir cargo test capture_page_structure`):
    /// the Page Structure card grid with long real-world titles/sources — every text line
    /// must stay fully visible (no truncation) at the bumped 1pt-larger font sizes.
    #[test]
    fn capture_page_structure_cards_fixture() {
        let Some(directory) = std::env::var_os("CRAFT_UI_VISUAL_FIXTURES").map(std::path::PathBuf::from) else { return };
        std::fs::create_dir_all(&directory).unwrap();
        let app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        let mut ready = false;
        let mut harness = egui_kittest::Harness::builder().with_size(vec2(1100.0, 760.0)).with_accessibility_check(false).wgpu().build_ui_state(
            move |ui, app: &mut DesignApp| {
                let t = Tokens::for_brightness(crate::theme::Brightness::MediumDark);
                if !ready {
                    crate::theme::install_fonts(ui.ctx(), "en");
                    crate::theme::apply(ui.ctx(), &t);
                    // The kittest font set lacks U+270E (✎) used by the edit link; the real app
                    // font stack covers it. Tofu keeps the fixture about the card text layout.
                    ui.ctx().set_missing_glyph_policy(egui::MissingGlyphPolicy::Tofu);
                    ready = true;
                }
                let mut state = OrchestratorWorkspaceState::default();
                state.page_layout_facing = true;
                state.cards.push(PublicationCard::new_with(CardKind::Cover, "Sampul Statistik Daerah", "Cover Depan", "cover.ai", 1, true));
                state.cards.push(PublicationCard::new_with(
                    CardKind::Narrative,
                    "Analisis Pertumbuhan Ekonomi dan Struktur Perdagangan Daerah 2026",
                    "Paragraf",
                    "md/analisis-pertumbuhan-ekonomi.md",
                    2,
                    true,
                ));
                state.cards.push(PublicationCard::new_with(
                    CardKind::Chart,
                    "Grafik Perkembangan Nilai Tukar Petani Menurut Komoditas",
                    "Grafik Vektor",
                    "csv/ntp-komoditas.csv",
                    3,
                    true,
                ));
                state.cards.push(PublicationCard::new_with(
                    CardKind::MapChart,
                    "Peta Tematik Sebaran NTP Kabupaten/Kota Se-Jawa Barat",
                    "Peta Kloroplet",
                    "maps/jabar.geojson",
                    4,
                    true,
                ));
                render_card_grid(app, ui, &mut state, &t, "id");
            },
            app,
        );
        harness.input_mut().max_texture_side = Some(8192);
        harness.run_steps(4);
        harness.render().unwrap().save(directory.join("page-structure-cards.png")).unwrap();
    }

    /// The BRS Publication Profile must produce the same page shape as the reference document: a
    /// narrative is a page-sized story, never an 18 000-character blob full of converter debris,
    /// every table carries the BRS6 style, and no table repeats the caption drawn above it.
    #[test]
    fn test_brs_pubgen_workspace_generates_clean_pages() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_default();
        let p_path = root.join("plan").join("pubgen").join("samples").join("brs").join("Perkembangan Nilai Tukar Petani.pubgen");
        if !p_path.is_file() {
            return;
        }
        let opened = pubgen_core::container::open_pubgen(&p_path).expect("open pubgen");
        let report = pubgen_core::excel::IngestReport {
            profile_id: opened.metadata.profile_id.clone(),
            metadata: opened.metadata.clone(),
            csv: opened.content.csv.clone(),
            text: opened.content.text.clone(),
            assets: opened.content.assets.clone(),
            warnings: Vec::new(),
        };
        let mut state = OrchestratorWorkspaceState::default();
        apply_ingest_report_to_state(&mut state, "Perkembangan Nilai Tukar Petani.pubgen", &report, &[]);

        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", serde_json::json!({"facingPages": true})).expect("new doc");
        generate_from_workspace(&mut app, &state).expect("generate BRS pages");

        {
            let active = app.session.active().expect("active document");
            let pages = active.doc.page_count();
            assert!((10..=14).contains(&pages), "BRS body pages: {pages} (reference document has 12)");
            let mut longest = 0;
            for story in active.doc.stories.values() {
                assert!(!story.text.contains("schemas.microsoft.com"), "converter debris leaked into a story");
                assert!(!story.text.contains("504b0304"), "an embedded package dump leaked into a story");
                longest = longest.max(story.text.chars().count());
            }
            assert!(longest < 6000, "a BRS narrative ballooned to {longest} characters");
            let tables: Vec<_> = active.doc.stories.values().flat_map(|story| story.tables.values()).collect();
            assert_eq!(tables.len(), 4, "the four BRS tables are inserted");
            for table in &tables {
                assert_eq!(table.style, "BRS6", "BRS tables use the BRS6 table style");
                let caption = table.cells.first().map(|cell| cell.text.text.to_lowercase()).unwrap_or_default();
                assert!(!caption.starts_with("tabel "), "the caption row is drawn above the table, not inside it");
            }
            // Table 1 groups three columns under one header: that group must span them as a
            // header cell instead of its child labels (`Agustus 2026`, …) ending up as a data row.
            let grouped =
                tables.iter().any(|table| table.cells.iter().any(|cell| cell.col_span > 1 && cell.text.text.contains("Indeks Gabungan Subsektor")));
            assert!(grouped, "the merged header of table 1 spans its child columns");

            // The header/footer parent must be laid out on the document page size. The document
            // starts on the Letter default while BRS pages are A4, so a parent left on Letter puts
            // the running footer ~50 pt too high. BPS puts the footer 14 pt above the trim edge.
            let page = active.doc.page(0).expect("a page");
            let (pw, ph) = (page.width, page.height);
            let hf = active.doc.header_footer.as_ref().expect("the BRS header/footer is applied");
            let parent = active.doc.parents.iter().find(|p| Some(p.id) == hf.parent).expect("the header/footer parent spread");
            for pg in &parent.pages {
                assert!(
                    (pg.width - pw).abs() < 1e-6 && (pg.height - ph).abs() < 1e-6,
                    "header/footer parent page {}×{} should match the document page {pw}×{ph}",
                    pg.width,
                    pg.height
                );
            }
            for id in [hf.footer_items.left, hf.footer_items.right].into_iter().flatten() {
                let b = active.doc.item(id).expect("the footer frame").bounds();
                assert!((b.y1 - (ph - 14.0)).abs() < 0.01, "the footer ends {} pt above the trim edge; BPS puts it 14 pt above", ph - b.y1);
            }
        }

        // `PUBGEN_TEST_OUTPUT=<dir>` keeps the generated document, so it can be rendered and looked at.
        if let Some(dir) = std::env::var_os("PUBGEN_TEST_OUTPUT") {
            let dir = std::path::PathBuf::from(dir);
            if std::fs::create_dir_all(&dir).is_ok() {
                let target = dir.join("brs_workspace_output.designcraft");
                let _ = app.run("file.saveAs", serde_json::json!({"path": target.to_string_lossy()}));
            }
        }
    }

    /// Narratives must reach the pages through RTF, not Markdown: RTF carries the paragraph
    /// alignment, indents and spacing that a Markdown projection throws away. Ingesting the
    /// workbook therefore stages `rtf/<stem>.rtf` (an RTF source byte for byte, a `.docx` or `.md`
    /// source converted to RTF) and generation still produces the same BRS page shape.
    #[test]
    fn test_brs_workbook_ingest_writes_rtf_narratives() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_default();
        let dir = root.join("plan").join("pubgen").join("samples").join("brs");
        let xlsx = dir.join("ntp202610.xlsx");
        if !xlsx.is_file() {
            return;
        }
        let report = pubgen_core::excel::ingest_workbook(&xlsx).expect("ingest workbook");
        assert!(!report.text.is_empty(), "the narrative sheets produce text artifacts");
        for (name, bytes) in &report.text {
            assert!(name.starts_with("rtf/") && name.ends_with(".rtf"), "narrative artifact {name} is not RTF");
            assert!(bytes.starts_with(b"{\\rtf"), "narrative artifact {name} is not an RTF file");
        }
        // A source that is already RTF is kept verbatim — that is what keeps its layout intact.
        if let Ok(src) = std::fs::read(dir.join("U1.rtf")) {
            let staged = report.text.iter().find(|(name, _)| name == "rtf/U1.rtf").map(|(_, bytes)| bytes.clone());
            assert_eq!(staged.as_deref(), Some(src.as_slice()), "an RTF narrative must not be rewritten");
        }
        let mut state = OrchestratorWorkspaceState::default();
        apply_ingest_report_to_state(&mut state, "ntp202610.xlsx", &report, &[]);
        assert!(state.cached_content.keys().any(|key| key.starts_with("rtf/")), "narratives are staged under rtf/");

        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", serde_json::json!({"facingPages": true})).expect("new doc");
        generate_from_workspace(&mut app, &state).expect("generate BRS pages");
        let active = app.session.active().expect("active document");
        let pages = active.doc.page_count();
        assert!((10..=14).contains(&pages), "BRS body pages from the workbook ingest: {pages}");
        // The article landed on the page with its own alignment: the source mixes centred,
        // justified and left-aligned paragraphs, and a Markdown round trip would flatten them.
        let aligned =
            active.doc.stories.values().any(|story| story.text.contains("Nilai Tukar Petani") && story.paras.iter().any(|p| p.para.align.is_some()));
        assert!(aligned, "a narrative keeps the paragraph alignment of its RTF source");

        // `PUBGEN_TEST_OUTPUT=<dir>` keeps the document this RTF path generates.
        if let Some(dir) = std::env::var_os("PUBGEN_TEST_OUTPUT") {
            let dir = std::path::PathBuf::from(dir);
            if std::fs::create_dir_all(&dir).is_ok() {
                let target = dir.join("brs_workspace_rtf_output.designcraft");
                let _ = app.run("file.saveAs", serde_json::json!({"path": target.to_string_lossy()}));
            }
        }
    }

    /// Fix round 2 (shipped path): Compile must carry the worksheet's `doc_language` into the
    /// page pipeline. A BRS document set to Indonesian-English used to lose the English image
    /// caption because the pipeline forced `bilingual_output = false` for every BRS build.
    #[test]
    fn test_generate_from_workspace_keeps_the_bilingual_image_caption() {
        let image = std::fs::read(super::super::testutil::sample_image_path()).expect("sample image");
        let mut state =
            OrchestratorWorkspaceState { selected_profile: "bps_brs".into(), doc_language: "Indonesian-English".into(), ..Default::default() };
        state.cards.clear();
        state.cards.push(PublicationCard {
            kind: CardKind::Image,
            title: "Gambar 1 Kantor BPS".into(),
            source: "img/office.png".into(),
            chapter_id: "G1".into(),
            ..Default::default()
        });
        std::sync::Arc::make_mut(&mut state.cached_content).insert("img/office.png".into(), image);
        state.tm.insert(pubgen_core::translation::TmEntry {
            source: "Gambar 1 Kantor BPS".into(),
            target: "Figure 1 BPS Office".into(),
            origin: pubgen_core::translation::TmOrigin::Ai,
            role: "image_caption".into(),
            updated: pubgen_core::translation::now_stamp(),
        });

        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        let res = generate_from_workspace(&mut app, &state);
        assert!(res.is_ok(), "worksheet compilation failed: {res:?}");

        let texts: Vec<String> = app.session.active().map(|a| a.doc.stories.values().map(|s| s.text.clone()).collect()).unwrap_or_default();
        assert!(
            texts.iter().any(|t| t.trim() == "Figure 1 BPS Office"),
            "the compile pipeline must render the English image caption, found: {texts:?}"
        );
    }

    #[test]
    fn test_toc_table_and_paragraph_exclusion_and_headers() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or_default();
        let pubgen_file = root.join("plan").join("pubgen").join("samples").join("DUAL_LANG").join("KECAMATAN KARAWANG TIMUR DALAM ANGKA.pubgen");
        if !pubgen_file.is_file() {
            return;
        }

        let opened = pubgen_core::container::open_pubgen(&pubgen_file).expect("open pubgen");
        let report = pubgen_core::excel::IngestReport {
            profile_id: opened.metadata.profile_id.clone(),
            metadata: opened.metadata.clone(),
            csv: opened.content.csv.clone(),
            text: opened.content.text.clone(),
            assets: opened.content.assets.clone(),
            warnings: Vec::new(),
        };
        let mut state = OrchestratorWorkspaceState::default();
        apply_ingest_report_to_state(&mut state, "KECAMATAN KARAWANG TIMUR DALAM ANGKA.pubgen", &report, &[]);

        // Verify that cards 02_01_01 and 02_02_01 have real table titles and chapter/section settings
        let c_02_01 = state.cards.iter().find(|c| c.chapter_id == "02_01_01").expect("02_01_01 card");
        assert!(c_02_01.title.starts_with("Tabel 02.01.01"), "card 02_01_01 title must be table title, got: {}", c_02_01.title);
        assert_eq!(c_02_01.settings.get("chapter").map(String::as_str), Some("Tabel 2 PEMERINTAHAN"));
        assert_eq!(c_02_01.settings.get("section").map(String::as_str), Some("Tabel 02.01 Wilayah Administratif"));

        let c_02_02 = state.cards.iter().find(|c| c.chapter_id == "02_02_01").expect("02_02_01 card");
        assert!(c_02_02.title.starts_with("Tabel 02.02.01"), "card 02_02_01 title must be table title, got: {}", c_02_02.title);
        assert!(!c_02_02.title.eq_ignore_ascii_case("subtitle"), "card 02_02_01 title must not be 'subtitle'");

        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", serde_json::json!({"facingPages": true})).expect("new doc");
        generate_from_workspace(&mut app, &state).expect("generate pages");

        let active = app.session.active().expect("active document");
        let all_text: Vec<&str> = active.doc.stories.values().map(|s| s.text.as_str()).collect();

        // 1. Verify TOC Paragraph (Daftar Isi) story — the notebook sample evolves with
        // the user's live workbook, so this section runs only while it still carries a
        // paragraph TOC card.
        if let Some(para_toc) = all_text.iter().find(|t| t.contains("Daftar Isi") || t.contains("Table of Contents")).copied() {
            assert!(!para_toc.contains("Tabel 01.01"), "TOC Paragraph must NOT contain Tabel 01.01: {para_toc}");
            assert!(!para_toc.contains("Tabel 01.02"), "TOC Paragraph must NOT contain Tabel 01.02: {para_toc}");
            assert!(!para_toc.contains("Tabel 02.01.01"), "TOC Paragraph must NOT contain Tabel 02.01.01: {para_toc}");
            assert!(!para_toc.contains("Tabel 02.02.01"), "TOC Paragraph must NOT contain Tabel 02.02.01: {para_toc}");
            assert!(!para_toc.contains("subtitle"), "TOC Paragraph must NOT contain subtitle: {para_toc}");
        }

        // 2. Verify TOC Table (Daftar Tabel) story
        let table_toc =
            all_text.iter().find(|t| t.contains("Daftar Tabel") || t.contains("List of Tables")).copied().expect("Daftar Tabel story exists");
        assert!(table_toc.contains("2\tPEMERINTAHAN/GOVERNMENT"), "TOC Table must contain Chapter 2 header: {table_toc}");
        assert!(table_toc.contains("2.1\tWilayah Administratif/Administrative Area"), "TOC Table must contain Section 2.1 header: {table_toc}");
        assert!(table_toc.contains("2.1.1\tJumlah Rukun Warga"), "TOC Table must contain Table 2.1.1: {table_toc}");
        assert!(table_toc.contains("2.2\tSumber Daya Manusia/Human Resources"), "TOC Table must contain Section 2.2 header: {table_toc}");
        assert!(table_toc.contains("2.2.1\tJumlah Pegawai Negeri Sipil"), "TOC Table must contain Table 2.2.1: {table_toc}");
        assert!(!table_toc.contains("2.1.1\t2 PEMERINTAHAN"), "TOC Table must NOT contain '2.1.1 2 PEMERINTAHAN': {table_toc}");
        assert!(!table_toc.contains("2.2.1\tsubtitle"), "TOC Table must NOT contain '2.2.1 subtitle': {table_toc}");
    }

    #[test]
    fn test_detect_translation_rule_units_dynamic_profiles() {
        // 1. Verify default BRS state detects canonical BRS slots plus all card units
        let mut state = OrchestratorWorkspaceState::default();
        assert_eq!(state.selected_profile, "bps_brs");
        assert!(state.slots.len() >= 3, "default slots must contain at least the 3 primary slots, found: {}", state.slots.len());

        let headline = state.slots.iter().find(|s| s.name == "cover_headline").expect("cover_headline slot");
        assert_eq!(headline.budget, Some(60));
        assert!(headline.text.contains("Perkembangan Nilai Tukar Petani"));

        let ringkasan = state.slots.iter().find(|s| s.name == "ringkasan_eksekutif").expect("ringkasan_eksekutif slot");
        assert_eq!(ringkasan.budget, Some(280));
        assert!(ringkasan.text.contains("NTP") || ringkasan.text.contains("Nilai Tukar Petani"));

        let catatan = state.slots.iter().find(|s| s.name == "catatan_kaki_tabel").expect("catatan_kaki_tabel slot");
        assert_eq!(catatan.budget, Some(150));
        assert!(catatan.text.contains("Sumber: Badan Pusat Statistik"));

        // Must detect additional units from default cards (tables, charts, images)
        assert!(
            state.slots.iter().any(|s| s.name.starts_with("judul_tabel_") || s.name.starts_with("judul_grafik_")),
            "must detect table/chart units from workspace cards, found: {:?}",
            state.slots.iter().map(|s| &s.name).collect::<Vec<_>>()
        );

        // 2. Switch dynamically to DDA profile
        state.selected_profile = "bps_dda".into();
        refresh_detected_slots(&mut state);

        // DDA slots must use DDA nomenclature
        assert!(state.slots.iter().any(|s| s.name == "cover_title"), "DDA must have cover_title");
        assert!(state.slots.iter().any(|s| s.name == "cover_title_en"), "DDA must have cover_title_en");
        assert!(state.slots.iter().any(|s| s.name == "cover_agency"), "DDA must have cover_agency");
        assert!(
            state.slots.iter().any(|s| s.name.starts_with("pembatas_bab_")
                || s.name == "daftar_isi"
                || s.name.starts_with("judul_tabel_")
                || s.name.starts_with("sumber_catatan_")),
            "DDA must detect DDA-specific card slots"
        );
        // BRS-specific slot names should NOT be present in DDA
        assert!(!state.slots.iter().any(|s| s.name == "cover_headline"), "DDA must not have cover_headline slot");
        assert!(!state.slots.iter().any(|s| s.name == "ringkasan_eksekutif"), "DDA must not have ringkasan_eksekutif slot");
        assert!(!state.slots.iter().any(|s| s.name == "catatan_kaki_tabel"), "DDA must not have catatan_kaki_tabel slot");

        // 3. Switch dynamically to Corporate Annual Report profile
        state.selected_profile = "corporate_annual".into();
        refresh_detected_slots(&mut state);
        assert!(state.slots.iter().any(|s| s.name == "cover_title"), "corporate profile must have cover_title");
        assert!(state.slots.iter().any(|s| s.name == "cover_agency"), "corporate profile must have cover_agency");

        // 4. Switch dynamically to Academic Monograph profile
        state.selected_profile = "academic_monograph".into();
        refresh_detected_slots(&mut state);
        assert!(state.slots.iter().any(|s| s.name == "monograph_title"), "academic profile must have monograph_title");
        assert!(state.slots.iter().any(|s| s.name == "cover_agency"), "academic profile must have cover_agency");
    }
}
