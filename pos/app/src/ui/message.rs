#![allow(dead_code)]

use iced::{window::Direction, Size};

use crate::covers::CoverStyle;
use crate::i18n::UiLang;
use crate::main_ui::CenterTab;
use crate::metadata_form::{
    CoverField, ImageArea, Lang, MarginSide, PageItemScope, PageKind, PageLayout, Paper, TocLayout, TocStyle,
};
use crate::toc::TocKind;
use crate::vector::ChartType;

/// Cloneable handle to the detached job's result `App`.
/// The update handler takes the payload out of the mutex exactly once.
#[derive(Clone, Default)]
pub struct JobPayload(pub std::sync::Arc<std::sync::Mutex<Option<crate::main_ui::App>>>);

impl std::fmt::Debug for JobPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JobPayload(…)")
    }
}

/// Cloneable handle to a DC health probe result.
#[derive(Clone)]
pub struct ProbePayload(pub std::sync::Arc<anyhow::Result<crate::control_client::ControlClient>>);

impl std::fmt::Debug for ProbePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProbePayload(…)")
    }
}

// No Clone on the enum itself is impossible (widgets require it); the
// non-clonable payloads live behind Arc handles above instead.
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    DcProbe(ProbePayload),
    JobDone(JobPayload),
    WindowResized(Size),
    SystemTheme { dark: bool },
    Chrome(ChromeMsg),
    Menu(MenuMsg),
    DcPillLaunched,
    DcPillReprobe,
    PanelToggled(&'static str),
    PanelsCollapseAll,
    PanelsExpandBasics,
    HomeToggled,
    GoHome,
    /// `Esc`: close the topmost transient surface (modal → menus → no-op).
    EscapePressed,
    /// Menu-bar keyboard navigation (`F10`, arrows, `Enter`, `Space`).
    MenuKey(MenuKey),
    CompilePressed,
    CompileMenuToggled,
    LangSelected(UiLang),
    // Home page
    #[allow(dead_code)]
    HomeNewProject,
    HomeOpen,
    HomeImportXlsx,
    HomeSampleBrs(std::path::PathBuf),
    HomeSampleDda(std::path::PathBuf),
    #[allow(dead_code)]
    HomeLaunchDc,
    ProfileCardSelected(&'static str, &'static str),
    RecentOpened(std::path::PathBuf),
    // Studio panels
    Action(&'static str),
    TabSelected(CenterTab),
    #[allow(dead_code)]
    Input(InputMsg),
    Combo(ComboMsg),
    #[allow(dead_code)]
    GlossaryAdd,
    #[allow(dead_code)]
    GlossaryRemove(usize),
    #[allow(dead_code)]
    GlossaryDnt(bool),
    CoverPromptCopy,
    #[allow(dead_code)]
    TocExtract(TocKind),
    #[allow(dead_code)]
    TocCopy,
    SlotRemove(usize),
    SlotUnname(usize),
    SlotToggled(usize),
    SlotBudget(usize, String),
    SlotSource(usize, String),
    SlotText(usize, String),
    ProfilePicked(crate::profiles::PubgenProfile),
    PreviewModeSelected(crate::main_ui::PreviewMode),
    CoverSvgCopy,
    Metadata(MetadataMsg),
    OpenDialog(DialogKind),
    CloseDialog,
    DialogMsg(DialogMsg),
}

#[derive(Debug, Clone)]
pub enum MetadataMsg {
    // Form fields
    Title(String),
    TitleEn(String),
    Subtitle(String),
    Date(String),
    Volume(String),
    Catalogue(String),
    Code(String),
    ReleaseNo(String),
    ReleaseDate(String),
    ReleaseMonth(String),
    Agency(String),
    Address(String),
    Layout(PageLayout),
    Lang(Lang),
    Paper(Paper),
    MarginStep(MarginSide, f64),

    // Publication cards
    CardOpen(usize),
    CardClose,
    CardScope(usize, PageItemScope),
    CardKind(usize, PageKind),
    CardTitle(usize, String),
    CardTitleEn(usize, String),
    CardChapterId(usize, String),
    CardSource(usize, Option<String>),
    CardChartType(usize, ChartType),
    CardImageArea(usize, ImageArea),
    CardCover(usize, CoverField, String),
    CardCoverTab(crate::ui::CardCoverTab),
    CardCoverPalettePreset(usize, usize),
    CardCoverTemplate(usize, String),
    CardCoverAiTheme(usize, String),
    CardCoverAiPreset(usize, String),
    CardCoverAiPalette(usize, String),
    CardCoverIncludeInsideBack(usize, bool),
    CardCoverAddMap(usize, bool),
    CardCoverMapStyle(usize, u8),

    CardSectionStart(usize, bool),
    CardSectionAutomatic(usize, bool),
    CardSectionStartPage(usize, u32),
    CardSectionPrefix(usize, String),
    CardSectionStyle(usize, String),
    CardSectionMarker(usize, String),
    CardSectionIncludePrefix(usize, bool),
    CardSectionChapterStyle(usize, String),
    CardSectionChapterSource(usize, String),
    CardSectionChapterStart(usize, u32),
    CardSectionApplyForward(usize),

    CardTableStyle(usize, String),
    CardTableTheme(usize, String),
    CardTableLayoutPreset(usize, String),
    CardTableFirstRowHeader(usize, bool),
    CardTableColNumbers(usize, bool),
    CardTableFooter(usize, bool),
    CardTableRowConditions(usize, bool),
    CardTableApplyToAll(usize),

    CardTocTitle(usize, String),
    CardTocStyle(usize, TocStyle),
    CardTocLayout(usize, TocLayout),
    CardTocFlag(usize, TocFlag, bool),
    /// Confirmed removal (G10): only sent after the inline confirmation.
    CardRemove(usize),
    /// First press on `Hapus`: arms the confirmation and mutates nothing.
    CardDeleteRequest,
    /// `Batal` (or `Esc`): disarms the confirmation.
    CardDeleteCancel,
    CardAdd(PageItemScope),
    CardAddKind(PageKind),
    /// Insert a new card (of the add-bar's chosen kind) before the card at this index.
    CardInsertBefore(usize),
    /// Insert a new card (of the add-bar's chosen kind) after the card at this index.
    CardInsertAfter(usize),
}

/// The boolean TOC card options (mirrors the reference pubgen TOC checkboxes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TocFlag {
    HangingIndent,
    IncludeSelf,
    PairBilingual,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InputMsg {
    LlmBase(String),
    LlmModel(String),
    LlmKey(String),
    GlossaryTerm(String),
    GlossaryReplacement(String),
    AiCoverTheme(String),
    ChartTitle(String),
}

#[derive(Debug, Clone)]
pub enum ComboMsg {
    CoverStyle(CoverStyle),
    ChartType(ChartType),
}

#[derive(Debug, Clone, Copy)]
pub enum ChromeMsg {
    DragTitlebar,
    ToggleMaximize,
    Minimize,
    Close,
    BeginResize(Direction),
    Maximized(bool),
}

#[derive(Debug, Clone)]
pub enum MenuMsg {
    Toggled(usize),
    Chosen(&'static str),
    /// Pointer entered a bar title while a menu is open (hover-to-switch).
    Hovered(usize),
}

/// Keyboard navigation inside the menu bar and its popovers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKey {
    /// `F10`: focus the bar, or leave it if already focused.
    FocusBar,
    Left,
    Right,
    Down,
    Up,
    /// `Enter` / `Space`: open the focused menu or activate the highlighted row.
    Activate,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    CoverStudio,
    VectorChart,
    TocGenerator,
    AiTranslation,
    Glossary,
    Diff,
    OnboardingWizard,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DialogMsg {
    // Cover Studio
    CoverStyle(CoverStyle),
    CoverIncludeBack(bool),
    CoverAddMap(bool),
    CoverMapStyle(usize),
    CoverThemePreset(usize),
    CoverTitle(String),
    CoverTitle2(String),
    CoverTitleEn(String),
    CoverYear(String),
    CoverVolume(String),
    CoverCatalogue(String),
    CoverPubNo(String),
    CoverAgency(String),
    CoverPrimaryColor(String),
    CoverAccentColor(String),
    CoverDarkColor(String),
    CoverAiTheme(String),
    CoverExportSvg,

    // Vector Chart
    ChartType(ChartType),
    ChartTitle(String),
    ChartTitleEn(String),
    ChartCategoryCol(String),
    ChartSeries1Col(String),
    ChartSeries1Label(String),
    ChartSeries2Enabled(bool),
    ChartSeries2Col(String),
    ChartSeries2Label(String),
    ChartSeries3Enabled(bool),
    ChartSeries3Col(String),
    ChartSeries3Label(String),
    ChartDataStartRow(usize),
    ChartDataEndRow(usize),
    ChartValueFormat(usize),
    ChartShowLabels(bool),
    ChartShowLegend(bool),
    ChartShowGrid(bool),
    ChartMaxVal(String),
    ChartMapPalette(usize),
    ChartRawCsv(String),
    ChartLoadSample,
    ChartGenerateSvg,
    ChartGenerateChoropleth,

    // Table of Contents
    TocTitle(String),
    TocKind(TocKind),
    TocLayout(usize),
    TocHangingIndent(bool),
    TocIncludeSelf(bool),
    TocPairBilingual(bool),
    #[allow(dead_code)]
    TocExtract(TocKind),
    #[allow(dead_code)]
    TocCopy,

    // AI Translation
    AiProvider(usize),
    AiBaseUrl(String),
    AiModel(String),
    AiApiKey(String),
    AiBilingual(bool),
    AiTranslateProject,

    // Glossary
    GlossaryFilter(String),
    GlossaryTerm(String),
    GlossaryReplacement(String),
    #[allow(dead_code)]
    GlossaryDnt(bool),
    #[allow(dead_code)]
    GlossaryAdd,
    #[allow(dead_code)]
    GlossaryRemove(usize),
    GlossaryImport,
    GlossaryExport,

    // Translation Diff
    DiffTranslate,

    // Onboarding Wizard (A/B Test Variant B)
    WizardNext,
    WizardPrev,
    WizardSelectProfile(String),
    WizardTitle(String),
    WizardYear(String),
    WizardFinish,
    WizardCancel,
}
