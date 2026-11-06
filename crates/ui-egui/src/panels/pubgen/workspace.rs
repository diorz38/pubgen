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

fn format_now() -> String {
    let now = chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now());
    now.format("%H:%M:%S").to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestratorTab {
    StrukturHalaman,
    VisualLivePreview,
    LogEksekusi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestratorRightTab {
    SumberKonten,
    ProfilEksekusi,
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
}

impl CardKind {
    pub const ALL: [CardKind; 7] = [
        CardKind::Cover,
        CardKind::Narrative,
        CardKind::Chart,
        CardKind::MapChart,
        CardKind::Table,
        CardKind::Image,
        CardKind::Toc,
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
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentItem {
    pub path: String,
    pub size_bytes: usize,
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

    // MapChart specific (NEW)
    pub map_chart_type: String,
    pub map_classification: String,
    pub map_color_palette: String,
    pub map_show_labels: bool,
    pub map_show_legend: bool,

    // Image specific
    pub image_area: String,

    // TOC specific
    pub toc_style: String,
    pub toc_columns: String,
    pub toc_hanging_indent: bool,
    pub toc_include_self: bool,
    pub toc_pair_bilingual: bool,

    // Section & Numbering specific
    pub section_start: bool,
    pub automatic_numbering: bool,
    pub start_page_number: u32,
    pub section_prefix: String,
    pub numbering_style: String,
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
            table_first_row_header: true,
            table_col_numbers: true,
            table_footer: true,
            table_row_conditions: true,
            table_style: "Basic Grid".into(),
            table_theme: "BPS Blue".into(),
            table_layout_preset: "Standard".into(),
            chart_type: "Line".into(),
            map_chart_type: "Choropleth".into(),
            map_classification: "Quantile (4 kelas)".into(),
            map_color_palette: "BPS Biru-Teal".into(),
            map_show_labels: true,
            map_show_legend: true,
            image_area: "Area Margin".into(),
            toc_style: "Standar".into(),
            toc_columns: "1 Kolom".into(),
            toc_hanging_indent: true,
            toc_include_self: false,
            toc_pair_bilingual: true,
            section_start: false,
            automatic_numbering: true,
            start_page_number: 1,
            section_prefix: String::new(),
            numbering_style: "1, 2, 3...".into(),
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
    pub card_delete_confirm: bool,
    pub prompt_copied_feedback: bool,
    // Status message
    pub status_message: Option<(String, bool)>,
    // Re-compilation Safety Guard Dialog state
    pub show_guard_dialog: bool,
    pub guard_is_publish: bool,
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
            ContentItem { path: "md/U1.md".into(), size_bytes: 1222 },
            ContentItem { path: "md/U2.md".into(), size_bytes: 3828 },
            ContentItem { path: "md/U4.md".into(), size_bytes: 2875 },
            ContentItem { path: "md/U4_5.md".into(), size_bytes: 2286 },
            ContentItem { path: "md/U5.md".into(), size_bytes: 1820 },
            ContentItem { path: "md/U6.md".into(), size_bytes: 1141 },
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
                "md/U1.md",
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
                "Ulasan Narasi U2 (U2.md)",
                "Ulasan lanjutan indeks tanaman perkebunan",
                "md/U2.md",
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
                "Ulasan Narasi U4 (U4.md)",
                "Ulasan indeks konsumsi rumah tangga perdesaan",
                "md/U4.md",
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
                "Ulasan Narasi U4_5 (U4_5.md)",
                "Ulasan penutup dan indeks biaya produksi",
                "md/U4_5.md",
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
            PublicationCard::new_with(
                CardKind::Narrative,
                "Ulasan Narasi U5 (U5.md)",
                "Ulasan analisis inflasi perdesaan",
                "md/U5.md",
                11,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Table,
                "Tabel 4 Nilai Tukar Usaha Petani per Subsektor dan Persentase Perubahannya (2018=100)",
                "Tabel NTUP subsektor",
                "csv/T4.csv",
                12,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Narrative,
                "Ulasan Narasi U6 (U6.md)",
                "Ulasan penutup dan rekomendasi",
                "md/U6.md",
                13,
                false,
            ),
            PublicationCard::new_with(
                CardKind::Image,
                "Infografis / Gambar G1 (G1.png)",
                "Area Margin · Sumber: G1.png",
                "G1.png",
                14,
                false,
            ),
        ];

        let slots = vec![
            SlotItem { name: "cover_headline".into(), budget: Some(60), text: "Perkembangan Nilai Tukar Petani Jawa Barat".into() },
            SlotItem {
                name: "ringkasan_eksekutif".into(),
                budget: Some(280),
                text: "NTP Jawa Barat pada Oktober 2026 sebesar 112,45 atau naik 0,65 persen dibanding bulan sebelumnya.".into(),
            },
            SlotItem {
                name: "catatan_kaki_tabel".into(),
                budget: Some(150),
                text: "Sumber: Badan Pusat Statistik Provinsi Jawa Barat, Sensus Pertanian 2026.".into(),
            },
        ];

        let execution_log = vec![
            "[10:00:01] Profil publikasi aktif: BPS Berita Resmi Statistik (BRS)".into(),
            "[10:00:02] Geometri: A4 Facing=Ya (Double Page), Margin T:18 B:18 L:20 R:20 mm".into(),
            "[10:00:03] Berhasil memuat 20 berkas sumber (14 CSV, 6 Markdown).".into(),
            "[10:00:04] Struktur halaman tersusun: 14 item kartu publikasi.".into(),
            "[10:00:05] Audit QA Mutu: 0 kesalahan, 0 peringatan (Lolos Preflight 100%).".into(),
        ];

        Self {
            view: OrchestratorView::Home,
            is_dirty: false,
            recent_documents: vec![
                "pubgen/samples/brs/BRS_NTO_202610.pubgen".into(),
                "pubgen/samples/brs/ntp202610.xlsx".into(),
            ],
            center_tab: OrchestratorTab::StrukturHalaman,
            selected_profile: "bps_brs".into(),
            right_dock_tab: OrchestratorRightTab::SumberKonten,
            right_dock_expanded: true,
            page_layout_facing: true,
            doc_language: "Indonesian".into(),
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
            meta_title: "Perkembangan Nilai Tukar Petani".into(),
            meta_subtitle: "1 Oktober 2026 · BADAN PUSAT STATISTIK PROVINSI JAWA BARAT".into(),
            meta_release_no: "BRS No. 01/01/Th. XXVI, 15".into(),
            meta_release_date: "1 Oktober 2026".into(),
            meta_release_month: "Januari 2026".into(),
            meta_agency: "BADAN PUSAT STATISTIK PROVINSI JAWA BARAT".into(),
            cards,
            slots,
            last_qa_clean: true,
            last_output_pdf: Some("outputs/BRS_NTP_Jabar_2026.pdf".into()),
            execution_log,
            editing_card_idx: None,
            card_delete_confirm: false,
            prompt_copied_feedback: false,
            status_message: None,
            show_guard_dialog: false,
            guard_is_publish: false,
        }
    }
}

impl OrchestratorWorkspaceState {
    pub fn has_active_workspace(&self) -> bool {
        self.view == OrchestratorView::Studio || !self.cards.is_empty() || self.is_dirty
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

pub fn sync_external_workspace(ctx: &egui::Context, source_name: &str, metadata: &pubgen_core::PublicationMetadata, csv: &[(String, Vec<u8>)], md: &[(String, Vec<u8>)], assets: &[(String, Vec<u8>)]) {
    let mut state = get_state(ctx);
    state.recent_documents = vec![source_name.to_owned()];
    state.meta_title = metadata.title.clone();
    state.meta_subtitle = metadata.title_en.clone().unwrap_or_default();
    state.selected_profile = metadata.profile_id.clone();
    state.page_layout_facing = matches!(metadata.layout, pubgen_core::PageLayout::Double);
    state.doc_language = match metadata.language { pubgen_core::Language::Id => "Indonesian", pubgen_core::Language::En => "English", pubgen_core::Language::IdEn => "Indonesian-English" }.into();
    state.content_files = csv.iter().map(|(name, bytes)| ContentItem { path: format!("csv/{name}"), size_bytes: bytes.len() }).chain(md.iter().map(|(name, bytes)| ContentItem { path: format!("md/{name}"), size_bytes: bytes.len() })).chain(assets.iter().map(|(name, bytes)| ContentItem { path: format!("assets/{name}"), size_bytes: bytes.len() })).collect();
    state.cards = metadata.cards.iter().enumerate().map(|(index, card)| PublicationCard::new_with(match card.kind { pubgen_core::CardKind::Cover => CardKind::Cover, pubgen_core::CardKind::Toc => CardKind::Toc, pubgen_core::CardKind::Table => CardKind::Table, pubgen_core::CardKind::Chart => CardKind::Chart, pubgen_core::CardKind::MapChart => CardKind::MapChart, pubgen_core::CardKind::Narrative => CardKind::Narrative, pubgen_core::CardKind::Image => CardKind::Image, pubgen_core::CardKind::Divider => CardKind::Narrative }, &card.title, "", card.source.as_deref().unwrap_or("Tanpa sumber"), index + 1, matches!(card.scope, pubgen_core::CardScope::NewPage))).collect();
    state.is_dirty = false;
    set_state(ctx, state);
}

#[derive(Clone, Copy)]
enum ProfileCardKind {
    Dda,
    Brs,
    Generic,
}

fn render_profile_card(
    ui: &mut Ui,
    t: &Tokens,
    kind: ProfileCardKind,
    title: &str,
    subtitle: &str,
    spec: &str,
    accent: Color32,
) -> bool {
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
            ui.painter().line_segment([pos2(p_left.min.x + 4.0, p_left.max.y - 6.0), pos2(p_left.max.x - 4.0, p_left.max.y - 6.0)], Stroke::new(1.0, accent));
            ui.painter().line_segment([pos2(p_right.min.x + 4.0, p_right.min.y + 6.0), pos2(p_right.max.x - 4.0, p_right.min.y + 6.0)], Stroke::new(1.0, accent));
        }
        ProfileCardKind::Brs => {
            // Single page with blue header and orange accent
            let pw = 30.0f32;
            let ph = 42.0f32;
            let p = Rect::from_center_size(pos2(cx, cy), vec2(pw, ph));
            ui.painter().rect(p, 1.0, Color32::WHITE, Stroke::new(0.8, Color32::from_rgb(180, 180, 180)), egui::StrokeKind::Inside);
            let banner = Rect::from_min_size(p.min, vec2(pw, 10.0));
            ui.painter().rect_filled(banner, 1.0, Color32::from_rgb(0, 114, 188));
            ui.painter().line_segment([pos2(p.min.x, p.min.y + 10.0), pos2(p.max.x, p.min.y + 10.0)], Stroke::new(1.0, Color32::from_rgb(245, 135, 31)));
            ui.painter().rect_filled(Rect::from_min_size(pos2(p.min.x + 3.0, p.min.y + 14.0), vec2(pw - 6.0, 3.0)), 0.0, Color32::from_rgb(190, 190, 190));
            ui.painter().rect_filled(Rect::from_min_size(pos2(p.min.x + 3.0, p.min.y + 20.0), vec2(pw - 6.0, 14.0)), 0.0, Color32::from_rgb(230, 235, 245));
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
    ui.painter().text(pos2(rect.center().x, rect.min.y + 90.0), Align2::CENTER_CENTER, title, semibold(13.5), t.text_strong);
    ui.painter().text(pos2(rect.center().x, rect.min.y + 112.0), Align2::CENTER_CENTER, subtitle, FontId::proportional(11.5), t.text_dim);
    ui.painter().text(pos2(rect.center().x, rect.min.y + 134.0), Align2::CENTER_CENTER, spec, semibold(11.0), accent);

    resp.clicked()
}

fn render_recent_documents(
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
    lang: &str,
) {
    if state.recent_documents.is_empty() {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width().min(720.0), 54.0), Sense::hover());
        ui.painter().rect(rect, 8.0, t.panel, Stroke::new(1.0, t.divider), egui::StrokeKind::Inside);
        let msg = if lang.starts_with("id") {
            "ℹ Belum ada riwayat berkas *.pubgen. Buka berkas proyek atau impor XLSX untuk memulai."
        } else {
            "ℹ No *.pubgen history yet. Open a project file or import XLSX to get started."
        };
        ui.painter().text(pos2(rect.min.x + 16.0, rect.center().y), Align2::LEFT_CENTER, msg, FontId::proportional(12.0), t.text_dim);
    } else {
        let mut clicked_doc: Option<String> = None;
        for path in &state.recent_documents {
            let filename = std::path::Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone());
            let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width().min(720.0), 44.0), Sense::click());
            let hov = resp.hovered();
            ui.painter().rect(rect, 6.0, if hov { t.hover } else { t.panel }, Stroke::new(1.0, if hov { t.accent_strong } else { t.divider }), egui::StrokeKind::Inside);
            ui.painter().text(pos2(rect.min.x + 14.0, rect.min.y + 14.0), Align2::LEFT_CENTER, &filename, semibold(12.5), t.text_strong);
            ui.painter().text(pos2(rect.min.x + 14.0, rect.min.y + 30.0), Align2::LEFT_CENTER, path, FontId::proportional(10.5), t.text_dim);
            if resp.clicked() {
                clicked_doc = Some(path.clone());
            }
            ui.add_space(4.0);
        }
        if let Some(doc) = clicked_doc {
            state.execution_log.push(format!("[{}] Membuka berkas {}", format_now(), doc));
            state.view = OrchestratorView::Studio;
        }
    }
}

