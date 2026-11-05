use serde::{Deserialize, Serialize};
use crate::i18n::UiLang;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum PageLayout {
    Single,
    #[default]
    Double,
}

impl PageLayout {
    pub const ALL: [PageLayout; 2] = [PageLayout::Single, PageLayout::Double];
    pub fn label(&self) -> &'static str {
        match self {
            PageLayout::Single => "Single Page",
            PageLayout::Double => "Double Page (Facing)",
        }
    }
}

impl std::fmt::Display for PageLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Lang {
    Id,
    En,
    #[default]
    IdEn,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Id, Lang::En, Lang::IdEn];
    pub fn label(&self) -> &'static str {
        match self {
            Lang::Id => "Indonesian",
            Lang::En => "English",
            Lang::IdEn => "Bilingual (ID/EN)",
        }
    }
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Paper {
    #[default]
    A4,
    B5,
    Letter,
    Custom,
}

impl Paper {
    pub const ALL: [Paper; 4] = [Paper::A4, Paper::B5, Paper::Letter, Paper::Custom];
    pub fn label(&self) -> &'static str {
        match self {
            Paper::A4 => "A4 (210 x 297 mm)",
            Paper::B5 => "B5 (176 x 250 mm)",
            Paper::Letter => "Letter (8.5 x 11 in)",
            Paper::Custom => "Custom",
        }
    }
}

impl std::fmt::Display for Paper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[allow(dead_code)]
pub const MARGINS_MIN: f64 = 5.0;
#[allow(dead_code)]
pub const MARGINS_MAX: f64 = 100.0;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarginSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverField {
    ReleaseNo,
    Date,
    Title,
    Title2,
    TitleEn,
    Month,
    PubOffice,
    Hero,
    Logo,
    Point,
    Qr,
    PicName,
    PicPhoto,
    Position,
    Office,
    Phone,
    Email,
    Year,
    Volume,
    Catalogue,
    Code,
    Agency,
    Palette,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Margins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 20.0,
            right: 20.0,
            bottom: 20.0,
            left: 20.0,
        }
    }
}

/// Scope of content items: whether it creates a new page or flows vertically on the current page.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum PageItemScope {
    #[default]
    NewPage,
    NewObject,
}

impl PageItemScope {
    pub const ALL: [PageItemScope; 2] = [PageItemScope::NewPage, PageItemScope::NewObject];
    #[allow(dead_code)]
    pub fn label(&self, lang: UiLang) -> &'static str {
        match self {
            PageItemScope::NewPage => crate::i18n::tr(lang, "scope_page"),
            PageItemScope::NewObject => crate::i18n::tr(lang, "scope_object"),
        }
    }
}

impl std::fmt::Display for PageItemScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewPage => write!(f, "Halaman Baru (New Page)"),
            Self::NewObject => write!(f, "Objek Baru (New Object)"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum PageKind {
    #[default]
    Paragraph,
    Table,
    Chart,
    Image,
    Cover,
    Toc,
}

impl PageKind {
    pub const ALL: [PageKind; 6] = [PageKind::Paragraph, PageKind::Table, PageKind::Chart, PageKind::Image, PageKind::Cover, PageKind::Toc];
    /// Localized display label for the UI.
    pub fn label(&self, lang: UiLang) -> &'static str {
        crate::i18n::tr(
            lang,
            match self {
                PageKind::Paragraph => "kind_paragraph",
                PageKind::Table => "kind_table",
                PageKind::Chart => "kind_chart",
                PageKind::Image => "kind_image",
                PageKind::Cover => "kind_cover",
                PageKind::Toc => "kind_toc",
            },
        )
    }
    /// Language-neutral label for logs and persisted extras.
    pub fn label_en(&self) -> &'static str {
        match self {
            PageKind::Paragraph => "Paragraph",
            PageKind::Table => "Table",
            PageKind::Chart => "Chart",
            PageKind::Image => "Image",
            PageKind::Cover => "Cover",
            PageKind::Toc => "TOC",
        }
    }
}