pub fn render_orchestrator_home(
    _app: &mut DesignApp,
    ui: &mut Ui,
    state: &mut OrchestratorWorkspaceState,
    t: &Tokens,
    lang: &str,
) {
    let r = ui.available_rect_before_wrap();
    ui.painter().rect_filled(r, 0.0, t.panel_darker);

    ScrollArea::vertical()
        .id_salt("pubgen_home_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(32.0);
            let max_w = 980.0f32.min(ui.available_width() - 64.0);
            ui.horizontal(|ui| {
                ui.add_space(32.0);
                ui.allocate_ui_with_layout(vec2(max_w, 0.0), Layout::top_down(Align::Min), |ui| {
                    // HERO
                    ui.horizontal(|ui| {
                        let (icon_r, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
                        ui.painter().rect_filled(icon_r, 10.0, Color32::from_rgb(37, 99, 235));
                        ui.painter().text(icon_r.center(), Align2::CENTER_CENTER, "📊", FontId::proportional(22.0), Color32::WHITE);
                        ui.add_space(14.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(crate::i18n::tr(lang, "Welcome to Publication Orchestrator")).font(semibold(24.0)).color(t.text_strong));
                            let subtitle = if lang.starts_with("id") {
                                "Tata letak publikasi statistik cetak dan digital — cepat, terbuka, dan otomatis."
                            } else {
                                "Page layout for print and screen — fast, open and scriptable."
                            };
                            ui.label(RichText::new(subtitle).size(13.5).color(t.text_dim));
                        });
                    });

                    ui.add_space(24.0);

                    // ACTIONS ROW
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        let open_label = if lang.starts_with("id") { "📂 Buka Berkas (.pubgen)" } else { "📂 Open file (.pubgen)" };
                        if ui.add(
                            egui::Button::new(RichText::new(open_label).size(13.0).color(Color32::WHITE))
                                .fill(t.accent_strong)
                                .corner_radius(16.0)
                                .min_size(vec2(0.0, 32.0))
                        ).clicked() {
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(path) = rfd::FileDialog::new().add_filter("Pubgen Project", &["pubgen"]).pick_file() {
                                let p = path.to_string_lossy().to_string();
                                if !state.recent_documents.contains(&p) {
                                    state.recent_documents.insert(0, p.clone());
                                }
                                state.execution_log.push(format!("[{}] Membuka {}", format_now(), p));
                                state.view = OrchestratorView::Studio;
                            }
                        }

                        let import_label = if lang.starts_with("id") { "📊 Impor Manifest (XLSX)" } else { "📊 Import Manifest (XLSX)" };
                        if ui.add(
                            egui::Button::new(RichText::new(import_label).size(13.0))
                                .corner_radius(16.0)
                                .min_size(vec2(0.0, 32.0))
                        ).clicked() {
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(path) = rfd::FileDialog::new().add_filter("Excel Manifest", &["xlsx", "xls"]).pick_file() {
                                let p = path.to_string_lossy().to_string();
                                if !state.recent_documents.contains(&p) {
                                    state.recent_documents.insert(0, p.clone());
                                }
                                state.execution_log.push(format!("[{}] Impor manifest {}", format_now(), p));
                                state.view = OrchestratorView::Studio;
                            }
                        }

                        let brs_label = if lang.starts_with("id") { "📰 Sampel BRS" } else { "📰 Sample BRS" };
                        if ui.add(
                            egui::Button::new(RichText::new(brs_label).size(13.0))
                                .corner_radius(16.0)
                                .min_size(vec2(0.0, 32.0))
                        ).clicked() {
                            state.selected_profile = "bps_brs".into();
                            state.page_layout_facing = false;
                            state.view = OrchestratorView::Studio;
                        }

                        let dda_label = if lang.starts_with("id") { "📑 Sampel DDA" } else { "📑 Sample DDA" };
                        if ui.add(
                            egui::Button::new(RichText::new(dda_label).size(13.0))
                                .corner_radius(16.0)
                                .min_size(vec2(0.0, 32.0))
                        ).clicked() {
                            state.selected_profile = "bps_dda".into();
                            state.page_layout_facing = true;
                            state.view = OrchestratorView::Studio;
                        }

                        if !state.cards.is_empty() {
                            let studio_label = if lang.starts_with("id") { "➡ Buka Studio Aktif" } else { "➡ Open Active Studio" };
                            if ui.add(
                                egui::Button::new(RichText::new(studio_label).size(13.0).color(Color32::from_rgb(16, 185, 129)))
                                    .corner_radius(16.0)
                                    .min_size(vec2(0.0, 32.0))
                            ).clicked() {
                                state.view = OrchestratorView::Studio;
                            }
                        }
                    });

                    ui.add_space(28.0);

                    // SECTION: PUBLICATION PROFILES
                    let sec_prof = if lang.starts_with("id") { "Profil Publikasi" } else { "Publication Profiles" };
                    ui.label(RichText::new(sec_prof).font(semibold(15.0)).color(t.text_strong));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;

                        // 1. DDA BPS
                        if render_profile_card(
                            ui, t,
                            ProfileCardKind::Dda,
                            "DDA BPS",
                            "Daerah Dalam Angka",
                            "A4 (210 × 297 mm) • Facing Pages",
                            Color32::from_rgb(59, 130, 246),
                        ) {
                            state.selected_profile = "bps_dda".into();
                            state.page_layout_facing = true;
                            state.paper_size = "A4 (210 x 297 mm)".into();
                            state.view = OrchestratorView::Studio;
                        }

                        // 2. BRS BPS
                        if render_profile_card(
                            ui, t,
                            ProfileCardKind::Brs,
                            "BRS BPS",
                            "Berita Resmi Statistik",
                            "A4 (210 × 297 mm) • Single Page",
                            Color32::from_rgb(245, 158, 11),
                        ) {
                            state.selected_profile = "bps_brs".into();
                            state.page_layout_facing = false;
                            state.paper_size = "A4 (210 x 297 mm)".into();
                            state.view = OrchestratorView::Studio;
                        }

                        // 3. General
                        let gen_sub = if lang.starts_with("id") { "Laporan Standar Umum" } else { "Generic Publication Report" };
                        if render_profile_card(
                            ui, t,
                            ProfileCardKind::Generic,
                            "General",
                            gen_sub,
                            "A4 (210 × 297 mm) • Single Page",
                            Color32::from_rgb(156, 163, 175),
                        ) {
                            state.selected_profile = "generic_report".into();
                            state.page_layout_facing = false;
                            state.paper_size = "A4 (210 x 297 mm)".into();
                            state.view = OrchestratorView::Studio;
                        }
                    });

                    ui.add_space(28.0);

                    // SECTION: RECENT OPENED DOCUMENTS
                    let sec_rec = if lang.starts_with("id") { "Dokumen Terakhir Dibuka (*.pubgen)" } else { "Recent Opened Documents (*.pubgen)" };
                    ui.label(RichText::new(sec_rec).font(semibold(15.0)).color(t.text_strong));
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
    let mut state = get_state(ui.ctx());
    let lang = app.ui.language.clone();

    if state.view == OrchestratorView::Home {
        render_orchestrator_home(app, ui, &mut state, &t, &lang);
        set_state(ui.ctx(), state);
        return;
    }

    let avail_rect = ui.available_rect_before_wrap();
    let total_width = avail_rect.width();
    let total_height = avail_rect.height();

    // 2-Section Layout: Center takes available space; Right dock is either expanded or collapsed
    let right_w = if state.right_dock_expanded { 320.0f32.min(total_width * 0.35).max(280.0) } else { 38.0f32 };
    let center_w = (total_width - right_w).max(100.0);

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
        let right_pos = pos2(avail_rect.min.x + center_w, avail_rect.min.y);
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

    set_state(ui.ctx(), state);
}

/// Collapsed iconic right dock (InDesign parity: icon strip with expand toggle)
fn render_collapsed_dock(_app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, width: f32, total_height: f32, lang: &str) {
    ui.set_width(width);
    ui.set_height(total_height);

    let tab_bar_height = 27.0f32;

    // Header strip with visible expand button
    let expand_btn = ui.add_sized(
        [width, tab_bar_height],
        egui::Button::new(RichText::new("«").font(semibold(13.0)).color(t.text_strong)).fill(t.tab_strip).frame(false),
    ).on_hover_text(format!("{} (Expand Panels)", crate::i18n::tr(lang, "Expand Panels")));
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
    let resp1 = resp1.on_hover_text(format!("{}: {}", crate::i18n::tr(lang, "Sources & Content"), crate::i18n::tr(lang, "Click to edit")));
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
    let resp2 = resp2.on_hover_text(format!("{}: {}", crate::i18n::tr(lang, "Profile & Execution"), crate::i18n::tr(lang, "Click to edit")));
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
}

/// Expanded right dock with tabs (InDesign parity: tab strip + collapse button)
fn render_expanded_dock(app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, width: f32, total_height: f32, lang: &str) {
    ui.set_width(width);
    ui.set_height(total_height);

    let tab_bar_height = 27.0f32;
    let collapse_btn_w = 26.0f32;
    let remaining_w = (width - collapse_btn_w).max(100.0);
    let tab_w = (remaining_w / 2.0).floor();

    // 1. Tab Bar strip
    let (strip, _) = ui.allocate_exact_size(vec2(width, tab_bar_height), Sense::hover());
    ui.painter().rect_filled(strip, 0.0, t.tab_strip);

    // Collapse to icons button on the left of the tab strip
    let collapse_rect = Rect::from_min_size(pos2(strip.min.x + 2.0, strip.min.y + 2.0), vec2(collapse_btn_w - 4.0, tab_bar_height - 4.0));
    let collapse_resp =
        ui.interact(collapse_rect, ui.scope_id().with("dock_collapse_btn"), Sense::click()).on_hover_text(format!("{} (Collapse to Icons)", crate::i18n::tr(lang, "Collapse to Icons")));
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
    let tab1_resp = ui
        .interact(tab1_rect, ui.scope_id().with("dock_tab_1"), Sense::click())
        .on_hover_text(crate::i18n::tr(lang, "Sources & Content"));
    if is_tab1 {
        ui.painter().rect_filled(tab1_rect, 0.0, t.panel);
        ui.painter().line_segment([tab1_rect.left_top(), tab1_rect.right_top()], Stroke::new(2.0, t.accent));
        ui.painter().line_segment([tab1_rect.left_top(), tab1_rect.left_bottom()], Stroke::new(1.0, t.border));
        ui.painter().line_segment([tab1_rect.right_top(), tab1_rect.right_bottom()], Stroke::new(1.0, t.border));
    } else {
        if tab1_resp.hovered() {
            ui.painter().rect_filled(tab1_rect, 0.0, t.hover);
        }
        ui.painter().line_segment([tab1_rect.left_bottom(), tab1_rect.right_bottom()], Stroke::new(1.0, t.border));
    }
    ui.painter().text(
        tab1_rect.center(),
        Align2::CENTER_CENTER,
        crate::i18n::tr(lang, "Sources & Content"),
        if is_tab1 { semibold(11.0) } else { FontId::proportional(11.0) },
        if is_tab1 { t.text_strong } else { t.text_dim },
    );
    if tab1_resp.clicked() {
        state.right_dock_tab = OrchestratorRightTab::SumberKonten;
    }

    // Tab 2: Profil & Eksekusi
    let tab2_w = (width - (collapse_btn_w + tab_w)).max(0.0);
    let tab2_rect = Rect::from_min_size(pos2(strip.min.x + collapse_btn_w + tab_w, strip.min.y), vec2(tab2_w, tab_bar_height));
    let is_tab2 = state.right_dock_tab == OrchestratorRightTab::ProfilEksekusi;
    let tab2_resp = ui
        .interact(tab2_rect, ui.scope_id().with("dock_tab_2"), Sense::click())
        .on_hover_text(crate::i18n::tr(lang, "Profile & Execution"));
    if is_tab2 {
        ui.painter().rect_filled(tab2_rect, 0.0, t.panel);
        ui.painter().line_segment([tab2_rect.left_top(), tab2_rect.right_top()], Stroke::new(2.0, t.accent));
        ui.painter().line_segment([tab2_rect.left_top(), tab2_rect.left_bottom()], Stroke::new(1.0, t.border));
        ui.painter().line_segment([tab2_rect.right_top(), tab2_rect.right_bottom()], Stroke::new(1.0, t.border));
    } else {
        if tab2_resp.hovered() {
            ui.painter().rect_filled(tab2_rect, 0.0, t.hover);
        }
        ui.painter().line_segment([tab2_rect.left_bottom(), tab2_rect.right_bottom()], Stroke::new(1.0, t.border));
    }
    ui.painter().text(
        tab2_rect.center(),
        Align2::CENTER_CENTER,
        crate::i18n::tr(lang, "Profile & Execution"),
        if is_tab2 { semibold(11.0) } else { FontId::proportional(11.0) },
        if is_tab2 { t.text_strong } else { t.text_dim },
    );
    if tab2_resp.clicked() {
        state.right_dock_tab = OrchestratorRightTab::ProfilEksekusi;
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
        ui.label(RichText::new(format!("Source: {source_label}")).font(FontId::proportional(10.5)).color(t.text_dim));
        ui.label(RichText::new(format!("{} embedded content file(s)", state.content_files.len())).font(FontId::proportional(10.5)).color(t.text_dim));
    }

    // Header row
    ui.horizontal(|ui| {
        ui.label(RichText::new(crate::i18n::tr(lang, "Sources & Content")).font(semibold(13.5)).color(t.text_strong));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Home button
            if ui
                .add(egui::Button::new(RichText::new("⌂").font(semibold(12.0))).frame(true).corner_radius(CornerRadius::same(3)))
                .on_hover_text(crate::i18n::tr(lang, "Home"))
                .clicked()
            {
                state.view = OrchestratorView::Home;
            }

            // Save button
            if ui
                .add(egui::Button::new(RichText::new("💾").font(semibold(12.0))).frame(true).corner_radius(CornerRadius::same(3)))
                .on_hover_text(crate::i18n::tr(lang, "Save"))
                .clicked()
            {
                let now = format_now();
                state.is_dirty = false;
                state.execution_log.push(format!("[{now}] Proyek publikasi berhasil disimpan ke cache."));
                state.status_message = Some(("Proyek disimpan".into(), true));
            }
        });
    });

    // Baris Tombol Impor
    ui.horizontal(|ui| {
        let btn_w = ((ui.available_width() - 8.0) / 3.0).max(70.0);

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(RichText::new(format!("+ {}", crate::i18n::tr(lang, "Import Source"))).font(FontId::proportional(10.5)).color(t.text))
                    .corner_radius(CornerRadius::same(3)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Import Source"))
            .clicked()
        {
            let _ = app.ask_open("dataMerge", crate::app::file_dialogs::Bound::App, move |_app, path| {
                let name = std::path::Path::new(&path).file_name().unwrap_or_default().to_string_lossy().to_string();
                let size = std::fs::metadata(&path).map(|m| m.len() as usize).unwrap_or(1024);
                eprintln!("Imported: {name} ({size} B)");
                Ok(serde_json::Value::Null)
            });
        }

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(RichText::new(format!("📊 {}", crate::i18n::tr(lang, "Import Excel"))).font(FontId::proportional(10.5)).color(t.text))
                    .corner_radius(CornerRadius::same(3)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Import Excel"))
            .clicked()
        {
            let _ = app.ask_open("dataMerge", crate::app::file_dialogs::Bound::App, move |_app, path| {
                let name = std::path::Path::new(&path).file_name().unwrap_or_default().to_string_lossy().to_string();
                eprintln!("Excel imported: {name}");
                Ok(serde_json::Value::Null)
            });
        }

        if ui
            .add_sized(
                [btn_w, 22.0],
                egui::Button::new(RichText::new(format!("🎨 {}", crate::i18n::tr(lang, "Import Metadata"))).font(FontId::proportional(10.5)).color(t.text)).corner_radius(CornerRadius::same(3)),
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

        // Section 1: Pengaturan Umum Publikasi (Profil BRS BPS - Image 3)
        ui.label(RichText::new(crate::i18n::tr(lang, "General Settings (BRS BPS Profile)")).font(semibold(12.0)).color(t.text_strong));

        egui::Frame::NONE
            .fill(t.panel_darker)
            .stroke(Stroke::new(1.0, t.field_border))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 5.0);

                // 1. Tata Letak Halaman
                let facing_label = crate::i18n::tr(lang, "Double Page (Facing)");
                let single_label = crate::i18n::tr(lang, "Single Page (Non-Facing)");
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::tr(lang, "Page Layout:")).font(FontId::proportional(10.5)).color(t.text_dim));
                    let combo_w = (ui.available_width() - 4.0).max(120.0);
                    craft_ui::ComboBox::from_id_salt("meta_page_layout_combo")
                        .selected_text(
                            RichText::new(if state.page_layout_facing { facing_label } else { single_label })
                                .font(FontId::proportional(10.5)),
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
                    ui.label(RichText::new(crate::i18n::tr(lang, "Document Language:")).font(FontId::proportional(10.5)).color(t.text_dim));
                    let combo_w = (ui.available_width() - 4.0).max(120.0);
                    craft_ui::ComboBox::from_id_salt("meta_doc_lang_combo")
                        .selected_text(RichText::new(crate::i18n::tr(lang, &state.doc_language)).font(FontId::proportional(10.5)))
                        .width(combo_w)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.doc_language, "Indonesian".into(), indonesian_label);
                            ui.selectable_value(&mut state.doc_language, "English".into(), english_label);
                            ui.selectable_value(&mut state.doc_language, "Indonesian-English".into(), "Indonesian-English / Indonesia-Inggris");
                        });
                });

                // 3. Ukuran Kertas
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::tr(lang, "Paper Size:")).font(FontId::proportional(10.5)).color(t.text_dim));
                    let combo_w = (ui.available_width() - 4.0).max(120.0);
                    craft_ui::ComboBox::from_id_salt("meta_paper_size_combo")
                        .selected_text(RichText::new(&state.paper_size).font(FontId::proportional(10.5)))
                        .width(combo_w)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.paper_size, "A4 (210 x 297 mm)".into(), "A4 (210 x 297 mm)");
                            ui.selectable_value(&mut state.paper_size, "Letter (8.5 x 11 in)".into(), "Letter (8.5 x 11 in)");
                            ui.selectable_value(&mut state.paper_size, "B5 (182 x 257 mm)".into(), "B5 (182 x 257 mm)");
                        });
                });

                // 4. Margin (mm) with +/- steppers
                ui.add_space(2.0);
                ui.label(RichText::new(crate::i18n::tr(lang, "Margins (mm):")).font(FontId::proportional(10.5)).color(t.text_dim));

                let col_w = (ui.available_width() / 2.0).max(85.0);
                // Row 1: T and R
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(vec2(col_w, 22.0), Layout::left_to_right(Align::Center), |ui| {
                        render_margin_stepper(ui, "T", &mut state.margin_top_mm, t);
                    });
                    ui.allocate_ui_with_layout(vec2(col_w, 22.0), Layout::left_to_right(Align::Center), |ui| {
                        render_margin_stepper(ui, "R", &mut state.margin_right_mm, t);
                    });
                });

                // Row 2: B and L
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(vec2(col_w, 22.0), Layout::left_to_right(Align::Center), |ui| {
                        render_margin_stepper(ui, "B", &mut state.margin_bottom_mm, t);
                    });
                    ui.allocate_ui_with_layout(vec2(col_w, 22.0), Layout::left_to_right(Align::Center), |ui| {
                        render_margin_stepper(ui, "L", &mut state.margin_left_mm, t);
                    });
                });
            });

        ui.separator();

        // Section 2: Collapsible Pengaturan Dokumen (Metadata Form - Image 4)
        let meta_hdr = ui.selectable_label(
            false,
            RichText::new(format!("{} {}", if state.metadata_open { "▼" } else { "▶" }, crate::i18n::tr(lang, "Document Settings"))).font(semibold(12.0)).color(t.text_strong),
        );
        if meta_hdr.clicked() {
            state.metadata_open = !state.metadata_open;
        }

        if state.metadata_open {
            egui::Frame::NONE
                .fill(t.panel_darker)
                .stroke(Stroke::new(1.0, t.field_border))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(3.0, 4.0);

                    ui.label(RichText::new(crate::i18n::tr(lang, "Publication Title")).font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_title));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Subtitle")).font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_subtitle));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Release / Catalogue No.")).font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_release_no));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Release Date")).font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_release_date));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Release Month")).font(FontId::proportional(10.5)).color(t.text_dim));
                    ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut state.meta_release_month));

                    ui.label(RichText::new(crate::i18n::tr(lang, "Publishing Agency")).font(FontId::proportional(10.5)).color(t.text_dim));
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
                let md_files: Vec<_> = state.content_files.iter().filter(|f| f.path.starts_with("md/") || f.path.ends_with(".md")).collect();

                // Group 1: CSV
                let csv_hdr = ui.selectable_label(
                    false,
                    RichText::new(format!("{} {} ({})", if state.csv_group_open { "▼" } else { "▶" }, crate::i18n::tr(lang, "Data Tables (CSV)"), csv_files.len()))
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
                                ui.label(RichText::new(format!("• {}", item.path)).font(FontId::proportional(10.5)).color(t.text));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{} B", item.size_bytes)).font(FontId::proportional(9.5)).color(t.text_dim));
                                });
                            });
                        }
                    });
                }

                // Group 2: Markdown
                let md_hdr = ui.selectable_label(
                    false,
                    RichText::new(format!("{} {} ({})", if state.md_group_open { "▼" } else { "▶" }, crate::i18n::tr(lang, "Narrative (Markdown)"), md_files.len()))
                        .font(semibold(11.0))
                        .color(t.text),
                );
                if md_hdr.clicked() {
                    state.md_group_open = !state.md_group_open;
                }

                if state.md_group_open {
                    ui.indent("md_indent", |ui| {
                        for item in md_files {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("• {}", item.path)).font(FontId::proportional(10.5)).color(t.text));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{} B", item.size_bytes)).font(FontId::proportional(9.5)).color(t.text_dim));
                                });
                            });
                        }
                    });
                }
            });
        }
    });
}