impl std::fmt::Display for PageKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label_en())
    }
}

/// Placement of an image content entry on the page.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum ImageArea {
    FullPage,
    #[default]
    MarginArea,
}

impl ImageArea {
    pub const ALL: [ImageArea; 2] = [ImageArea::FullPage, ImageArea::MarginArea];
    #[allow(dead_code)]
    pub fn label(&self, lang: UiLang) -> &'static str {
        match self {
            ImageArea::FullPage => crate::i18n::tr(lang, "image_full_page"),
            ImageArea::MarginArea => crate::i18n::tr(lang, "image_margin_area"),
        }
    }
}

impl std::fmt::Display for ImageArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FullPage => write!(f, "Satu Halaman Penuh (Full Page)"),
            Self::MarginArea => write!(f, "Dalam Margin (Margin Area)"),
        }
    }
}

/// What a TOC card lists — mirrors the reference pubgen "TOC style" combo.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum TocStyle {
    /// TOC Entry: every page whose title carries the TOC tag style, plus narrative pages.
    #[default]
    Entry,
    /// TOC Table: only pages whose first text box starts with "Tabel"/"Table".
    Table,
    /// TOC Chart: only pages whose first text box starts with "Grafik"/"Chart".
    Chart,
    /// TOC Paragraph: the narrative pages (G/R sheets) with their headings nested under them.
    Paragraph,
}

impl TocStyle {
    pub const ALL: [TocStyle; 4] = [TocStyle::Entry, TocStyle::Table, TocStyle::Chart, TocStyle::Paragraph];
    pub fn label(self, lang: UiLang) -> &'static str {
        crate::i18n::tr(
            lang,
            match self {
                TocStyle::Entry => "toc_style_entry",
                TocStyle::Table => "toc_style_table",
                TocStyle::Chart => "toc_style_chart",
                TocStyle::Paragraph => "toc_style_paragraph",
            },
        )
    }
}

/// TOC entry layout — mirrors the reference pubgen "TOC layout" combo.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum TocLayout {
    #[default]
    Standard,
    /// BPS statistical catalog layout with a "Tabel / halaman" header and hanging indent.
    TablePage,
}

impl TocLayout {
    pub const ALL: [TocLayout; 2] = [TocLayout::Standard, TocLayout::TablePage];
    pub fn label(self, lang: UiLang) -> &'static str {
        crate::i18n::tr(
            lang,
            match self {
                TocLayout::Standard => "toc_layout_standard",
                TocLayout::TablePage => "toc_layout_tablepage",
            },
        )
    }
}

/// Options for a Table of Contents card, mirroring the reference pubgen TOC form.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TocData {
    /// TOC title (e.g. "Daftar Isi", "DAFTAR TABEL / LIST OF TABLES").
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub style: TocStyle,
    #[serde(default)]
    pub layout: TocLayout,
    /// Indent wrapped TOC entries (hanging indent).
    #[serde(default)]
    pub hanging_indent: bool,
    /// Include the TOC page in its own entries.
    #[serde(default)]
    pub include_self: bool,
    /// Pair bilingual headings (ID / EN) into one row.
    #[serde(default)]
    pub pair_bilingual: bool,
}

impl Default for TocData {
    fn default() -> Self {
        Self {
            title: "Daftar Isi".to_string(),
            style: TocStyle::Entry,
            layout: TocLayout::Standard,
            hanging_indent: true,
            include_self: false,
            pair_bilingual: false,
        }
    }
}

impl TocData {
    /// The title a TOC style fills in when picked; `None` leaves the typed title alone
    /// (reference `toc_kind_title`).
    pub fn auto_title(style: TocStyle, bilingual: bool) -> Option<&'static str> {
        match (style, bilingual) {
            (TocStyle::Table, false) => Some("Daftar Tabel"),
            (TocStyle::Table, true) => Some("Daftar Tabel / List of Tables"),
            (TocStyle::Chart, false) => Some("Daftar Grafik"),
            (TocStyle::Chart, true) => Some("Daftar Grafik / List of Charts"),
            (TocStyle::Paragraph, false) => Some("Daftar Isi"),
            (TocStyle::Paragraph, true) => Some("Daftar Isi / Table of Contents"),
            _ => None,
        }
    }

    /// The title the TOC writes; "Daftar Isi" when the field is empty.
    pub fn title_or_default(&self) -> &str {
        if self.title.trim().is_empty() {
            "Daftar Isi"
        } else {
            self.title.trim()
        }
    }
}

/// A metadata form field, shown dynamically based on the active publication profile.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaField {
    Title,
    TitleEn,
    Subtitle,
    Year,
    Volume,
    Catalogue,
    Code,
    ReleaseNo,
    ReleaseDate,
    ReleaseMonth,
    Authors,
    Publisher,
}

#[allow(dead_code)]
impl MetaField {
    pub fn label(&self, lang: UiLang) -> &'static str {
        crate::i18n::tr(
            lang,
            match self {
                Self::Title => "meta_title",
                Self::TitleEn => "meta_title_en",
                Self::Subtitle => "meta_subtitle",
                Self::Year => "meta_year",
                Self::Volume => "meta_volume",
                Self::Catalogue => "meta_catalogue",
                Self::Code => "meta_code",
                Self::ReleaseNo => "meta_release_no",
                Self::ReleaseDate => "meta_release_date",
                Self::ReleaseMonth => "meta_release_month",
                Self::Authors => "meta_authors",
                Self::Publisher => "meta_publisher",
            },
        )
    }
}

fn default_true() -> bool {
    true
}

fn default_one() -> u32 {
    1
}

fn default_arabic() -> String {
    "arabic".to_string()
}

fn default_automatic() -> String {
    "automatic".to_string()
}

fn default_table_style() -> String {
    "Basic Grid".to_string()
}

fn default_table_theme() -> String {
    "#FFCB56 - #FFE9A9".to_string()
}

fn default_layout_preset() -> String {
    "1 Row · 3 Columns".to_string()
}

#[allow(dead_code)]
pub const CANONICAL_COVER_TEMPLATES: &[(&str, &str, &str)] = &[
    ("000_wave_curves", "Wave Curves", "assets/templates/covers/000_wave_curves.designcraft"),
    ("001_geometric_medallion", "Geometric Medallion", "assets/templates/covers/001_geometric_medallion.designcraft"),
    ("002_folded_origami", "Folded Origami", "assets/templates/covers/002_folded_origami.designcraft"),
    ("003_chevron_medallion", "Chevron Medallion", "assets/templates/covers/003_chevron_medallion.designcraft"),
    ("004_honeycomb_hexagon", "Honeycomb Hexagon", "assets/templates/covers/004_honeycomb_hexagon.designcraft"),
    ("005_corporate_headline", "Corporate Headline", "assets/templates/covers/005_corporate_headline.designcraft"),
    ("006_diagonal_arrow", "Diagonal Arrow", "assets/templates/covers/006_diagonal_arrow.designcraft"),
    ("007_modern_horizon", "Modern Horizon", "assets/templates/covers/007_modern_horizon.designcraft"),
    ("008_business_report", "Business Report", "assets/templates/covers/008_business_report.designcraft"),
    ("009_urban_angle", "Urban Angle", "assets/templates/covers/009_urban_angle.designcraft"),
    ("010_tri_color_bands", "Tri-Color Bands", "assets/templates/covers/010_tri_color_bands.designcraft"),
    ("011_prism_diamonds", "Prism Diamonds", "assets/templates/covers/011_prism_diamonds.designcraft"),
    ("012_quarter_stack", "Quarter Stack", "assets/templates/covers/012_quarter_stack.designcraft"),
    ("brs", "BRS Official", "assets/templates/covers/brs/brs.designcraft"),
    ("100_ai_base", "AI Base Cover", "assets/templates/covers/100_ai_base.designcraft"),
];

#[allow(dead_code)]
pub const AI_COVER_PRESETS: &[&str] = &[
    "Retro Waves",
    "Geometric Business",
    "Minimal Sci-Fi",
    "Dynamic Waves Spread",
    "Vibrant S-Curve Spread",
];