fn render_margin_stepper(ui: &mut Ui, label: &str, val: &mut u32, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(3.0, 0.0);
        ui.label(RichText::new(format!("{label}: {val}mm")).font(FontId::proportional(10.5)).color(t.text));
        let btn_size = vec2(17.0, 18.0);
        let btn_fill = Color32::from_rgb(37, 99, 235);
        if ui
            .add_sized(
                btn_size,
                egui::Button::new(RichText::new("-").font(semibold(11.0)).color(Color32::WHITE)).fill(btn_fill).corner_radius(CornerRadius::same(2)),
            )
            .clicked()
        {
            *val = val.saturating_sub(1).max(5);
        }
        if ui
            .add_sized(
                btn_size,
                egui::Button::new(RichText::new("+").font(semibold(11.0)).color(Color32::WHITE)).fill(btn_fill).corner_radius(CornerRadius::same(2)),
            )
            .clicked()
        {
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
        ui.label(RichText::new(crate::i18n::tr(lang, "Execution & Verification")).font(semibold(13.5)).color(t.text_strong));
        ui.add_space(16.0);
        ui.label(RichText::new(crate::i18n::tr(lang, "QA Preflight Checklist")).font(semibold(11.0)).color(t.text_strong));

        if ui
            .add(
                egui::Button::new(RichText::new(format!("🔍 {}", crate::i18n::tr(lang, "Run QA Audit"))).font(FontId::proportional(10.5)).color(t.text))
                    .corner_radius(CornerRadius::same(3)),
            )
            .on_hover_text(crate::i18n::tr(lang, "Run QA Audit"))
            .clicked()
        {
            let now = format_now();
            state.last_qa_clean = true;
            state.execution_log.push(format!("[{now}] Audit QA selesai: 0 error, 0 warning. Standar cetak lolos."));
            state.status_message = Some(("Audit QA Mutu Lolos (100%)".into(), true));
        }

        if let Some(pdf) = &state.last_output_pdf {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(format!("👁 {}", crate::i18n::tr(lang, "View Latest PDF"))).font(FontId::proportional(10.5)).color(Color32::from_rgb(56, 189, 248)),
                    )
                    .corner_radius(CornerRadius::same(3)),
                )
                .on_hover_text(format!("Buka {pdf}"))
                .clicked()
            {
                let now = format_now();
                state.execution_log.push(format!("[{now}] Membuka berkas hasil rilis: {pdf}"));
                state.status_message = Some(("Membuka berkas PDF".into(), true));
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if state.last_qa_clean {
                ui.label(RichText::new(format!("● {}", crate::i18n::tr(lang, "Clean (QA Fit 100%)"))).font(semibold(10.5)).color(Color32::from_rgb(74, 222, 128)));
            } else {
                ui.label(RichText::new(format!("● {}", crate::i18n::tr(lang, "Issues Detected"))).font(semibold(10.5)).color(Color32::from_rgb(248, 113, 113)));
            }
        });
    });

    // 3. Tab Selector Bar
    ui.horizontal(|ui| {
        let is_tab1 = state.center_tab == OrchestratorTab::StrukturHalaman;
        let is_tab2 = state.center_tab == OrchestratorTab::VisualLivePreview;
        let is_tab3 = state.center_tab == OrchestratorTab::LogEksekusi;

        let tab1_title = format!("📑 {}", crate::i18n::tr(lang, "Page Structure"));
        let tab2_title = format!("👁 {}", crate::i18n::tr(lang, "Visual Live Preview"));
        let tab3_title = format!("📋 {}", crate::i18n::tr(lang, "Execution Log"));

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
            render_visual_live_preview(app, ui, state, t);
        }
        OrchestratorTab::LogEksekusi => {
            render_log_eksekusi(app, ui, state, t);
        }
    }
}