#[allow(dead_code)]
pub const AI_COVER_PALETTES: &[(&str, &str, &str)] = &[
    ("Warm Terracotta", "#B5543B", "#E8A34D"),
    ("Pastel Dream", "#8E7CC3", "#E8A0BF"),
    ("Mono Slate", "#53616C", "#9FB1BC"),
    ("Forest Gold", "#2F5D50", "#C9A227"),
    ("Corporate Blue", "#003399", "#00A8FF"),
    ("Teal Lime", "#006D6F", "#84CC16"),
];

#[allow(dead_code)]
pub const NUMBERING_STYLES: &[(&str, &str)] = &[
    ("arabic", "1, 2, 3, 4…"),
    ("upper_roman", "I, II, III, IV…"),
    ("lower_roman", "i, ii, iii, iv…"),
    ("upper_letters", "A, B, C, D…"),
    ("lower_letters", "a, b, c, d…"),
    ("leading_zero", "01, 02, 03…"),
    ("three_digits", "001, 002, 003…"),
];

#[allow(dead_code)]
pub const TABLE_STYLES: &[&str] = &[
    "Basic Grid",
    "Corporate Navy",
    "Minimalist",
    "Horizontal Rules",
    "BRS6",
];

#[allow(dead_code)]
pub const TABLE_THEMES: &[(&str, &str)] = &[
    ("#FFCB56 - #FFE9A9", "Gold / Amber"),
    ("#1A73E8 - #E8F0FE", "Corporate Blue"),
    ("#006D6F - #E6F4F1", "Teal Green"),
    ("#2F5D50 - #E8F0EC", "Forest Green"),
    ("#53616C - #F1F3F5", "Slate Monochrome"),
];

#[allow(dead_code)]
pub const TABLE_LAYOUT_PRESETS: &[&str] = &[
    "1 Row · 3 Columns",
    "1 Row · 3 Equal Cols",
    "Editorial Chapter (Stacked)",
    "Technical Spec Card",
    "Minimal Zine",
];

/// InDesign-compatible Numbering & Section Options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SectionOptions {
    #[serde(default = "default_true")]
    pub start_section: bool,
    #[serde(default = "default_true")]
    pub automatic_numbering: bool,
    #[serde(default = "default_one")]
    pub start_page_number: u32,
    #[serde(default)]
    pub section_prefix: String,
    #[serde(default = "default_arabic")]
    pub numbering_style: String,
    #[serde(default)]
    pub section_marker: String,
    #[serde(default)]
    pub include_prefix: bool,
    #[serde(default = "default_arabic")]
    pub chapter_style: String,
    #[serde(default = "default_automatic")]
    pub chapter_source: String,
    #[serde(default = "default_one")]
    pub chapter_start: u32,
}

impl Default for SectionOptions {
    fn default() -> Self {
        Self {
            start_section: true,
            automatic_numbering: true,
            start_page_number: 1,
            section_prefix: String::new(),
            numbering_style: "arabic".to_string(),
            section_marker: String::new(),
            include_prefix: false,
            chapter_style: "arabic".to_string(),
            chapter_source: "automatic".to_string(),
            chapter_start: 1,
        }
    }
}

/// Table styling and layout presets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableOptions {
    #[serde(default = "default_table_style")]
    pub style: String,
    #[serde(default = "default_table_theme")]
    pub theme: String,
    #[serde(default = "default_layout_preset")]
    pub layout_preset: String,
    #[serde(default = "default_true")]
    pub first_row_header: bool,
    #[serde(default = "default_true")]
    pub col_numbers: bool,
    #[serde(default = "default_true")]
    pub table_footer: bool,
    #[serde(default = "default_true")]
    pub row_conditions: bool,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            style: "Basic Grid".to_string(),
            theme: "#FFCB56 - #FFE9A9".to_string(),
            layout_preset: "1 Row · 3 Columns".to_string(),
            first_row_header: true,
            col_numbers: true,
            table_footer: true,
            row_conditions: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct PageEntry {
    #[serde(default)]
    pub kind: PageKind,
    #[serde(default)]
    pub scope: PageItemScope,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub chapter_id: String,
    #[serde(default)]
    pub source: Option<String>,
    /// Chart-specific option: only meaningful when `kind == Chart`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chart_type: Option<crate::vector::ChartType>,
    /// Image-specific option: only meaningful when `kind == Image`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_area: Option<ImageArea>,
    /// Cover-specific option: key/value fields mirroring the XLSX `COV` sheet and Cover Generator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<ExcelCoverData>,
    /// Section & page numbering options starting at this page forward.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<SectionOptions>,
    /// Table formatting preset: only meaningful when `kind == Table`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table_options: Option<TableOptions>,
    /// TOC-specific option: only meaningful when `kind == Toc`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toc: Option<TocData>,
    /// Whether the underlying content begins with a Paragraph of style "Title".
    #[serde(default)]
    pub has_title: bool,
}

impl PageEntry {
    /// Reset kind-specific options after a kind switch.
    pub fn sync_kind_options(&mut self) {
        self.chart_type = (self.kind == PageKind::Chart).then(crate::vector::ChartType::default);
        self.image_area = (self.kind == PageKind::Image).then(ImageArea::default);
        self.cover = (self.kind == PageKind::Cover).then(ExcelCoverData::default);
        self.table_options = (self.kind == PageKind::Table).then(TableOptions::default);
        if self.scope == PageItemScope::NewPage && self.section.is_none() {
            self.section = Some(SectionOptions::default());
        }
        self.toc = (self.kind == PageKind::Toc).then(TocData::default);
    }
}

/// Metadata extracted from Excel `COVER` sheet.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ExcelCoverData {
    pub style: String,
    pub level: String,
    pub title1: String,
    pub title2: String,
    pub title_en: String,
    pub year: String,
    pub volume: String,
    pub catalogue: String,
    pub code: String,
    pub agency: String,
    pub pub_office: String,
    pub image_hero: String,
    pub image_landscape: String,
    pub logo: String,
    pub table_theme: String,
    pub event_theme: String,
    pub office_address: String,
    pub palette: String,
    pub release_no: String,
    pub release_date: String,
    pub release_month: String,
    /// BRS COV sheet extras: summary point, QR image, contact person, office info.
    pub point: String,
    pub qr: String,
    pub pic_name: String,
    pub pic_photo: String,
    pub position: String,
    pub office: String,
    pub phone: String,
    pub email: String,
    /// Path or identifier to template in `assets/templates/covers/`
    #[serde(default)]
    pub template_path: String,
    #[serde(default)]
    pub ai_theme: String,
    #[serde(default)]
    pub ai_preset: String,
    #[serde(default)]
    pub ai_palette: String,
    #[serde(default)]
    pub include_inside_back: bool,
    #[serde(default)]
    pub add_map: bool,
    #[serde(default)]
    pub map_style: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    #[serde(default = "default_profile_id")]
    pub profile_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub issue: String,
    #[serde(default)]
    pub volume: String,
    #[serde(default)]
    pub catalogue: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub release_no: String,
    #[serde(default)]
    pub release_date: String,
    #[serde(default)]
    pub release_month: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub agency: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub authors: String,
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub layout: PageLayout,
    #[serde(default)]
    pub lang: Lang,
    #[serde(default)]
    pub paper: Paper,
    #[serde(default)]
    pub margins: Margins,
    #[serde(default)]
    pub pages: Vec<PageEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_table_options: Option<TableOptions>,
}

fn default_profile_id() -> String {
    "bps_dda".to_string()
}