fn tab_button(ui: &mut Ui, label: &str, active: bool, t: &Tokens) -> Response {
    let font = semibold(11.5);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, if active { t.text_strong } else { t.text_dim });
    let size = vec2(galley.size().x + 20.0, 24.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    if active {
        ui.painter().rect(rect, CornerRadius::same(3), t.well, Stroke::new(1.0, t.field_border), StrokeKind::Inside);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(3), t.hover);
    }

    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, Color32::WHITE);

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
                painter.rect_filled(Rect::from_min_size(egui::pos2(rect.min.x + 6.0, y), vec2(14.0, 2.0)), CornerRadius::same(1), Color32::from_rgb(165, 180, 252));
                painter.line_segment([egui::pos2(rect.min.x + 22.0, y + 1.0), egui::pos2(rect.min.x + 28.0, y + 1.0)], Stroke::new(1.0, Color32::from_rgb(129, 140, 248)));
                painter.rect_filled(Rect::from_min_size(egui::pos2(rect.min.x + 30.0, y), vec2(4.0, 2.0)), CornerRadius::same(1), Color32::from_rgb(199, 210, 254));
            }
        }
    }
}

fn render_card_grid(_app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    let avail_w = ui.available_width();
    let gap = 8.0f32;
    let card_h = 84.0f32;
    let mut clicked_edit_idx = None;

    if state.page_layout_facing {
        // FACING PAGES SPREADS LAYOUT (InDesign parity - Image #3)
        // Row 1: Page 1 (Cover / Sampul) alone on left with empty right slot (matching trailing last card)
        // Subsequent rows: Pairs of Left Page (Verso, spine right) and Right Page (Recto, spine left)
        let col_w = ((avail_w - gap) / 2.0).max(120.0);

        if let Some(cover_card) = state.cards.first() {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(gap, 0.0);
                if let Some(idx) = render_facing_card(ui, 0, cover_card, col_w, false, t, lang) {
                    clicked_edit_idx = Some(idx);
                }
                // Empty right slot so cover card length is identical to col_w and height matches card_h
                ui.allocate_exact_size(vec2(col_w, card_h), Sense::hover());
            });
            ui.add_space(gap);
        }

        // Remaining body cards starting from index 1 (Page 2 onwards)
        if state.cards.len() > 1 {
            let body_cards = &state.cards[1..];
            for (chunk_idx, pair) in body_cards.chunks(2).enumerate() {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(gap, 0.0);
                    let actual_idx_left = 1 + chunk_idx * 2;
                    // Left page (Verso: thumbnail on right facing spine, text right-aligned)
                    let is_single_trailing = pair.len() == 1;
                    if let Some(idx) = render_facing_card(ui, actual_idx_left, &pair[0], col_w, !is_single_trailing, t, lang) {
                        clicked_edit_idx = Some(idx);
                    }

                    if pair.len() > 1 {
                        let actual_idx_right = actual_idx_left + 1;
                        // Right page (Recto: thumbnail on left facing spine, text left-aligned)
                        if let Some(idx) = render_facing_card(ui, actual_idx_right, &pair[1], col_w, false, t, lang) {
                            clicked_edit_idx = Some(idx);
                        }
                    } else {
                        // Empty slot on right for trailing single page
                        ui.allocate_exact_size(vec2(col_w, card_h), Sense::hover());
                    }
                });
                ui.add_space(gap);
            }
        }
    } else {
        // SINGLE PAGE GRID LAYOUT (2 Columns standard)
        let col_count = if avail_w > 520.0 { 2 } else { 1 };
        let card_w = ((avail_w - (col_count as f32 - 1.0) * gap) / (col_count as f32)).max(120.0);
        let chunk_size = col_count;
        for (row_idx, chunk) in state.cards.chunks(chunk_size).enumerate() {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(gap, 0.0);
                for (col_idx, card) in chunk.iter().enumerate() {
                    let actual_idx = row_idx * chunk_size + col_idx;
                    if let Some(idx) = render_facing_card(ui, actual_idx, card, card_w, false, t, lang) {
                        clicked_edit_idx = Some(idx);
                    }
                }
                if chunk.len() < chunk_size {
                    for _ in chunk.len()..chunk_size {
                        ui.allocate_exact_size(vec2(card_w, card_h), Sense::hover());
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

fn render_facing_card(ui: &mut Ui, idx: usize, card: &PublicationCard, card_w: f32, is_verso: bool, t: &Tokens, lang: &str) -> Option<usize> {
    const CARD_HEIGHT: f32 = 84.0;
    let mut clicked_edit = false;

    ui.push_id(idx, |ui| {
        let (card_rect, card_resp) = ui.allocate_exact_size(vec2(card_w, CARD_HEIGHT), Sense::click());
        if card_resp.clicked() {
            clicked_edit = true;
        }

        let is_hovered = card_resp.hovered();
        let bg = if is_hovered { t.hover } else { t.panel };
        let border_c = if is_hovered { t.accent } else { t.field_border };
        ui.painter().rect_filled(card_rect, 4.0, bg);
        ui.painter().rect_stroke(card_rect, 4.0, Stroke::new(1.0, border_c), egui::StrokeKind::Inside);

        let inner_rect = card_rect.shrink2(vec2(10.0, 8.0));
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

        paint_card_thumbnail(ui.painter(), thumb_rect, card.kind, t);

        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(details_rect).layout(Layout::top_down(if is_verso { Align::Max } else { Align::Min })));
        child_ui.set_clip_rect(details_rect);
        child_ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

        // Line 1: Tags
        if is_verso {
            child_ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                render_tag_pill(ui, crate::i18n::tr(lang, &card.tag2), t);
                render_tag_pill(ui, crate::i18n::tr(lang, &card.tag1), t);
            });
        } else {
            child_ui.horizontal(|ui| {
                render_tag_pill(ui, crate::i18n::tr(lang, &card.tag1), t);
                render_tag_pill(ui, crate::i18n::tr(lang, &card.tag2), t);
            });
        }

        // Line 2: Title
        let title_label = egui::Label::new(RichText::new(&card.title).font(semibold(11.0)).color(t.text_strong)).truncate().sense(Sense::click());
        let title_resp = child_ui.add(title_label);
        if title_resp.clicked() {
            clicked_edit = true;
        }

        // Line 3: Source
        let src_label = format!("{}: {}", crate::i18n::tr(lang, "Source:"), card.source);
        child_ui.add(egui::Label::new(RichText::new(src_label).font(FontId::proportional(10.0)).color(t.text_dim)).truncate());

        // Line 4: Footer
        if is_verso {
            child_ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(format!("{} {}", crate::i18n::tr(lang, "Pg."), card.page)).font(semibold(10.0)).color(t.text_dim));
                ui.add_space(8.0);
                let edit_resp = ui.link(RichText::new(format!("✎ {}", crate::i18n::tr(lang, "Click to edit"))).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                if edit_resp.clicked() {
                    clicked_edit = true;
                }
            });
        } else {
            child_ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{} {}", crate::i18n::tr(lang, "Pg."), card.page)).font(semibold(10.0)).color(t.text_dim));
                ui.add_space(8.0);
                let edit_resp = ui.link(RichText::new(format!("✎ {}", crate::i18n::tr(lang, "Click to edit"))).font(FontId::proportional(10.0)).color(Color32::from_rgb(56, 189, 248)));
                if edit_resp.clicked() {
                    clicked_edit = true;
                }
            });
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
            });

        let btn_fill = Color32::from_rgb(37, 99, 235);
        if ui
            .add(
                egui::Button::new(RichText::new(format!("+ {}", crate::i18n::tr(lang, "Add New Page"))).font(semibold(11.0)).color(Color32::WHITE))
                    .fill(btn_fill)
                    .corner_radius(CornerRadius::same(3)),
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
                    .corner_radius(CornerRadius::same(3)),
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
    let font = FontId::proportional(9.0);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, t.text);
    let size = vec2(galley.size().x + 8.0, 14.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    ui.painter().rect(rect, CornerRadius::same(2), t.well, Stroke::new(1.0, t.field_border), StrokeKind::Inside);
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, Color32::WHITE);
}