impl Default for Metadata {
    fn default() -> Self {
        let mut m = Self {
            profile_id: default_profile_id(),
            name: String::new(),
            description: String::new(),
            title: String::new(),
            subtitle: String::new(),
            title_en: String::new(),
            issue: String::new(),
            volume: String::new(),
            catalogue: String::new(),
            code: String::new(),
            date: String::new(),
            release_no: String::new(),
            release_date: String::new(),
            release_month: String::new(),
            publisher: String::new(),
            agency: String::new(),
            address: String::new(),
            language: String::new(),
            authors: String::new(),
            price: String::new(),
            layout: PageLayout::Double,
            lang: Lang::IdEn,
            paper: Paper::A4,
            margins: Margins::default(),
            pages: Vec::new(),
            default_table_options: None,
        };
        m.apply_profile("bps_dda");
        m
    }
}

impl Metadata {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    pub fn from_json(v: &serde_json::Value) -> Metadata {
        serde_json::from_value(v.clone()).unwrap_or_default()
    }

    /// Whether this metadata is in bilingual Indonesian-English mode.
    pub fn is_bilingual(&self) -> bool {
        self.lang == Lang::IdEn
    }

    /// Memuat default metadata untuk profil tertentu dari template JSON profil.
    pub fn default_for_profile(profile_id: &str) -> Option<Metadata> {
        let clean = profile_id.trim().to_lowercase();
        let json = match clean.as_str() {
            "bps_dda" | "dda" => Some(include_str!("../assets/profiles/bps_dda.json")),
            "bps_brs" | "brs" => Some(include_str!("../assets/profiles/bps_brs.json")),
            "academic_monograph" => Some(include_str!("../assets/profiles/academic_monograph.json")),
            "corporate_annual" => Some(include_str!("../assets/profiles/corporate_annual.json")),
            _ => None,
        }?;
        let clean_json = json.trim_start_matches('\u{feff}');
        serde_json::from_str::<Metadata>(clean_json).ok()
    }

    /// Mengaplikasikan profil publikasi (BPS DDA, BPS BRS, Monograph, Corporate)
    /// dan mengisi nilai awal metadata dari file profil JSON.
    pub fn apply_profile(&mut self, profile_id: &str) {
        if let Some(def) = Self::default_for_profile(profile_id) {
            *self = def;
            return;
        }
        self.profile_id = profile_id.to_string();
    }