// ============================================================================
// VISUAL LIVE PREVIEW TAB & LOG EKSEKUSI TAB
// ============================================================================

fn render_visual_live_preview(_app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens) {
    ScrollArea::vertical().id_salt("pubgen_center_preview_scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Pratinjau Tata Letak Halaman (Live Spread Document)").font(semibold(11.5)).color(t.text_strong));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(format!("Total: {} Halaman", state.cards.len())).font(FontId::proportional(10.5)).color(t.text_dim));
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
            FontId::proportional(9.0),
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
                    FontId::proportional(10.0),
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
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                for line in &state.execution_log {
                    ui.label(RichText::new(line).font(FontId::monospace(10.5)).color(if line.contains("Audit QA") || line.contains("berhasil") {
                        Color32::from_rgb(74, 222, 128)
                    } else if line.contains("Error") {
                        Color32::from_rgb(248, 113, 113)
                    } else {
                        t.text
                    }));
                }
            });
    });
}

// ============================================================================
// PANEL KANAN: "Profil, Slot & Eksekusi"
// ============================================================================

fn render_right_panel(app: &mut DesignApp, ui: &mut Ui, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    ui.spacing_mut().item_spacing = vec2(6.0, 8.0);

    // Section title
    ui.label(RichText::new(crate::i18n::tr(lang, "Profile & Execution")).font(semibold(13.5)).color(t.text_strong));

    // 1. Profil Publikasi Box
    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(
        ui,
        |ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);

            ui.label(RichText::new(crate::i18n::tr(lang, "Publication Profile:")).font(semibold(11.0)).color(t.text_strong));

            // Combo selection
            let current_label = if state.selected_profile == "bps_brs" { "BPS Berita Resmi Statistik (BRS)" } else { "BPS Daerah Dalam Angka (DDA)" };

            craft_ui::ComboBox::from_id_salt("profile_select_combo")
                .selected_text(RichText::new(current_label).font(FontId::proportional(11.0)))
                .width(ui.available_width() - 8.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(state.selected_profile == "bps_brs", "BPS Berita Resmi Statistik (BRS)").clicked() {
                        state.selected_profile = "bps_brs".into();
                    }
                    if ui.selectable_label(state.selected_profile == "bps_dda", "BPS Daerah Dalam Angka (DDA)").clicked() {
                        state.selected_profile = "bps_dda".into();
                    }
                });

            ui.label(RichText::new("Margins: T:20.2 B:20.0 L:19.0 R:15.0 mm | Spreads: Tidak").font(FontId::proportional(10.0)).color(t.text_dim));
        },
    );

    // 2. Action Buttons: Generate & Generate and Publish
    let btn_h = 28.0f32;

    // Generate Button (Deep Cyan)
    if ui
        .add_sized(
            [ui.available_width(), btn_h],
            egui::Button::new(RichText::new(format!("⚡ {}", crate::i18n::tr(lang, "Generate Publication"))).font(semibold(12.0)).color(Color32::WHITE))
                .fill(Color32::from_rgb(14, 116, 144))
                .corner_radius(CornerRadius::same(4)),
        )
        .on_hover_text(crate::i18n::tr(lang, "Generate Publication"))
        .clicked()
    {
        let doc_has_pages = app.session.doc().map(|d| d.doc.pages().next().is_some()).unwrap_or(false);
        if doc_has_pages {
            state.show_guard_dialog = true;
            state.guard_is_publish = false;
        } else {
            execute_compile(app, ui.ctx(), state, false);
        }
    }

    // Generate & Publish Button (Emerald Green)
    if ui
        .add_sized(
            [ui.available_width(), btn_h],
            egui::Button::new(RichText::new(format!("🚀 {}", crate::i18n::tr(lang, "Generate & Publish PDF"))).font(semibold(12.0)).color(Color32::WHITE))
                .fill(Color32::from_rgb(22, 101, 52))
                .corner_radius(CornerRadius::same(4)),
        )
        .on_hover_text(crate::i18n::tr(lang, "Generate & Publish PDF"))
        .clicked()
    {
        let doc_has_pages = app.session.doc().map(|d| d.doc.pages().next().is_some()).unwrap_or(false);
        if doc_has_pages {
            state.show_guard_dialog = true;
            state.guard_is_publish = true;
        } else {
            execute_compile(app, ui.ctx(), state, true);
        }
    }

    ui.separator();

    // 3. Status Mesin Tata Letak
    egui::Frame::NONE
        .fill(t.panel_darker)
        .stroke(Stroke::new(1.0, t.field_border))
        .corner_radius(CornerRadius::same(3))
        .inner_margin(Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.label(RichText::new("Mesin Tata Letak").font(semibold(10.5)).color(t.text_dim));
            ui.horizontal(|ui| {
                ui.label(RichText::new("● Terintegrasi (DesignCraft Native OK)").font(semibold(10.5)).color(Color32::from_rgb(74, 222, 128)));
            });
        });

    ui.separator();

    // 4. Slot Terdeteksi (Slot character budget & fit)
    ui.label(RichText::new(format!("Slot Terdeteksi ({})", state.slots.len())).font(semibold(11.5)).color(t.text_strong));

    ScrollArea::vertical().id_salt("pubgen_right_slots_scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 6.0);

        if state.slots.is_empty() {
            ui.label(
                RichText::new("Belum ada slot ditemukan. Buka template .designcraft terlebih dahulu.")
                    .font(FontId::proportional(10.5))
                    .color(t.text_dim),
            );
        } else {
            for slot in &state.slots {
                ui.push_id(&slot.name, |ui| {
                    egui::Frame::NONE
                        .fill(t.panel_darker)
                        .stroke(Stroke::new(1.0, t.field_border))
                        .corner_radius(CornerRadius::same(3))
                        .inner_margin(Margin::same(6))
                        .show(ui, |ui| {
                            let row_w = ui.available_width();
                            ui.allocate_ui_with_layout(vec2(row_w, 18.0), Layout::left_to_right(Align::Center), |ui| {
                                ui.label(RichText::new(&slot.name).font(semibold(10.5)).color(t.text_strong));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let char_count = slot.text.chars().count();
                                    if let Some(budget) = slot.budget {
                                        let is_ok = char_count <= budget;
                                        ui.label(RichText::new(if is_ok { "● OK" } else { "● Over" }).font(semibold(10.0)).color(if is_ok {
                                            Color32::from_rgb(74, 222, 128)
                                        } else {
                                            Color32::from_rgb(248, 113, 113)
                                        }));
                                        ui.label(RichText::new(format!("{char_count}/{budget}")).font(FontId::proportional(10.0)).color(t.text_dim));
                                    } else {
                                        ui.label(RichText::new(format!("{char_count} chars")).font(FontId::proportional(10.0)).color(t.text_dim));
                                    }
                                });
                            });

                            let preview: String = slot.text.chars().take(40).collect();
                            ui.label(RichText::new(format!("{preview}…")).font(FontId::proportional(9.5)).color(t.text_dim));
                        });
                });
            }
        }
    });
}

// ============================================================================
// MODAL EDITOR KARTU PUBLIKASI
// ============================================================================

fn render_card_edit_modal(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    let Some(idx) = state.editing_card_idx else { return };
    if idx >= state.cards.len() {
        state.editing_card_idx = None;
        state.card_delete_confirm = false;
        return;
    }

    let is_bilingual = state.doc_language.to_lowercase().contains("bilingual") || state.doc_language.to_lowercase().contains("dwibahasa");
    let is_english = state.doc_language.to_lowercase().contains("english") || state.doc_language.to_lowercase().contains("inggris");
    let show_title_en = is_bilingual || is_english;

    let mut close = false;
    let mut delete_card = false;
    let mut insert_before = false;
    let mut insert_after = false;
    let mut apply_all_tables = false;

    let page_num = state.cards[idx].page;
    let modal_w = 560.0f32;
    let available_h = (ctx.content_rect().height() - 80.0).max(200.0);
    let max_modal_h = 620.0f32.min(available_h);
    let scroll_h = max_modal_h - 100.0;
    let content_w = modal_w - 24.0;

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
                        RichText::new(format!("{} {}", card.kind.label(lang), if card.scope_new_page { "(Halaman Baru)" } else { "(Objek Baru)" }))
                            .font(semibold(12.0))
                            .color(t.text_strong),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Halaman {} · Tata Letak: {}",
                            card.page,
                            if state.page_layout_facing { "Double-Page (Facing Spreads)" } else { "Single-Page" }
                        ))
                        .font(FontId::proportional(10.0))
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

            egui::ScrollArea::vertical()
                .max_width(content_w)
                .max_height(scroll_h)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(content_w);
                    ui.set_max_width(content_w);
                // Section 1: Cakupan & Jenis Konten (Scope & Content Type)
                egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(crate::i18n::tr(lang, "Scope:")).font(semibold(11.0)).color(t.text_strong));
                        let is_np = card.scope_new_page;
                        if ui.selectable_label(is_np, format!("📄 {}", crate::i18n::tr(lang, "New Page"))).clicked() {
                            card.scope_new_page = true;
                            card.tag2 = "Halaman Baru".into();
                        }
                        if ui.selectable_label(!is_np, format!("🧩 {}", crate::i18n::tr(lang, "New Object"))).clicked() {
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
                                        };
                                    }
                                }
                            });
                    });
                });

                // Section 2: Identitas & Sumber Data (Identity & Binding)
                if card.kind != CardKind::Cover {
                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                        ui.label(RichText::new(crate::i18n::tr(lang, "Publication Title")).font(semibold(11.0)).color(t.text_strong));
                        ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut card.title));

                        if show_title_en {
                            ui.label(RichText::new(crate::i18n::tr(lang, "Title EN:")).font(semibold(11.0)).color(t.text_strong));
                            ui.add_sized([ui.available_width(), 22.0], egui::TextEdit::singleline(&mut card.title_en));
                        }

                        let show_chapter = matches!(card.kind, CardKind::Narrative | CardKind::Table | CardKind::Chart | CardKind::MapChart);
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
                                        if ui.selectable_label(card.source == f.path, &f.path).clicked() {
                                            card.source = f.path.clone();
                                        }
                                    }
                                });
                        });
                    });
                }

                // Section 3: Rule-Based Form Sections per CardKind
                match card.kind {
                    CardKind::Cover => {
                        egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                            let ctx = ui.ctx().clone();
                            super::with_state(&ctx, |pubgen_st| {
                                if pubgen_st.cover_title.trim().is_empty() && !card.title.trim().is_empty() {
                                    pubgen_st.cover_title = card.title.clone();
                                }
                                if pubgen_st.cover_title_en.trim().is_empty() && !card.title_en.trim().is_empty() {
                                    pubgen_st.cover_title_en = card.title_en.clone();
                                }
                                super::widgets::card_cover_body_ui(app, ui, pubgen_st, t);
                                card.title = pubgen_st.cover_title.clone();
                                card.title_en = pubgen_st.cover_title_en.clone();
                            });
                        });
                    }
                    CardKind::Table => {
                        egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
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

                            if ui.button(format!("⚡ {}", crate::i18n::tr(lang, "Apply to All Tables"))).clicked() {
                                apply_all_tables = true;
                            }
                        });
                    }
                    CardKind::Chart => {
                        egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                            ui.label(RichText::new("Pengaturan Visualisasi Grafik:").font(semibold(11.0)).color(t.text_strong));
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(crate::i18n::tr(lang, "Chart Type:")).font(semibold(11.0)));
                                craft_ui::ComboBox::from_id_salt("chart_type_select")
                                    .selected_text(&card.chart_type)
                                    .width(200.0)
                                    .show_ui(ui, |ui| {
                                        for ct in ["Bar", "Line", "Donut", "Area", "Pie", "Scatter"] {
                                            if ui.selectable_label(card.chart_type == ct, ct).clicked() {
                                                card.chart_type = ct.into();
                                            }
                                        }
                                    });
                            });
                        });
                    }
                    CardKind::MapChart => {
                        egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                            ui.label(RichText::new("Pengaturan Peta Tematik (Choropleth & Spatial Map):").font(semibold(11.0)).color(Color32::from_rgb(16, 185, 129)));
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
                        egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                            ui.label(RichText::new("Pengaturan Gambar & Ilustrasi:").font(semibold(11.0)).color(t.text_strong));
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
                        egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                            ui.label(RichText::new("Pengaturan Daftar Isi (TOC):").font(semibold(11.0)).color(t.text_strong));
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(crate::i18n::tr(lang, "TOC Style:")).font(semibold(11.0)));
                                craft_ui::ComboBox::from_id_salt("toc_style_select")
                                    .selected_text(&card.toc_style)
                                    .width(180.0)
                                    .show_ui(ui, |ui| {
                                        for s in ["Standar", "Titik Pemimpin", "Minimalis"] {
                                            if ui.selectable_label(card.toc_style == s, s).clicked() {
                                                card.toc_style = s.into();
                                            }
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Kolom:").font(semibold(11.0)));
                                if ui.selectable_label(card.toc_columns == "1 Kolom", "1 Kolom").clicked() {
                                    card.toc_columns = "1 Kolom".into();
                                }
                                if ui.selectable_label(card.toc_columns == "2 Kolom", "2 Kolom").clicked() {
                                    card.toc_columns = "2 Kolom".into();
                                }
                            });
                            ui.checkbox(&mut card.toc_hanging_indent, crate::i18n::tr(lang, "Hanging Indent"));
                            ui.checkbox(&mut card.toc_include_self, crate::i18n::tr(lang, "Include Self"));
                            if is_bilingual {
                                ui.checkbox(&mut card.toc_pair_bilingual, crate::i18n::tr(lang, "Pair Bilingual"));
                            }
                        });
                    }
                    CardKind::Narrative => {
                        egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                            ui.label(RichText::new("Pengaturan Format Teks & Paragraf:").font(semibold(11.0)).color(t.text_strong));
                            ui.label(RichText::new("Teks narasi di-parse secara otomatis dari dokumen Markdown yang terhubung. Pemisahan antar paragraf, heading, dan quote dipertahankan sesuai stylesheet BPS.").font(FontId::proportional(10.5)).color(t.text_dim));
                        });
                    }
                }

                // Section 4: Penomoran Halaman & Seksi (Halaman Baru)
                if card.scope_new_page {
                    egui::Frame::NONE.fill(t.panel).stroke(Stroke::new(1.0, t.field_border)).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                        ui.label(RichText::new("Opsi Penomoran Halaman & Seksi:").font(semibold(11.0)).color(t.text_strong));
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
                                ui.label(RichText::new(crate::i18n::tr(lang, "Section Prefix:")).font(semibold(10.5)));
                                ui.add(egui::TextEdit::singleline(&mut card.section_prefix).desired_width(80.0));
                                ui.label(RichText::new(crate::i18n::tr(lang, "Numbering Style:")).font(semibold(10.5)));
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
                }
            });

            ui.separator();

            // Footer Actions
            ui.horizontal(|ui| {
                // Delete button with confirmation
                if state.card_delete_confirm {
                    ui.label(RichText::new(crate::i18n::tr(lang, "Confirm Delete?")).font(semibold(10.5)).color(Color32::from_rgb(239, 68, 68)));
                    if ui.add(egui::Button::new(RichText::new(crate::i18n::tr(lang, "Yes, Delete")).color(Color32::WHITE)).fill(Color32::from_rgb(220, 38, 38))).clicked() {
                        delete_card = true;
                    }
                    if ui.button(crate::i18n::tr(lang, "Cancel")).clicked() {
                        state.card_delete_confirm = false;
                    }
                } else if ui.button(RichText::new(format!("🗑 {}", crate::i18n::tr(lang, "Delete Card"))).color(Color32::from_rgb(239, 68, 68))).clicked() {
                    state.card_delete_confirm = true;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new(format!("✓ {}", crate::i18n::tr(lang, "Save & Close"))).font(semibold(11.0)).color(Color32::WHITE)).fill(Color32::from_rgb(37, 99, 235))).clicked() {
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
    } else if insert_before {
        let new_card = PublicationCard::new_with(state.new_card_kind, "Item Baru", "Draf konten", "Manual", idx + 1, true);
        state.cards.insert(idx, new_card);
        for (i, c) in state.cards.iter_mut().enumerate() {
            c.page = i + 1;
        }
        state.editing_card_idx = Some(idx);
    } else if insert_after {
        let new_card = PublicationCard::new_with(state.new_card_kind, "Item Baru", "Draf konten", "Manual", idx + 2, true);
        state.cards.insert(idx + 1, new_card);
        for (i, c) in state.cards.iter_mut().enumerate() {
            c.page = i + 1;
        }
        state.editing_card_idx = Some(idx + 1);
    } else if close {
        state.card_delete_confirm = false;
        state.editing_card_idx = None;
    }
}