    /// Auto-populates or updates metadata using extracted Excel cover data.
    pub fn apply_excel_cover(&mut self, cover: &ExcelCoverData) {
        if !cover.title1.is_empty() {
            self.title = cover.title1.clone();
        }
        if !cover.title2.is_empty() && self.subtitle.is_empty() {
            self.subtitle = cover.title2.clone();
        }
        if !cover.title_en.is_empty() {
            self.title_en = cover.title_en.clone();
        }
        if !cover.year.is_empty() {
            self.date = cover.year.clone();
        }
        if !cover.volume.is_empty() {
            self.volume = cover.volume.clone();
        }
        if !cover.catalogue.is_empty() {
            self.catalogue = cover.catalogue.clone();
        }
        if !cover.code.is_empty() {
            self.code = cover.code.clone();
        }
        if !cover.agency.is_empty() {
            self.agency = cover.agency.clone();
        }
        if !cover.office_address.is_empty() {
            self.address = cover.office_address.clone();
        }
        if !cover.release_no.is_empty() {
            self.release_no = cover.release_no.clone();
        }
        if !cover.release_date.is_empty() {
            self.release_date = cover.release_date.clone();
        }
        if !cover.release_month.is_empty() {
            self.release_month = cover.release_month.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_json_round_trip() {
        let mut m = Metadata::default();
        m.pages.clear();
        m.title = "Catalog".into();
        m.title_en = "Catalog EN".into();
        m.layout = PageLayout::Double;
        m.lang = Lang::IdEn;
        m.paper = Paper::Letter;
        m.margins.left = 30.0;
        m.pages.push(PageEntry {
            kind: PageKind::Table,
            scope: PageItemScope::NewPage,
            title: "Numbers".into(),
            source: Some("tables/t.csv".into()),
            section: Some(SectionOptions {
                start_section: true,
                automatic_numbering: false,
                start_page_number: 5,
                section_prefix: "TAB-".into(),
                numbering_style: "upper_roman".into(),
                section_marker: "Tabel Utama".into(),
                include_prefix: true,
                chapter_style: "arabic".into(),
                chapter_source: "user_defined".into(),
                chapter_start: 2,
            }),
            table_options: Some(TableOptions {
                style: "Corporate Navy".into(),
                theme: "#1A73E8 - #E8F0FE".into(),
                layout_preset: "1 Row · 3 Columns".into(),
                ..Default::default()
            }),
            ..Default::default()
        });
        m.pages.push(PageEntry {
            kind: PageKind::Cover,
            scope: PageItemScope::NewPage,
            title: "Cover Page".into(),
            cover: Some(ExcelCoverData {
                template_path: "assets/templates/covers/000_wave_curves.designcraft".into(),
                ai_theme: "pertanian".into(),
                ai_preset: "Retro Waves".into(),
                ai_palette: "Corporate Blue".into(),
                include_inside_back: true,
                add_map: true,
                map_style: 1,
                ..Default::default()
            }),
            ..Default::default()
        });
        m.default_table_options = Some(TableOptions {
            style: "Basic Grid".into(),
            theme: "#FFCB56 - #FFE9A9".into(),
            layout_preset: "1 Row · 3 Equal Cols".into(),
            ..Default::default()
        });
        let v = m.to_json();
        let back = Metadata::from_json(&v);
        assert_eq!(back.title, "Catalog");
        assert_eq!(back.title_en, "Catalog EN");
        assert_eq!(back.layout, PageLayout::Double);
        assert_eq!(back.lang, Lang::IdEn);
        assert_eq!(back.paper, Paper::Letter);
        assert_eq!(back.margins.left, 30.0);
        assert_eq!(back.pages[0].kind, PageKind::Table);
        assert_eq!(back.pages[0].source.as_deref(), Some("tables/t.csv"));
        let sec = back.pages[0].section.as_ref().unwrap();
        assert_eq!(sec.start_page_number, 5);
        assert_eq!(sec.section_prefix, "TAB-");
        assert_eq!(sec.numbering_style, "upper_roman");
        let tbl = back.pages[0].table_options.as_ref().unwrap();
        assert_eq!(tbl.style, "Corporate Navy");
        assert_eq!(tbl.theme, "#1A73E8 - #E8F0FE");
        let cov = back.pages[1].cover.as_ref().unwrap();
        assert_eq!(cov.template_path, "assets/templates/covers/000_wave_curves.designcraft");
        assert_eq!(cov.ai_preset, "Retro Waves");
        assert!(cov.include_inside_back);
        assert!(cov.add_map);
        assert_eq!(back.default_table_options.unwrap().layout_preset, "1 Row · 3 Equal Cols");
    }

    #[test]
    fn page_kind_labels_are_localized() {
        assert_eq!(PageKind::Chart.label(UiLang::Id), "Grafik");
        assert_eq!(PageKind::Chart.label(UiLang::En), "Chart");
        assert_eq!(PageKind::Chart.label_en(), "Chart");
        assert_eq!(PageKind::Chart.to_string(), "Chart");
    }

    #[test]
    fn defaults_are_sensible() {
        let m = Metadata::default();
        assert_eq!(m.profile_id, "bps_dda");
        assert_eq!(m.layout, PageLayout::Double);
        assert_eq!(m.lang, Lang::IdEn);
        assert_eq!(m.paper, Paper::A4);
        assert_eq!(m.margins.top, 20.2);
        assert_eq!(m.margins.bottom, 20.0);
        assert_eq!(m.margins.left, 19.0);
        assert_eq!(m.margins.right, 15.0);
        assert!(!m.pages.is_empty());
        assert_eq!(m.pages[0].scope, PageItemScope::NewPage);
    }

    #[test]
    fn test_profile_switching_brs_and_scope() {
        let mut m = Metadata::default();
        m.pages.clear();
        m.apply_profile("bps_brs");
        assert_eq!(m.profile_id, "bps_brs");
        assert_eq!(m.layout, PageLayout::Single);
        assert_eq!(m.lang, Lang::Id);
        assert_eq!(m.margins.top, 20.2);
        assert_eq!(m.margins.left, 19.0);
        assert_eq!(m.release_no, "No. 01/01/Th. XXVII");
        assert!(!m.pages.is_empty());
        assert!(m.pages.iter().any(|p| p.scope == PageItemScope::NewObject));
        assert_eq!(m.pages[1].scope, PageItemScope::NewObject);

        // Test round trip with NewObject
        let v = m.to_json();
        let back = Metadata::from_json(&v);
        assert!(back.pages.iter().any(|p| p.scope == PageItemScope::NewObject));
        assert_eq!(back.pages[1].scope, PageItemScope::NewObject);
    }

    #[test]
    fn test_apply_excel_cover() {
        let mut m = Metadata::default();
        let cover = ExcelCoverData {
            title1: "KECAMATAN KARAWANG TIMUR".into(),
            title2: "DALAM ANGKA".into(),
            title_en: "EAST KARAWANG IN FIGURES".into(),
            year: "2026".into(),
            volume: "12".into(),
            catalogue: "1102001".into(),
            code: "ISSN 1234".into(),
            agency: "BPS Karawang".into(),
            office_address: "Jl. Ahmad Yani".into(),
            release_no: "61/10/32".into(),
            release_date: "1 Oktober 2026".into(),
            release_month: "Oktober 2026".into(),
            ..Default::default()
        };
        m.apply_excel_cover(&cover);
        assert_eq!(m.title, "KECAMATAN KARAWANG TIMUR");
        assert_eq!(m.subtitle, "DALAM ANGKA");
        assert_eq!(m.title_en, "EAST KARAWANG IN FIGURES");
        assert_eq!(m.date, "2026");
        assert_eq!(m.volume, "12");
        assert_eq!(m.catalogue, "1102001");
        assert_eq!(m.agency, "BPS Karawang");
        assert_eq!(m.release_no, "61/10/32");
    }

    #[test]
    fn kind_options_round_trip_and_sync() {
        let mut m = Metadata::default();
        m.pages.clear();
        m.pages.push(PageEntry {
            kind: PageKind::Chart,
            scope: PageItemScope::NewObject,
            title: "Inflasi".into(),
            chart_type: Some(crate::vector::ChartType::Donut),
            ..Default::default()
        });
        m.pages.push(PageEntry {
            kind: PageKind::Image,
            scope: PageItemScope::NewPage,
            title: "Sampul".into(),
            image_area: Some(ImageArea::MarginArea),
            ..Default::default()
        });
        m.pages.push(PageEntry {
            kind: PageKind::Cover,
            scope: PageItemScope::NewPage,
            title: "Cover".into(),
            cover: Some(ExcelCoverData {
                point: "NTP Oktober naik 0,21%".into(),
                qr: "qr_code.png".into(),
                pic_name: "Ani".into(),
                ..Default::default()
            }),
            ..Default::default()
        });
        let back = Metadata::from_json(&m.to_json());
        assert_eq!(back.pages[0].chart_type, Some(crate::vector::ChartType::Donut));
        assert_eq!(back.pages[1].image_area, Some(ImageArea::MarginArea));
        assert_eq!(back.pages[0].image_area, None);
        assert_eq!(back.pages[1].chart_type, None);
        let cover_back = back.pages[2].cover.as_ref().unwrap();
        assert_eq!(cover_back.point, "NTP Oktober naik 0,21%");
        assert_eq!(cover_back.pic_name, "Ani");
        assert_eq!(back.pages[2].kind, PageKind::Cover);

        // Kind switch resets kind-specific options.
        let mut e = back.pages.into_iter().next().unwrap();
        assert_eq!(e.chart_type, Some(crate::vector::ChartType::Donut));
        e.kind = PageKind::Image;
        e.sync_kind_options();
        assert_eq!(e.chart_type, None);
        assert_eq!(e.image_area, Some(ImageArea::MarginArea));
        assert_eq!(e.cover, None);
    }
}