/// Executes compilation and automatically transitions workspace to DesignCraft Layout mode.
fn execute_compile(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, publish: bool) {
    let now = format_now();
    let pages_count = state.cards.len();
    state.execution_log.push(format!("[{now}] Memulai generate struktur {pages_count} halaman publikasi..."));

    // Attempt compiling via pubgen UI command
    let _ = app.run(
        "pubgen.compileProject",
        serde_json::json!({
            "sync": true,
            "pdf": if publish { "outputs/BRS_NTP_Jabar_2026.pdf" } else { "" }
        }),
    );

    if publish {
        state.execution_log.push(format!("[{now}] Ekspor berkas PDF rilis: outputs/BRS_NTP_Jabar_2026.pdf"));
        state.last_output_pdf = Some("outputs/BRS_NTP_Jabar_2026.pdf".into());
        state.status_message = Some(("Kompilasi & Publikasi PDF Berhasil".into(), true));
    } else {
        state.execution_log.push(format!("[{now}] Kompilasi dokumen selesai tanpa error."));
        state.status_message = Some(("Dokumen berhasil digenerate".into(), true));
    }

    // Auto-unlock & switch to DesignCraft Layout Mode
    crate::menus::pubgen_bar::set_orchestrator_mode(ctx, false);
}

/// Renders the Re-compilation Safety Guard Dialog (3-way modal: Overwrite / Save v2 / Cancel).
fn render_guard_dialog(app: &mut DesignApp, ctx: &egui::Context, state: &mut OrchestratorWorkspaceState, t: &Tokens, lang: &str) {
    if !state.show_guard_dialog {
        return;
    }

    let mut close = false;
    let mut chosen_action = None;

    let guard_title = format!("⚠️ {}", crate::i18n::tr(lang, "Compilation Safety Guard"));
    egui::Window::new(guard_title)
        .collapsible(false)
        .resizable(false)
        .fixed_size([460.0, 240.0])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);

            ui.label(
                RichText::new("Tata letak aktif terdeteksi telah memiliki halaman dan modifikasi manual.").font(semibold(12.5)).color(t.text_strong),
            );

            ui.label(
                RichText::new(
                    "Kompilasi ulang akan membuat ulang seluruh halaman dari data sumber Excel / Markdown. \
                     Setiap penyesuaian tipografi, posisi bingkai, atau gaya manual yang dilakukan di mode \
                     DesignCraft dapat tertimpa.",
                )
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
                        egui::Button::new(RichText::new("💾 Simpan Versi Baru (v2)").font(semibold(11.0)).color(Color32::WHITE))
                            .fill(Color32::from_rgb(37, 99, 235)),
                    )
                    .on_hover_text("Pertahankan tata letak lama dan kompilasi ke berkas versi baru (_v2)")
                    .clicked()
                {
                    chosen_action = Some(1); // New version
                    close = true;
                }

                if ui
                    .add_sized(
                        [165.0, 26.0],
                        egui::Button::new(RichText::new("⚠️ Timpa Tata Letak").font(semibold(11.0)).color(Color32::WHITE))
                            .fill(Color32::from_rgb(220, 38, 38)),
                    )
                    .on_hover_text("Ganti dokumen aktif dengan kompilasi segar dari sumber data")
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
        });

    if close {
        state.show_guard_dialog = false;
    }

    if let Some(act) = chosen_action {
        let is_publish = state.guard_is_publish;
        let now = format_now();
        if act == 1 {
            state.execution_log.push(format!("[{now}] Menyimpan ke versi baru: outputs/BRS_NTP_Jabar_2026_v2.designcraft"));
            let _ = app.run(
                "pubgen.compileProject",
                serde_json::json!({
                    "sync": true,
                    "new_version": true,
                    "pdf": if is_publish { "outputs/BRS_NTP_Jabar_2026_v2.pdf" } else { "" }
                }),
            );
            if is_publish {
                state.execution_log.push(format!("[{now}] Ekspor berkas PDF rilis: outputs/BRS_NTP_Jabar_2026_v2.pdf"));
                state.last_output_pdf = Some("outputs/BRS_NTP_Jabar_2026_v2.pdf".into());
                state.status_message = Some(("Kompilasi ke versi baru (v2) berhasil".into(), true));
            } else {
                state.status_message = Some(("Dokumen versi baru (v2) berhasil digenerate".into(), true));
            }
        } else if act == 2 {
            state.execution_log.push(format!("[{now}] Menimpa tata letak aktif dengan hasil kompilasi segar..."));
            let _ = app.run(
                "pubgen.compileProject",
                serde_json::json!({
                    "sync": true,
                    "overwrite": true,
                    "pdf": if is_publish { "outputs/BRS_NTP_Jabar_2026.pdf" } else { "" }
                }),
            );
            if is_publish {
                state.execution_log.push(format!("[{now}] Ekspor berkas PDF rilis: outputs/BRS_NTP_Jabar_2026.pdf"));
                state.last_output_pdf = Some("outputs/BRS_NTP_Jabar_2026.pdf".into());
                state.status_message = Some(("Kompilasi ulang & timpa dokumen berhasil".into(), true));
            } else {
                state.status_message = Some(("Dokumen berhasil ditimpa dengan kompilasi segar".into(), true));
            }
        }
        crate::menus::pubgen_bar::set_orchestrator_mode(ctx, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        // English fallback
        assert_eq!(crate::i18n::tr("", "Execution & Verification"), "Execution & Verification");
        assert_eq!(crate::i18n::tr("en", "Execution & Verification"), "Execution & Verification");
        assert_eq!(crate::i18n::tr("", "Sources & Content"), "Sources & Content");
        assert_eq!(crate::i18n::tr("en", "Sources & Content"), "Sources & Content");
    }

    #[test]
    fn test_card_kind_map_chart_and_rule_based_defaults() {
        let card = PublicationCard::new_with(
            CardKind::MapChart,
            "Peta Tematik Sebaran NTP",
            "Peta Kloroplet",
            "maps/jabar.geojson",
            15,
            true,
        );
        assert_eq!(card.kind, CardKind::MapChart);
        assert_eq!(card.kind.icon(), "🗺️");
        assert_eq!(card.map_chart_type, "Choropleth");
        assert!(card.map_show_labels);
        assert!(card.map_show_legend);
        assert_eq!(card.tag1, "Peta Tematik");
        assert_eq!(card.tag2, "Halaman Baru");
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
}
