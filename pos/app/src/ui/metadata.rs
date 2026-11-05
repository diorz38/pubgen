// Metadata form editor & publication card grid (Phase 10).
// Replaces the legacy metadata_form::ui_bilingual and ui_publication_cards.

use iced::{
    widget::{
        button, canvas, checkbox, column, container, mouse_area, opaque, row, scrollable, text, text_input,
    },
    Element, Length, Point, Size, Theme,
};

use crate::i18n::{tr, UiLang};
use crate::metadata_form::{
    CoverField, ImageArea, Lang, MarginSide, PageItemScope, PageKind, PageLayout, Paper, TocLayout, TocStyle,
};
use crate::vector::ChartType;

use super::canvas_cache::{content_hash, CanvasCache};
use super::message::{Message, MetadataMsg};
use super::theme;
use super::widgets::combo_row;
use super::UiApp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedOption {
    pub id: String,
    pub label: String,
}

impl NamedOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

impl std::fmt::Display for NamedOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

fn selection_pill<'a>(selected: bool, label: impl Into<String>, msg: Message) -> Element<'a, Message> {
    button(
        row![
            text(if selected { "●" } else { "○" }).size(11),
            text(label.into()).size(11),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([3, 8])
    .on_press(msg)
    .style(move |theme: &Theme, _status| {
        let bg = if selected {
            theme::ACCENT.scale_alpha(0.18)
        } else {
            theme.palette().text.scale_alpha(0.04)
        };
        button::Style {
            background: Some(bg.into()),
            text_color: if selected { theme::ACCENT } else { theme.palette().text },
            border: iced::Border {
                radius: (4.0).into(),
                width: if selected { 1.2 } else { 0.8 },
                color: if selected { theme::ACCENT } else { theme.palette().text.scale_alpha(0.18) },
            },
            ..Default::default()
        }
    })
    .into()
}

/// Source option wrapper for pick_list support.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceOption {
    pub source: Option<String>,
    lang: UiLang,
}

impl SourceOption {
    fn new(source: Option<String>, lang: UiLang) -> Self {
        Self { source, lang }
    }
}

impl std::fmt::Display for SourceOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(s) => write!(f, "{s}"),
            None => f.write_str(tr(self.lang, "card_no_source")),
        }
    }
}

/// Pick-list wrapper that renders an enum using the active UI language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Localized<V: Copy> {
    value: V,
    lang: UiLang,
}

impl<V: Copy> Localized<V> {
    fn new(value: V, lang: UiLang) -> Self {
        Self { value, lang }
    }
}

impl std::fmt::Display for Localized<PageKind> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.label(self.lang))
    }
}

impl std::fmt::Display for Localized<TocStyle> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.label(self.lang))
    }
}

impl std::fmt::Display for Localized<TocLayout> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.label(self.lang))
    }
}

impl std::fmt::Display for Localized<ChartType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.label(self.lang))
    }
}

impl std::fmt::Display for Localized<PageItemScope> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.label(self.lang))
    }
}

impl std::fmt::Display for Localized<ImageArea> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.label(self.lang))
    }
}

fn localized_all<V: Copy>(all: &[V], lang: UiLang) -> Vec<Localized<V>> {
    all.iter().map(|v| Localized::new(*v, lang)).collect()
}

/// Left panel metadata editor form.
pub fn metadata_form(ui: &UiApp) -> Element<'_, Message> {
    let app = &ui.app;
    let m = &app.project.metadata;
    let lang = app.ui_lang;
    let is_brs = m.profile_id == "bps_brs" || app.selected_profile == "bps_brs";

    let mut fields = column![].spacing(6);

    if is_brs {
        fields = fields.push(meta_text_row(
            tr(lang, "meta_release_no"),
            &m.release_no,
            |s| Message::Metadata(MetadataMsg::ReleaseNo(s)),
        ));
        fields = fields.push(meta_text_row(
            tr(lang, "meta_release_date"),
            &m.release_date,
            |s| Message::Metadata(MetadataMsg::ReleaseDate(s)),
        ));
        fields = fields.push(meta_text_row(
            tr(lang, "meta_release_month"),
            &m.release_month,
            |s| Message::Metadata(MetadataMsg::ReleaseMonth(s)),
        ));
        fields = fields.push(meta_text_row(
            tr(lang, "meta_title"),
            &m.title,
            |s| Message::Metadata(MetadataMsg::Title(s)),
        ));
    } else {
        fields = fields.push(meta_text_row(
            tr(lang, "meta_title"),
            &m.title,
            |s| Message::Metadata(MetadataMsg::Title(s)),
        ));
        if m.is_bilingual() {
            fields = fields.push(meta_text_row(
                tr(lang, "meta_title_en"),
                &m.title_en,
                |s| Message::Metadata(MetadataMsg::TitleEn(s)),
            ));
        }
        fields = fields.push(meta_text_row(
            tr(lang, "meta_subtitle"),
            &m.subtitle,
            |s| Message::Metadata(MetadataMsg::Subtitle(s)),
        ));
        fields = fields.push(meta_text_row(
            tr(lang, "meta_year"),
            &m.date,
            |s| Message::Metadata(MetadataMsg::Date(s)),
        ));
        fields = fields.push(meta_text_row(
            tr(lang, "meta_volume"),
            &m.volume,
            |s| Message::Metadata(MetadataMsg::Volume(s)),
        ));
        fields = fields.push(meta_text_row(
            tr(lang, "meta_catalogue"),
            &m.catalogue,
            |s| Message::Metadata(MetadataMsg::Catalogue(s)),
        ));
        fields = fields.push(meta_text_row(
            tr(lang, "meta_code"),
            &m.code,
            |s| Message::Metadata(MetadataMsg::Code(s)),
        ));
    }

    fields = fields.push(meta_text_row(
        tr(lang, "meta_agency"),
        &m.agency,
        |s| Message::Metadata(MetadataMsg::Agency(s)),
    ));
    fields = fields.push(meta_text_row(
        tr(lang, "meta_address"),
        &m.address,
        |s| Message::Metadata(MetadataMsg::Address(s)),
    ));

    // Combo rows: Layout, Lang, Paper
    fields = fields.push(
        row![
            text(format!("{}:", tr(lang, "meta_layout"))).size(11).width(75),
            combo_row(PageLayout::ALL.to_vec(), Some(m.layout), 170.0, |l| {
                Message::Metadata(MetadataMsg::Layout(l))
            }),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    );

    fields = fields.push(
        row![
            text(format!("{}:", tr(lang, "meta_lang"))).size(11).width(75),
            combo_row(Lang::ALL.to_vec(), Some(m.lang), 170.0, |l| {
                Message::Metadata(MetadataMsg::Lang(l))
            }),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    );

    fields = fields.push(
        row![
            text(format!("{}:", tr(lang, "meta_paper"))).size(11).width(75),
            combo_row(Paper::ALL.to_vec(), Some(m.paper), 170.0, |p| {
                Message::Metadata(MetadataMsg::Paper(p))
            }),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    );

    // Margins (Top, Right, Bottom, Left)
    let margins_row_top = row![
        margin_control("T", m.margins.top, MarginSide::Top),
        margin_control("R", m.margins.right, MarginSide::Right),
    ]
    .spacing(8);

    let margins_row_bot = row![
        margin_control("B", m.margins.bottom, MarginSide::Bottom),
        margin_control("L", m.margins.left, MarginSide::Left),
    ]
    .spacing(8);

    fields = fields.push(
        column![
            text(format!("{}:", tr(lang, "meta_margins"))).size(11),
            margins_row_top,
            margins_row_bot,
        ]
        .spacing(4),
    );

    fields.into()
}

fn meta_text_row(
    label: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'static,
) -> Element<'static, Message> {
    row![
        text(format!("{label}:")).size(11).width(75),
        text_input(label, value).size(11).width(170).on_input(on_input),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}

fn margin_control(label: &'static str, val: f64, side: MarginSide) -> Element<'static, Message> {
    row![
        text(format!("{label}: {:.0}mm", val)).size(11).width(55),
        button(text("-").size(12))
            .padding([2, 6])
            .on_press(Message::Metadata(MetadataMsg::MarginStep(side, -1.0))),
        button(text("+").size(12))
            .padding([2, 6])
            .on_press(Message::Metadata(MetadataMsg::MarginStep(side, 1.0))),
    ]
    .spacing(3)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Center panel Content tab: compact publication summary cards.
/// Clicking a card opens the full editor form in a modal popup.
/// In Double-Sided mode (PageLayout::Double), items are displayed as
/// facing-page book spreads with spread numbering (e.g. 1, 2–3, 4–5, ..., 22).
/// In Single-Sided mode (PageLayout::Single), items are displayed in the 2-column grid.
pub fn publication_cards(ui: &UiApp) -> Element<'_, Message> {
    let app = &ui.app;
    let m = &app.project.metadata;
    let lang = app.ui_lang;
    let dark = theme::is_dark(app.theme, ui.system_dark);
    let is_bilingual = m.is_bilingual();

    let mut card_rows = column![].spacing(10);
    if m.pages.is_empty() {
        card_rows = card_rows.push(text(tr(lang, "card_empty")).size(12));
    } else if m.layout == PageLayout::Double {
        // Double-Sided: Facing-pages spread layout with unified summary cards
        // Row 0: Page 1 (Front Cover) alone on the left, right slot empty
        let cover = summary_card(
            0,
            &m.pages[0],
            is_bilingual,
            lang,
            dark,
            false,
            ui.editing_page == Some(0),
        );
        let empty_slot = iced::widget::Space::new().width(Length::FillPortion(1));
        card_rows = card_rows.push(row![cover, empty_slot].spacing(10));

        // Subsequent rows: Spreads (Page 2k on left, Page 2k+1 on right)
        // Left card is mirrored (thumbnail on the inside/right facing center spine)
        // Right card has thumbnail on the inside/left facing center spine
        let mut i = 1;
        while i < m.pages.len() {
            if i + 1 < m.pages.len() {
                let left = summary_card(
                    i,
                    &m.pages[i],
                    is_bilingual,
                    lang,
                    dark,
                    true,
                    ui.editing_page == Some(i),
                );
                let right = summary_card(
                    i + 1,
                    &m.pages[i + 1],
                    is_bilingual,
                    lang,
                    dark,
                    false,
                    ui.editing_page == Some(i + 1),
                );
                card_rows = card_rows.push(row![left, right].spacing(10));
            } else {
                let last = summary_card(
                    i,
                    &m.pages[i],
                    is_bilingual,
                    lang,
                    dark,
                    false,
                    ui.editing_page == Some(i),
                );
                let empty_slot = iced::widget::Space::new().width(Length::FillPortion(1));
                card_rows = card_rows.push(row![last, empty_slot].spacing(10));
            }
            i += 2;
        }
    } else {
        // Single-Sided: Standard 2-column grid of summary cards
        let mut i = 0;
        while i < m.pages.len() {
            let first = summary_card(
                i,
                &m.pages[i],
                is_bilingual,
                lang,
                dark,
                false,
                ui.editing_page == Some(i),
            );
            if i + 1 < m.pages.len() {
                let second = summary_card(
                    i + 1,
                    &m.pages[i + 1],
                    is_bilingual,
                    lang,
                    dark,
                    false,
                    ui.editing_page == Some(i + 1),
                );
                card_rows = card_rows.push(row![first, second].spacing(10));
            } else {
                let empty_slot = iced::widget::Space::new().width(Length::FillPortion(1));
                card_rows = card_rows.push(row![first, empty_slot].spacing(10));
            }
            i += 2;
        }
    }

    // Bottom add controls
    let add_bar = row![
        text(tr(lang, "card_add")).size(12),
        combo_row(
            localized_all(&PageKind::ALL, lang),
            Some(Localized::new(ui.add_page_kind, lang)),
            120.0,
            |k| Message::Metadata(MetadataMsg::CardAddKind(k.value)),
        ),
        super::widgets::tooltip(
            button(text(tr(lang, "add_new_page")).size(12))
                .padding([4, 10])
                .on_press(Message::Metadata(MetadataMsg::CardAdd(PageItemScope::NewPage))),
            tr(lang, "scope_new_page_tip"),
        ),
        super::widgets::tooltip(
            button(text(tr(lang, "add_new_object")).size(12))
                .padding([4, 10])
                .on_press(Message::Metadata(MetadataMsg::CardAdd(PageItemScope::NewObject))),
            tr(lang, "scope_new_object_tip"),
        ),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let summary = text(format!(
        "{} ({} {})",
        tr(lang, "meta_pages"),
        m.pages.len(),
        if m.pages.len() == 1 { "item" } else { "items" }
    ))
    .size(13);

    scrollable(
        column![summary, card_rows, add_bar]
            .spacing(14)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// A spread representation in double-sided facing-pages layout.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct SpreadItem {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub label: String,
}

/// Computes the facing-pages spreads for a publication.
/// Page 1 (index 0) is on the right side of the spread (Spread 0: [None, Some(0)], label "1").
/// Subsequent pages are paired side-by-side: Spread 1: [Some(1), Some(2)], label "2–3", etc.
#[allow(dead_code)]
pub fn compute_spreads(page_count: usize) -> Vec<SpreadItem> {
    if page_count == 0 {
        return Vec::new();
    }
    let mut spreads = Vec::new();

    // Spread 0: Page 1 on the right, left slot is empty
    spreads.push(SpreadItem {
        left: None,
        right: Some(0),
        label: "1".into(),
    });

    // Subsequent spreads: 2-3, 4-5, etc.
    let mut i = 1;
    while i < page_count {
        let left_idx = i;
        let right_idx = if i + 1 < page_count {
            Some(i + 1)
        } else {
            None
        };

        let label = match right_idx {
            Some(r) => format!("{}–{}", left_idx + 1, r + 1),
            None => format!("{}", left_idx + 1),
        };

        spreads.push(SpreadItem {
            left: Some(left_idx),
            right: right_idx,
            label,
        });

        i += 2;
    }

    spreads
}

pub(crate) fn kind_color(kind: PageKind) -> iced::Color {
    match kind {
        PageKind::Cover => iced::Color::from_rgb8(194, 24, 91),
        PageKind::Paragraph => iced::Color::from_rgb8(0, 137, 123),
        PageKind::Table => iced::Color::from_rgb8(230, 81, 0),
        PageKind::Chart => iced::Color::from_rgb8(25, 118, 210),
        PageKind::Image => iced::Color::from_rgb8(104, 58, 183),
        PageKind::Toc => iced::Color::from_rgb8(63, 81, 181),
    }
}

fn kind_icon(kind: PageKind) -> &'static str {
    match kind {
        PageKind::Cover => "📕",
        PageKind::Paragraph => "📝",
        PageKind::Table => "📊",
        PageKind::Chart => "📈",
        PageKind::Image => "🖼",
        PageKind::Toc => "📑",
    }
}

/// Compact card shown in the grid.
/// In double-sided spread mode, `is_left` mirrors the thumbnail to the right side
/// so that thumbnails face each other along the book spine.
fn summary_card(
    index: usize,
    entry: &crate::metadata_form::PageEntry,
    is_bilingual: bool,
    lang: UiLang,
    dark: bool,
    is_left: bool,
    is_selected: bool,
) -> Element<'static, Message> {
    let accent = kind_color(entry.kind);
    let text_color = if dark {
        iced::Color::from_rgb8(228, 231, 236)
    } else {
        iced::Color::from_rgb8(24, 28, 35)
    };
    let muted = theme::muted(dark);

    let title = if entry.title.trim().is_empty() {
        tr(lang, "card_untitled").to_string()
    } else {
        entry.title.trim().to_string()
    };

    let mut info = column![].spacing(3);
    if is_left {
        info = info.push(
            row![
                text(entry.scope.label(lang)).size(11).color(muted),
                text(format!("{} {}", entry.kind.label(lang), kind_icon(entry.kind)))
                    .size(11)
                    .color(accent),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
    } else {
        info = info.push(
            row![
                text(format!("{} {}", kind_icon(entry.kind), entry.kind.label(lang)))
                    .size(11)
                    .color(accent),
                text(entry.scope.label(lang)).size(11).color(muted),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );
    }

    info = info.push(text(title).size(13).color(text_color));
    if is_bilingual && !entry.title_en.trim().is_empty() {
        info = info.push(text(entry.title_en.trim().to_string()).size(11).color(muted));
    }

    // Compact value summary — text only, no editable widgets.
    let mut facts: Vec<String> = Vec::new();
    if !entry.chapter_id.trim().is_empty() {
        facts.push(format!("{} {}", tr(lang, "card_chapter"), entry.chapter_id.trim()));
    }
    match entry.kind {
        PageKind::Chart => facts.push(entry.chart_type.unwrap_or_default().label(lang).to_string()),
        PageKind::Image => facts.push(entry.image_area.unwrap_or_default().label(lang).to_string()),
        PageKind::Toc => {
            if let Some(t) = &entry.toc {
                facts.push(t.style.label(lang).to_string());
                if !t.title.trim().is_empty() {
                    facts.push(t.title.trim().to_string());
                }
            }
        }
        PageKind::Cover => {
            if let Some(c) = &entry.cover {
                if !c.title1.trim().is_empty() {
                    facts.push(c.title1.trim().to_string());
                }
                if !c.release_date.trim().is_empty() {
                    facts.push(c.release_date.trim().to_string());
                }
                if !c.pub_office.trim().is_empty() {
                    facts.push(c.pub_office.trim().to_string());
                }
            }
        }
        _ => {}
    }
    facts.push(match &entry.source {
        Some(src) if !src.trim().is_empty() => {
            format!("{} {}", tr(lang, "card_source"), src.trim())
        }
        _ => tr(lang, "card_no_source").to_string(),
    });
    info = info.push(text(facts.join("  ·  ")).size(11).color(muted));

    if is_left {
        info = info.push(
            text(format!(
                "{}  ·  {} {}",
                tr(lang, "card_page_hint"),
                tr(lang, "page_prefix"),
                index + 1
            ))
            .size(11)
            .color(accent),
        );
        info = info.align_x(iced::Alignment::End);
    } else {
        info = info.push(
            text(format!(
                "{} {}  ·  {}",
                tr(lang, "page_prefix"),
                index + 1,
                tr(lang, "card_page_hint")
            ))
            .size(11)
            .color(accent),
        );
        info = info.align_x(iced::Alignment::Start);
    }

    let thumb = canvas(PageThumb {
        kind: entry.kind,
        dark,
    })
    .width(78)
    .height(104);

    let body = if is_left {
        row![info.width(Length::Fill), thumb]
            .spacing(12)
            .align_y(iced::Alignment::Center)
    } else {
        row![thumb, info.width(Length::Fill)]
            .spacing(12)
            .align_y(iced::Alignment::Center)
    };

    button(body)
        .padding(10)
        .width(Length::FillPortion(1))
        .on_press(Message::Metadata(MetadataMsg::CardOpen(index)))
        .style(move |theme: &Theme, status| {
            let hovered = matches!(
                status,
                button::Status::Hovered | button::Status::Pressed
            );
            let bg = if hovered {
                if dark {
                    iced::Color::from_rgb8(45, 49, 58)
                } else {
                    iced::Color::from_rgb8(248, 250, 253)
                }
            } else if dark {
                iced::Color::from_rgb8(34, 37, 44)
            } else {
                iced::Color::WHITE
            };
            let border_color = if is_selected {
                iced::Color::from_rgb8(230, 81, 0)
            } else if hovered {
                accent
            } else {
                accent.scale_alpha(0.35)
            };
            let border_w = if is_selected {
                2.0
            } else if hovered {
                1.5
            } else {
                1.0
            };
            button::Style {
                background: Some(bg.into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    radius: (8.0).into(),
                    width: border_w,
                    color: border_color,
                },
                ..Default::default()
            }
        })
        .into()
}

// ---- Type-specific thumbnail ----

struct PageThumb {
    kind: PageKind,
    dark: bool,
}

impl canvas::Program<Message> for PageThumb {
    type State = CanvasCache;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let key = content_hash(&(self.kind, self.dark));
        state.draw(renderer, bounds, key, |frame| {
            thumb_frame(frame, self.kind, self.dark, bounds)
        })
    }
}

fn thumb_frame(
    frame: &mut canvas::Frame,
    kind: PageKind,
    dark: bool,
    bounds: iced::Rectangle,
) {
    use iced::widget::canvas::{Path, Stroke};

    let w = bounds.width;
    let h = bounds.height;
    let paper_w = (w - 4.0).max(1.0);
    let paper_h = (h - 4.0).max(1.0);
    let accent = kind_color(kind);
    let paper = iced::Color::from_rgb8(250, 251, 253);
    let border = if dark {
        iced::Color::from_rgb8(100, 105, 115)
    } else {
        iced::Color::from_rgb8(200, 205, 212)
    };
    let ink = if dark {
        iced::Color::from_rgb8(160, 166, 176)
    } else {
        iced::Color::from_rgb8(196, 202, 210)
    };

    // Page shadow + paper.
    frame.fill_rectangle(
        Point::new(3.0, 3.0),
        Size::new(paper_w, paper_h),
        iced::Color::from_rgba(0.0, 0.0, 0.0, 0.20),
    );
    frame.fill_rectangle(Point::ORIGIN, Size::new(paper_w, paper_h), paper);
    frame.stroke_rectangle(
        Point::ORIGIN,
        Size::new(paper_w, paper_h),
        Stroke::default().with_color(border).with_width(1.0),
    );

    let pad = 9.0;
    let inner_w = (paper_w - pad * 2.0).max(1.0);
    frame.fill_rectangle(Point::new(pad, pad), Size::new(inner_w, 5.0), accent);

    let body_y = 24.0;
    let body_h = (paper_h - body_y - 10.0).max(1.0);

    match kind {
        PageKind::Paragraph => {
            for i in 0..5 {
                let y = body_y + i as f32 * 12.0;
                let line_w = inner_w
                    * if i == 4 {
                        0.55
                    } else if i % 2 == 0 {
                        1.0
                    } else {
                        0.82
                    };
                frame.fill_rectangle(Point::new(pad, y), Size::new(line_w, 2.6), ink);
            }
        }
        PageKind::Table => {
            let rows = 4;
            let cols = 3;
            let row_h = body_h / rows as f32;
            let col_w = inner_w / cols as f32;

            frame.fill_rectangle(
                Point::new(pad, body_y),
                Size::new(inner_w, row_h),
                accent.scale_alpha(0.22),
            );
            for r in 0..=rows {
                let y = body_y + r as f32 * row_h;
                frame.stroke(
                    &Path::line(Point::new(pad, y), Point::new(pad + inner_w, y)),
                    Stroke::default().with_color(ink).with_width(0.8),
                );
            }
            for c in 0..=cols {
                let x = pad + c as f32 * col_w;
                frame.stroke(
                    &Path::line(Point::new(x, body_y), Point::new(x, body_y + body_h)),
                    Stroke::default().with_color(ink).with_width(0.8),
                );
            }
        }
        PageKind::Chart => {
            let base = body_y + body_h;
            frame.stroke(
                &Path::line(Point::new(pad, base), Point::new(pad + inner_w, base)),
                Stroke::default().with_color(ink).with_width(1.0),
            );
            let vals = [0.35f32, 0.62, 0.48, 0.78];
            let gap = inner_w / (vals.len() as f32 * 1.7);
            let bar_w = gap * 0.8;
            for (i, v) in vals.iter().enumerate() {
                let x = pad + gap * 0.5 + i as f32 * gap;
                let bar_h = body_h * v;
                frame.fill_rectangle(
                    Point::new(x, base - bar_h),
                    Size::new(bar_w, bar_h),
                    accent,
                );
            }
        }
        PageKind::Image => {
            let image = iced::Rectangle::new(Point::new(pad, body_y), Size::new(inner_w, body_h));
            frame.fill_rectangle(
                image.position(),
                image.size(),
                accent.scale_alpha(0.18),
            );
            frame.stroke_rectangle(
                image.position(),
                image.size(),
                Stroke::default().with_color(accent.scale_alpha(0.7)).with_width(0.8),
            );
            let path = Path::new(|p| {
                p.move_to(Point::new(pad + inner_w * 0.12, body_y + body_h * 0.82));
                p.line_to(Point::new(pad + inner_w * 0.40, body_y + body_h * 0.40));
                p.line_to(Point::new(pad + inner_w * 0.60, body_y + body_h * 0.66));
                p.line_to(Point::new(pad + inner_w * 0.76, body_y + body_h * 0.50));
                p.line_to(Point::new(pad + inner_w * 0.90, body_y + body_h * 0.82));
                p.close();
            });
            frame.fill(&path, accent.scale_alpha(0.55));
            frame.fill(
                &Path::circle(Point::new(pad + inner_w * 0.78, body_y + body_h * 0.20), 3.0),
                accent,
            );
        }
        PageKind::Cover => {
            let band_h = body_h * 0.42;
            frame.fill_rectangle(
                Point::new(pad, body_y),
                Size::new(inner_w, band_h),
                accent,
            );
            frame.fill_rectangle(
                Point::new(pad, body_y + band_h + 5.0),
                Size::new(inner_w * 0.78, 2.8),
                ink,
            );
            frame.fill_rectangle(
                Point::new(pad, body_y + band_h + 12.0),
                Size::new(inner_w * 0.55, 2.8),
                ink,
            );
            frame.fill_rectangle(
                Point::new(pad, body_y + band_h + 19.0),
                Size::new(inner_w * 0.40, 2.8),
                accent.scale_alpha(0.65),
            );
        }
        PageKind::Toc => {
            let mut y = body_y;
            for i in 0..5 {
                if i == 2 {
                    y += 4.0; // breathing gap after the title
                }
                let title_w = inner_w * if i == 0 { 0.62 } else { (0.7 - i as f32 * 0.08).max(0.3) };
                frame.fill_rectangle(Point::new(pad, y), Size::new(title_w, 2.4), ink);
                // Leader dots toward the page number.
                let dots_x = (pad + title_w + 8.0).min(pad + inner_w - 18.0);
                let dots_end = pad + inner_w - 16.0;
                let mut dx = dots_x;
                while dx < dots_end {
                    frame.fill_rectangle(
                        Point::new(dx, y + 0.9),
                        Size::new(2.0, 1.0),
                        accent.scale_alpha(0.5),
                    );
                    dx += 5.0;
                }
                frame.fill_rectangle(
                    Point::new(dots_end - 4.0, y),
                    Size::new(7.0, 2.4),
                    accent,
                );
                y += 11.0;
            }
        }
    }
}

/// Modal editor popup for one publication card. Returns `None` when no card
/// is currently being edited.
pub fn card_editor_modal(ui: &UiApp) -> Option<Element<'_, Message>> {
    let index = ui.editing_page?;
    let app = &ui.app;
    let lang = app.ui_lang;
    let entry = app.project.metadata.pages.get(index)?;
    let dark = theme::is_dark(app.theme, ui.system_dark);
    let is_bilingual = app.project.metadata.is_bilingual();
    let accent = kind_color(entry.kind);
    let muted = theme::muted(dark);

    let mut source_options = vec![SourceOption::new(None, lang)];
    for (name, _) in &app.project.content {
        source_options.push(SourceOption::new(Some(name.clone()), lang));
    }

    let form = card_form(index, entry, is_bilingual, &source_options, lang, ui.card_cover_tab, dark);

    let header = row![
        text(kind_icon(entry.kind)).size(17),
        column![
            text(format!(
                "{} {} — {} {}",
                tr(lang, "action_edit"),
                entry.kind.label(lang),
                tr(lang, "page_prefix"),
                index + 1
            ))
            .size(15),
            text(tr(lang, "modal_subtitle")).size(11).color(muted),
        ]
        .spacing(1)
        .width(Length::Fill),
        super::widgets::tooltip(
            button(text("×").size(16))
                .padding([2, 8])
                .on_press(Message::Metadata(MetadataMsg::CardClose))
                .style(|theme: &Theme, status| {
                    let hovered = matches!(status, button::Status::Hovered);
                    button::Style {
                        background: Some(
                            if hovered {
                                iced::Color::from_rgba(0.9, 0.2, 0.2, 0.18)
                            } else {
                                iced::Color::TRANSPARENT
                            }
                            .into(),
                        ),
                        text_color: theme.palette().text,
                        border: iced::Border::default(),
                        ..Default::default()
                    }
                }),
            tr(lang, "tip_close"),
        ),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    // G10: `Hapus` arms an inline confirmation; only `Ya, hapus` mutates.
    let delete_slot: Element<'static, Message> = if ui.confirm_delete {
        row![
            text(tr(lang, "confirm_delete_question"))
                .size(12)
                .color(iced::Color::from_rgb8(220, 65, 65)),
            super::widgets::tooltip(
                button(text(tr(lang, "action_confirm_delete")).size(12))
                    .padding([5, 12])
                    .on_press(Message::Metadata(MetadataMsg::CardRemove(index)))
                    .style(|_theme: &Theme, status| {
                        let hovered = matches!(status, button::Status::Hovered);
                        button::Style {
                            background: Some(
                                iced::Color::from_rgba(
                                    0.9,
                                    0.2,
                                    0.2,
                                    if hovered { 0.9 } else { 0.8 },
                                )
                                .into(),
                            ),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                radius: (6.0).into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
                tr(lang, "tip_delete"),
            ),
            super::widgets::tooltip(
                button(text(tr(lang, "action_cancel")).size(12))
                    .padding([5, 12])
                    .on_press(Message::Metadata(MetadataMsg::CardDeleteCancel))
                    .style(|theme: &Theme, status| {
                        let hovered = matches!(status, button::Status::Hovered);
                        button::Style {
                            background: Some(
                                if hovered {
                                    theme.palette().text.scale_alpha(0.1)
                                } else {
                                    iced::Color::TRANSPARENT
                                }
                                .into(),
                            ),
                            text_color: theme.palette().text,
                            border: iced::Border {
                                radius: (6.0).into(),
                                width: 1.0,
                                color: theme.palette().text.scale_alpha(0.3),
                            },
                            ..Default::default()
                        }
                    }),
                tr(lang, "tip_cancel_delete"),
            ),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        super::widgets::tooltip(
            button(text(tr(lang, "action_delete")).size(12))
                .padding([5, 12])
                .on_press(Message::Metadata(MetadataMsg::CardDeleteRequest))
                .style(|_theme: &Theme, status| {
                    let hovered = matches!(status, button::Status::Hovered);
                    button::Style {
                        background: Some(
                            if hovered {
                                iced::Color::from_rgba(0.9, 0.2, 0.2, 0.22)
                            } else {
                                iced::Color::from_rgba(0.9, 0.2, 0.2, 0.08)
                            }
                            .into(),
                        ),
                        text_color: iced::Color::from_rgb8(220, 65, 65),
                        border: iced::Border {
                            radius: (6.0).into(),
                            width: 1.0,
                            color: iced::Color::from_rgba(0.9, 0.2, 0.2, 0.35),
                        },
                        ..Default::default()
                    }
                }),
            tr(lang, "tip_delete"),
        )
    };

    // Insert-relative actions: new card of the add-bar's chosen kind, before/after
    // the card being edited. The editor jumps straight to the inserted card.
    let insert_before = super::widgets::tooltip(
        button(text(tr(lang, "insert_before")).size(12))
            .padding([5, 10])
            .on_press(Message::Metadata(MetadataMsg::CardInsertBefore(index)))
            .style(move |theme: &Theme, status| {
                let hovered = matches!(status, button::Status::Hovered);
                button::Style {
                    background: Some(
                        if hovered {
                            theme.palette().text.scale_alpha(0.1)
                        } else {
                            iced::Color::TRANSPARENT
                        }
                        .into(),
                    ),
                    text_color: theme.palette().text,
                    border: iced::Border {
                        radius: (6.0).into(),
                        width: 1.0,
                        color: accent.scale_alpha(0.5),
                    },
                    ..Default::default()
                }
            }),
        tr(lang, "tip_insert_before"),
    );
    let insert_after = super::widgets::tooltip(
        button(text(tr(lang, "insert_after")).size(12))
            .padding([5, 10])
            .on_press(Message::Metadata(MetadataMsg::CardInsertAfter(index)))
            .style(move |theme: &Theme, status| {
                let hovered = matches!(status, button::Status::Hovered);
                button::Style {
                    background: Some(
                        if hovered {
                            theme.palette().text.scale_alpha(0.1)
                        } else {
                            iced::Color::TRANSPARENT
                        }
                        .into(),
                    ),
                    text_color: theme.palette().text,
                    border: iced::Border {
                        radius: (6.0).into(),
                        width: 1.0,
                        color: accent.scale_alpha(0.5),
                    },
                    ..Default::default()
                }
            }),
        tr(lang, "tip_insert_after"),
    );

    let footer = row![
        delete_slot,
        iced::widget::space().width(Length::Fill),
        insert_before,
        insert_after,
        super::widgets::tooltip(
            button(text(tr(lang, "action_close")).size(12))
                .padding([5, 18])
                .on_press(Message::Metadata(MetadataMsg::CardClose))
                .style(move |_theme: &Theme, status| {
                    let hovered = matches!(status, button::Status::Hovered);
                    button::Style {
                        background: Some(
                            if hovered {
                                theme::lighten(accent, 0.12)
                            } else {
                                accent
                            }
                            .into(),
                        ),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            radius: (6.0).into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
            tr(lang, "tip_close"),
        ),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let dialog = container(
        column![
            header,
            iced::widget::rule::horizontal(1),
            scrollable(container(form).padding([2, 4])).height(Length::Fill),
            iced::widget::rule::horizontal(1),
            footer,
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .max_width(660.0)
    .height(Length::Fill)
    .max_height(760.0)
    .padding(14)
    .style(move |_theme: &Theme| container::Style {
        background: Some(
            (if dark {
                iced::Color::from_rgb8(30, 33, 40)
            } else {
                iced::Color::WHITE
            })
            .into(),
        ),
        border: iced::Border {
            radius: (10.0).into(),
            width: 1.0,
            color: accent.scale_alpha(0.5),
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, if dark { 0.55 } else { 0.28 }),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    });

    let centered = container(opaque(dialog))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    // Dimmed, click-to-dismiss backdrop behind the dialog.
    let backdrop = mouse_area(
        container(iced::widget::space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.45).into()),
                ..Default::default()
            }),
    )
    .on_press(Message::Metadata(MetadataMsg::CardClose));

    Some(
        iced::widget::stack![backdrop, centered]
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    )
}

/// Full editable form used inside the popup.
fn card_form<'a>(
    index: usize,
    entry: &'a crate::metadata_form::PageEntry,
    is_bilingual: bool,
    source_options: &[SourceOption],
    lang: UiLang,
    cover_tab: crate::ui::CardCoverTab,
    dark: bool,
) -> Element<'a, Message> {
    let mut card_col = column![].spacing(10).width(Length::Fill);
    let field_label = |label: &'static str| text(label).size(11);

    // Scope + kind
    let scope_tip = match entry.scope {
        PageItemScope::NewPage => tr(lang, "scope_new_page_tip"),
        PageItemScope::NewObject => tr(lang, "scope_new_object_tip"),
    };
    let scope_combo = combo_row(
        localized_all(&PageItemScope::ALL, lang),
        Some(Localized::new(entry.scope, lang)),
        210.0,
        move |sc| Message::Metadata(MetadataMsg::CardScope(index, sc.value)),
    );

    card_col = card_col.push(
        row![
            column![
                field_label(tr(lang, "modal_scope")),
                super::widgets::tooltip(scope_combo, scope_tip),
                text(scope_tip).size(10).color(theme::muted(dark)),
            ]
            .spacing(3),
            column![
                field_label(tr(lang, "modal_kind")),
                combo_row(
                    localized_all(&PageKind::ALL, lang),
                    Some(Localized::new(entry.kind, lang)),
                    190.0,
                    move |k| Message::Metadata(MetadataMsg::CardKind(index, k.value)),
                ),
            ]
            .spacing(3),
        ]
        .spacing(16),
    );

    // Title
    card_col = card_col.push(
        column![
            field_label(tr(lang, "modal_title_id")),
            text_input(tr(lang, "modal_title_placeholder"), &entry.title)
                .size(12)
                .width(Length::Fill)
                .on_input(move |s| Message::Metadata(MetadataMsg::CardTitle(index, s))),
        ]
        .spacing(3),
    );

    if is_bilingual {
        card_col = card_col.push(
            column![
                field_label(tr(lang, "modal_title_en")),
                text_input(tr(lang, "modal_title_en"), &entry.title_en)
                    .size(12)
                    .width(Length::Fill)
                    .on_input(move |s| Message::Metadata(MetadataMsg::CardTitleEn(index, s))),
            ]
            .spacing(3),
        );
    }

    // Chapter ID applies to textual / data pages.
    let show_chapter_row = match entry.kind {
        PageKind::Paragraph => entry.has_title,
        PageKind::Table | PageKind::Chart => true,
        PageKind::Cover | PageKind::Image | PageKind::Toc => false,
    };
    if show_chapter_row {
        card_col = card_col.push(
            column![
                field_label(tr(lang, "modal_chapter")),
                text_input(tr(lang, "card_chapter"), &entry.chapter_id)
                    .size(12)
                    .width(140)
                    .on_input(move |s| Message::Metadata(MetadataMsg::CardChapterId(index, s))),
            ]
            .spacing(3),
        );
    }

    // Content source
    card_col = card_col.push(
        column![
            field_label(tr(lang, "modal_source")),
            combo_row(
                source_options.to_vec(),
                Some(SourceOption::new(entry.source.clone(), lang)),
                320.0,
                move |opt| Message::Metadata(MetadataMsg::CardSource(index, opt.source)),
            ),
        ]
        .spacing(3),
    );

    // Kind-specific options
    match entry.kind {
        PageKind::Cover => {
            let c = entry.cover.as_ref().cloned().unwrap_or_default();

            // Cover Tab Switcher: "Edit Cover" and "Cover Detail"
            let tab_btn = |tab: crate::ui::CardCoverTab, label: &'static str, active: bool| {
                button(text(label).size(12))
                    .padding([5, 14])
                    .on_press(Message::Metadata(MetadataMsg::CardCoverTab(tab)))
                    .style(move |theme: &Theme, status| {
                        let hovered = matches!(status, button::Status::Hovered);
                        let bg = if active {
                            theme::ACCENT.scale_alpha(0.22)
                        } else if hovered {
                            theme.palette().text.scale_alpha(0.08)
                        } else {
                            iced::Color::TRANSPARENT
                        };
                        button::Style {
                            background: Some(bg.into()),
                            text_color: if active { theme::ACCENT } else { theme.palette().text },
                            border: iced::Border {
                                radius: (6.0).into(),
                                width: if active { 1.5 } else { 1.0 },
                                color: if active { theme::ACCENT } else { theme.palette().text.scale_alpha(0.2) },
                            },
                            ..Default::default()
                        }
                    })
            };

            let cover_tab_bar = row![
                tab_btn(crate::ui::CardCoverTab::EditCover, tr(lang, "tab_edit_cover"), cover_tab == crate::ui::CardCoverTab::EditCover),
                tab_btn(crate::ui::CardCoverTab::CoverDetail, tr(lang, "tab_cover_detail"), cover_tab == crate::ui::CardCoverTab::CoverDetail),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center);

            let tab_content: Element<'a, Message> = match cover_tab {
                crate::ui::CardCoverTab::EditCover => {
                    // 1. Gaya & Tata Letak Sampul
                    let tpl_options: Vec<NamedOption> = crate::metadata_form::CANONICAL_COVER_TEMPLATES
                        .iter()
                        .map(|(path, name, _)| NamedOption::new(*path, *name))
                        .collect();
                    let current_tpl = if c.template_path.is_empty() {
                        tpl_options.first().cloned()
                    } else {
                        tpl_options.iter().find(|o| o.id == c.template_path).cloned()
                            .or_else(|| tpl_options.first().cloned())
                    };

                    let map_style_row = row![
                        text(tr(lang, "cover_map_style")).size(11).width(80),
                        selection_pill(c.map_style == 0, "Watermark", Message::Metadata(MetadataMsg::CardCoverMapStyle(index, 0))),
                        selection_pill(c.map_style == 1, "Accent Hero", Message::Metadata(MetadataMsg::CardCoverMapStyle(index, 1))),
                        selection_pill(c.map_style == 2, "Outlines", Message::Metadata(MetadataMsg::CardCoverMapStyle(index, 2))),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center);

                    let sec1 = column![
                        text(tr(lang, "cover_style_layout")).size(12).color(theme::ACCENT),
                        row![
                            text(format!("{}:", tr(lang, "cover_template"))).size(11).width(120),
                            combo_row(
                                tpl_options,
                                current_tpl,
                                280.0,
                                move |o| Message::Metadata(MetadataMsg::CardCoverTemplate(index, o.id)),
                            ),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),

                        checkbox(c.include_inside_back)
                            .label(tr(lang, "cover_inside_back"))
                            .size(13)
                            .on_toggle(move |b| Message::Metadata(MetadataMsg::CardCoverIncludeInsideBack(index, b))),

                        checkbox(c.add_map)
                            .label(tr(lang, "cover_add_map"))
                            .size(13)
                            .on_toggle(move |b| Message::Metadata(MetadataMsg::CardCoverAddMap(index, b))),

                        if c.add_map {
                            container(map_style_row).padding([2, 10])
                        } else {
                            container(column![])
                        },
                    ]
                    .spacing(6);

                    // 2. Palet Warna & Tema Visual
                    let theme_presets = [
                        "Classic BPS (Blue/Gold)",
                        "Corporate Navy & Amber",
                        "Emerald & Gold",
                        "Crimson & Slate",
                        "Modern Teal & Cyan",
                        "Modern Violet & Slate",
                        "Report Navy & Orange",
                    ];
                    let mut preset_buttons = row![text(tr(lang, "cover_theme_preset")).size(11).width(90)].spacing(6).align_y(iced::Alignment::Center);
                    for (p_idx, p_name) in theme_presets.iter().take(4).enumerate() {
                        let btn = button(text(*p_name).size(10))
                            .padding([3, 7])
                            .on_press(Message::Metadata(MetadataMsg::CardCoverPalettePreset(index, p_idx)))
                            .style(|_theme: &Theme, _status| {
                                button::Style {
                                    background: Some(theme::ACCENT.scale_alpha(0.12).into()),
                                    text_color: theme::ACCENT,
                                    border: iced::Border {
                                        radius: (4.0).into(),
                                        width: 1.0,
                                        color: theme::ACCENT.scale_alpha(0.3),
                                    },
                                    ..Default::default()
                                }
                            });
                        preset_buttons = preset_buttons.push(btn);
                    }

                    let hex_row = row![
                        text(tr(lang, "cover_hex_colors")).size(11).width(90),
                        text_input("Palette / Hex", &c.palette)
                            .size(11)
                            .width(220)
                            .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::Palette, s))),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center);

                    let sec2 = column![
                        text(tr(lang, "cover_palette_theme")).size(12).color(theme::ACCENT),
                        preset_buttons,
                        hex_row,
                    ]
                    .spacing(6);

                    // 3. Tipografi & Teks Sampul
                    let sec3 = column![
                        text(tr(lang, "cover_typography")).size(12).color(theme::ACCENT),
                        row![
                            text(tr(lang, "cover_title_main")).size(11).width(110),
                            text_input("Judul Utama", &c.title1)
                                .size(11)
                                .width(Length::Fill)
                                .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::Title, s))),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),

                        row![
                            text(tr(lang, "cover_title_sub")).size(11).width(110),
                            text_input("Sub-Judul (Title 2)", &c.title2)
                                .size(11)
                                .width(Length::Fill)
                                .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::Title2, s))),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),

                        row![
                            text(tr(lang, "cover_title_en_label")).size(11).width(110),
                            text_input("Judul Bahasa Inggris (Title EN)", &c.title_en)
                                .size(11)
                                .width(Length::Fill)
                                .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::TitleEn, s))),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),

                        row![
                            column![
                                text(tr(lang, "cover_year_label")).size(10),
                                text_input("Tahun", &c.year)
                                    .size(11)
                                    .width(70)
                                    .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::Year, s))),
                            ].spacing(2),
                            column![
                                text(tr(lang, "cover_vol_label")).size(10),
                                text_input("Volume", &c.volume)
                                    .size(11)
                                    .width(70)
                                    .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::Volume, s))),
                            ].spacing(2),
                            column![
                                text(tr(lang, "cover_cat_label")).size(10),
                                text_input("Katalog", &c.catalogue)
                                    .size(11)
                                    .width(110)
                                    .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::Catalogue, s))),
                            ].spacing(2),
                            column![
                                text(tr(lang, "cover_pub_no_label")).size(10),
                                text_input("No. Publikasi", &c.code)
                                    .size(11)
                                    .width(110)
                                    .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::Code, s))),
                            ].spacing(2),
                        ]
                        .spacing(10)
                        .align_y(iced::Alignment::Center),

                        row![
                            text(tr(lang, "cover_agency_label")).size(11).width(110),
                            text_input("Penerbit / Agency", &c.agency)
                                .size(11)
                                .width(Length::Fill)
                                .on_input(move |s| Message::Metadata(MetadataMsg::CardCover(index, CoverField::Agency, s))),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                    ]
                    .spacing(6);

                    // 4. Generator Prompt Sampul AI
                    let ai_presets: Vec<NamedOption> = crate::metadata_form::AI_COVER_PRESETS
                        .iter()
                        .map(|name| NamedOption::new(*name, *name))
                        .collect();
                    let current_ai_preset = if c.ai_preset.is_empty() {
                        ai_presets.first().cloned()
                    } else {
                        ai_presets.iter().find(|o| o.id == c.ai_preset).cloned()
                            .or_else(|| ai_presets.first().cloned())
                    };

                    let ai_palettes: Vec<NamedOption> = crate::metadata_form::AI_COVER_PALETTES
                        .iter()
                        .map(|(name, c1, c2)| NamedOption::new(*name, format!("{name} ({c1} / {c2})")))
                        .collect();
                    let current_ai_palette = if c.ai_palette.is_empty() {
                        ai_palettes.first().cloned()
                    } else {
                        ai_palettes.iter().find(|o| o.id == c.ai_palette).cloned()
                            .or_else(|| ai_palettes.first().cloned())
                    };

                    let sec4 = column![
                        text(tr(lang, "cover_ai_generator")).size(12).color(theme::ACCENT),
                        row![
                            text(format!("{}:", tr(lang, "cover_ai_preset"))).size(11).width(110),
                            combo_row(
                                ai_presets,
                                current_ai_preset,
                                260.0,
                                move |o| Message::Metadata(MetadataMsg::CardCoverAiPreset(index, o.id)),
                            ),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),

                        row![
                            text(format!("{}:", tr(lang, "cover_ai_palette"))).size(11).width(110),
                            combo_row(
                                ai_palettes,
                                current_ai_palette,
                                260.0,
                                move |o| Message::Metadata(MetadataMsg::CardCoverAiPalette(index, o.id)),
                            ),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),

                        row![
                            text(format!("{}:", tr(lang, "cover_ai_theme"))).size(11).width(110),
                            text_input(tr(lang, "cover_ai_theme_placeholder"), &c.ai_theme)
                                .size(11)
                                .width(200)
                                .on_input(move |s| Message::Metadata(MetadataMsg::CardCoverAiTheme(index, s))),
                            button(text(tr(lang, "cover_copy_prompt")).size(11))
                                .padding([3, 10])
                                .on_press(Message::CoverPromptCopy)
                                .style(|_theme: &Theme, _status| {
                                    button::Style {
                                        background: Some(theme::ACCENT.into()),
                                        text_color: iced::Color::WHITE,
                                        border: iced::Border {
                                            radius: (4.0).into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }
                                }),
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                    ]
                    .spacing(6);

                    let edit_actions = row![
                        button(text(tr(lang, "export_cover_svg")).size(11))
                            .padding([4, 12])
                            .on_press(Message::DialogMsg(crate::ui::message::DialogMsg::CoverExportSvg))
                            .style(|_theme: &Theme, _status| {
                                button::Style {
                                    background: Some(theme::ACCENT.into()),
                                    text_color: iced::Color::WHITE,
                                    border: iced::Border {
                                        radius: (4.0).into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            }),
                        button(text(tr(lang, "cover_open_dialog")).size(11))
                            .padding([4, 12])
                            .on_press(Message::OpenDialog(crate::ui::DialogKind::CoverStudio))
                            .style(|theme: &Theme, _status| {
                                button::Style {
                                    background: Some(theme.palette().text.scale_alpha(0.08).into()),
                                    text_color: theme.palette().text,
                                    border: iced::Border {
                                        radius: (4.0).into(),
                                        width: 1.0,
                                        color: theme.palette().text.scale_alpha(0.2),
                                    },
                                    ..Default::default()
                                }
                            }),
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center);

                    column![sec1, sec2, sec3, sec4, edit_actions].spacing(12).into()
                }
                crate::ui::CardCoverTab::CoverDetail => {
                    let mut cover_fields = column![].spacing(6);
                    for (label, field_type, val) in [
                        (tr(lang, "meta_release_no"), CoverField::ReleaseNo, c.release_no),
                        (tr(lang, "cover_date"), CoverField::Date, c.release_date),
                        (tr(lang, "cover_title"), CoverField::Title, c.title1),
                        (tr(lang, "cover_month"), CoverField::Month, c.release_month),
                        (
                            tr(lang, "cover_pub_office"),
                            CoverField::PubOffice,
                            c.pub_office,
                        ),
                        (tr(lang, "cover_hero"), CoverField::Hero, c.image_hero),
                        (tr(lang, "cover_logo"), CoverField::Logo, c.logo),
                        (tr(lang, "cover_point"), CoverField::Point, c.point),
                        (tr(lang, "cover_qr"), CoverField::Qr, c.qr),
                        (tr(lang, "cover_pic_name"), CoverField::PicName, c.pic_name),
                        (
                            tr(lang, "cover_pic_photo"),
                            CoverField::PicPhoto,
                            c.pic_photo,
                        ),
                        (tr(lang, "cover_position"), CoverField::Position, c.position),
                        (tr(lang, "cover_office"), CoverField::Office, c.office),
                        (tr(lang, "cover_phone"), CoverField::Phone, c.phone),
                        (tr(lang, "cover_email"), CoverField::Email, c.email),
                    ] {
                        cover_fields = cover_fields.push(
                            row![
                                text(format!("{label}:")).size(11).width(110),
                                text_input(label, &val)
                                    .size(11)
                                    .width(Length::Fill)
                                    .on_input(move |s| {
                                        Message::Metadata(MetadataMsg::CardCover(index, field_type, s))
                                    }),
                            ]
                            .spacing(6)
                            .align_y(iced::Alignment::Center),
                        );
                    }
                    cover_fields.into()
                }
            };

            card_col = card_col.push(
                column![
                    cover_tab_bar,
                    container(tab_content)
                        .padding(10)
                        .style(|theme: &Theme| container::Style {
                            background: Some(theme.palette().text.scale_alpha(0.02).into()),
                            border: iced::Border {
                                radius: (6.0).into(),
                                width: 1.0,
                                color: theme.palette().text.scale_alpha(0.12),
                            },
                            ..Default::default()
                        }),
                ]
                .spacing(8),
            );
        }
        PageKind::Table => {
            let topts = entry.table_options.clone().unwrap_or_default();

            // 1. Table Rules & Ingest Options (from reference app card2_body_ui)
            let check_header = checkbox(topts.first_row_header)
                .label(tr(lang, "table_first_row_header"))
                .size(13)
                .on_toggle(move |b| Message::Metadata(MetadataMsg::CardTableFirstRowHeader(index, b)));

            let check_col_numbers = checkbox(topts.col_numbers)
                .label(tr(lang, "table_col_numbers"))
                .size(13)
                .on_toggle(move |b| Message::Metadata(MetadataMsg::CardTableColNumbers(index, b)));

            let check_footer = checkbox(topts.table_footer)
                .label(tr(lang, "table_footer_row"))
                .size(13)
                .on_toggle(move |b| Message::Metadata(MetadataMsg::CardTableFooter(index, b)));

            let check_conditions = checkbox(topts.row_conditions)
                .label(tr(lang, "table_row_conditions"))
                .size(13)
                .on_toggle(move |b| Message::Metadata(MetadataMsg::CardTableRowConditions(index, b)));

            let rules_section = column![
                text(tr(lang, "table_rules_title")).size(12).color(theme::ACCENT),
                check_header,
                check_col_numbers,
                check_footer,
                check_conditions,
            ]
            .spacing(6);

            // 2. Table Style Selector (5 styles from reference app: Basic Grid, Corporate Navy, Minimalist, Horizontal Rules, BRS6)
            let style_opts: Vec<NamedOption> = crate::metadata_form::TABLE_STYLES
                .iter()
                .map(|name| NamedOption::new(*name, *name))
                .collect();
            let current_style = if topts.style.is_empty() {
                style_opts.first().cloned()
            } else {
                style_opts.iter().find(|o| o.id == topts.style).cloned()
                    .or_else(|| style_opts.first().cloned())
            };

            // 3. Table Color Theme Selector (5 themes from reference app)
            let theme_opts: Vec<NamedOption> = crate::metadata_form::TABLE_THEMES
                .iter()
                .map(|(id, label)| NamedOption::new(*id, *label))
                .collect();
            let current_theme = if topts.theme.is_empty() {
                theme_opts.first().cloned()
            } else {
                theme_opts.iter().find(|o| o.id == topts.theme).cloned()
                    .or_else(|| theme_opts.first().cloned())
            };

            // 4. Table Layout Preset
            let layout_opts: Vec<NamedOption> = crate::metadata_form::TABLE_LAYOUT_PRESETS
                .iter()
                .map(|name| NamedOption::new(*name, *name))
                .collect();
            let current_layout = if topts.layout_preset.is_empty() {
                layout_opts.first().cloned()
            } else {
                layout_opts.iter().find(|o| o.id == topts.layout_preset).cloned()
                    .or_else(|| layout_opts.first().cloned())
            };

            let table_styles_section = column![
                text(tr(lang, "table_options_title")).size(12).color(theme::ACCENT),
                row![
                    text(tr(lang, "table_style")).size(11).width(120),
                    combo_row(
                        style_opts,
                        current_style,
                        280.0,
                        move |o| Message::Metadata(MetadataMsg::CardTableStyle(index, o.id)),
                    ),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),

                row![
                    text(tr(lang, "table_theme")).size(11).width(120),
                    combo_row(
                        theme_opts,
                        current_theme,
                        280.0,
                        move |o| Message::Metadata(MetadataMsg::CardTableTheme(index, o.id)),
                    ),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),

                row![
                    text(tr(lang, "table_layout")).size(11).width(120),
                    combo_row(
                        layout_opts,
                        current_layout,
                        280.0,
                        move |o| Message::Metadata(MetadataMsg::CardTableLayoutPreset(index, o.id)),
                    ),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(8);

            let table_actions = row![
                super::widgets::tooltip(
                    button(text(tr(lang, "table_apply_all")).size(11))
                        .padding([4, 12])
                        .on_press(Message::Metadata(MetadataMsg::CardTableApplyToAll(index)))
                        .style(|_theme: &Theme, status| {
                            let hovered = matches!(status, button::Status::Hovered);
                            button::Style {
                                background: Some(
                                    if hovered {
                                        theme::ACCENT.scale_alpha(0.25)
                                    } else {
                                        theme::ACCENT.scale_alpha(0.12)
                                    }
                                    .into(),
                                ),
                                text_color: theme::ACCENT,
                                border: iced::Border {
                                    radius: (4.0).into(),
                                    width: 1.0,
                                    color: theme::ACCENT.scale_alpha(0.4),
                                },
                                ..Default::default()
                            }
                        }),
                    tr(lang, "tip_table_apply_all"),
                ),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center);

            let table_settings = column![
                rules_section,
                iced::widget::rule::horizontal(1),
                table_styles_section,
                table_actions,
            ]
            .spacing(10);

            card_col = card_col.push(
                container(table_settings)
                    .padding(10)
                    .style(|theme: &Theme| container::Style {
                        background: Some(theme.palette().text.scale_alpha(0.02).into()),
                        border: iced::Border {
                            radius: (6.0).into(),
                            width: 1.0,
                            color: theme.palette().text.scale_alpha(0.12),
                        },
                        ..Default::default()
                    }),
            );
        }
        PageKind::Chart => {
            let ct = entry.chart_type.unwrap_or_default();
            card_col = card_col.push(
                column![
                    field_label(tr(lang, "modal_chart_type")),
                    combo_row(
                        localized_all(&ChartType::ALL, lang),
                        Some(Localized::new(ct, lang)),
                        260.0,
                        move |t| Message::Metadata(MetadataMsg::CardChartType(index, t.value)),
                    ),
                ]
                .spacing(3),
            );
        }
        PageKind::Image => {
            let ia = entry.image_area.unwrap_or_default();
            card_col = card_col.push(
                column![
                    field_label(tr(lang, "modal_image_area")),
                    combo_row(
                        localized_all(&ImageArea::ALL, lang),
                        Some(Localized::new(ia, lang)),
                        260.0,
                        move |a| Message::Metadata(MetadataMsg::CardImageArea(index, a.value)),
                    ),
                ]
                .spacing(3),
            );
        }
        PageKind::Toc => {
            let toc = entry.toc.clone().unwrap_or_default();
            // TOC title
            card_col = card_col.push(
                column![
                    field_label(tr(lang, "toc_title")),
                    text_input(tr(lang, "toc_title"), &toc.title)
                        .size(12)
                        .width(Length::Fill)
                        .on_input(move |s| {
                            Message::Metadata(MetadataMsg::CardTocTitle(index, s))
                        }),
                ]
                .spacing(3),
            );
            // TOC style + layout (reuse of the reference pubgen TOC form fields)
            card_col = card_col.push(
                row![
                    column![
                        field_label(tr(lang, "toc_style")),
                        combo_row(
                            localized_all(&TocStyle::ALL, lang),
                            Some(Localized::new(toc.style, lang)),
                            240.0,
                            move |t| Message::Metadata(MetadataMsg::CardTocStyle(index, t.value)),
                        ),
                    ]
                    .spacing(3),
                    column![
                        field_label(tr(lang, "toc_layout")),
                        combo_row(
                            localized_all(&TocLayout::ALL, lang),
                            Some(Localized::new(toc.layout, lang)),
                            160.0,
                            move |l| Message::Metadata(MetadataMsg::CardTocLayout(index, l.value)),
                        ),
                    ]
                    .spacing(3),
                ]
                .spacing(12),
            );
            // TOC options (reference pubgen checkboxes)
            card_col = card_col.push(
                column![
                    checkbox(toc.hanging_indent)
                        .label(tr(lang, "toc_hanging_indent"))
                        .on_toggle(move |b| {
                            Message::Metadata(MetadataMsg::CardTocFlag(
                                index,
                                super::message::TocFlag::HangingIndent,
                                b,
                            ))
                        }),
                    checkbox(toc.include_self)
                        .label(tr(lang, "toc_include_self"))
                        .on_toggle(move |b| {
                            Message::Metadata(MetadataMsg::CardTocFlag(
                                index,
                                super::message::TocFlag::IncludeSelf,
                                b,
                            ))
                        }),
                    checkbox(toc.pair_bilingual)
                        .label(tr(lang, "toc_pair_bilingual"))
                        .on_toggle(move |b| {
                            Message::Metadata(MetadataMsg::CardTocFlag(
                                index,
                                super::message::TocFlag::PairBilingual,
                                b,
                            ))
                        }),
                ]
                .spacing(6),
            );
        }
        _ => {}
    }

    // Numbering & Section Options (accessible when scope is NewPage)
    if entry.scope == PageItemScope::NewPage {
        let sec = entry.section.clone().unwrap_or_default();

        let num_style_opts: Vec<NamedOption> = crate::metadata_form::NUMBERING_STYLES
            .iter()
            .map(|(id, label)| NamedOption::new(*id, *label))
            .collect();
        let cur_num_style = num_style_opts.iter().find(|o| o.id == sec.numbering_style).cloned()
            .or_else(|| num_style_opts.first().cloned());

        let cur_chapter_style = num_style_opts.iter().find(|o| o.id == sec.chapter_style).cloned()
            .or_else(|| num_style_opts.first().cloned());

        let check_start_sec = checkbox(sec.start_section)
            .label(tr(lang, "section_start"))
            .size(13)
            .on_toggle(move |b| Message::Metadata(MetadataMsg::CardSectionStart(index, b)));

        let numbering_mode_row = row![
            selection_pill(
                sec.automatic_numbering,
                tr(lang, "section_auto_numbering"),
                Message::Metadata(MetadataMsg::CardSectionAutomatic(index, true)),
            ),
            selection_pill(
                !sec.automatic_numbering,
                tr(lang, "section_start_at"),
                Message::Metadata(MetadataMsg::CardSectionAutomatic(index, false)),
            ),
            text_input("1", &sec.start_page_number.to_string())
                .size(11)
                .width(60)
                .on_input(move |s| {
                    if let Ok(v) = s.parse::<u32>() {
                        Message::Metadata(MetadataMsg::CardSectionStartPage(index, v))
                    } else if s.is_empty() {
                        Message::Metadata(MetadataMsg::CardSectionStartPage(index, 1))
                    } else {
                        Message::Metadata(MetadataMsg::CardSectionStartPage(index, sec.start_page_number))
                    }
                }),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let page_num_box = column![
            text(tr(lang, "section_page_numbering")).size(11).color(theme::ACCENT),
            row![
                text(tr(lang, "section_prefix")).size(11).width(120),
                text_input("Prefix", &sec.section_prefix)
                    .size(11)
                    .width(140)
                    .on_input(move |s| Message::Metadata(MetadataMsg::CardSectionPrefix(index, s))),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),

            row![
                text(tr(lang, "section_style")).size(11).width(120),
                combo_row(
                    num_style_opts.clone(),
                    cur_num_style,
                    220.0,
                    move |o| Message::Metadata(MetadataMsg::CardSectionStyle(index, o.id)),
                ),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),

            row![
                text(tr(lang, "section_marker")).size(11).width(120),
                text_input("Marker", &sec.section_marker)
                    .size(11)
                    .width(140)
                    .on_input(move |s| Message::Metadata(MetadataMsg::CardSectionMarker(index, s))),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),

            checkbox(sec.include_prefix)
                .label(tr(lang, "section_include_prefix"))
                .size(13)
                .on_toggle(move |b| Message::Metadata(MetadataMsg::CardSectionIncludePrefix(index, b))),
        ]
        .spacing(6);

        let chapter_mode_row = row![
            selection_pill(
                sec.chapter_source == "automatic",
                tr(lang, "section_auto_chapter"),
                Message::Metadata(MetadataMsg::CardSectionChapterSource(index, "automatic".to_string())),
            ),
            selection_pill(
                sec.chapter_source == "start",
                tr(lang, "section_start_chapter_at"),
                Message::Metadata(MetadataMsg::CardSectionChapterSource(index, "start".to_string())),
            ),
            text_input("1", &sec.chapter_start.to_string())
                .size(11)
                .width(60)
                .on_input(move |s| {
                    if let Ok(v) = s.parse::<u32>() {
                        Message::Metadata(MetadataMsg::CardSectionChapterStart(index, v))
                    } else if s.is_empty() {
                        Message::Metadata(MetadataMsg::CardSectionChapterStart(index, 1))
                    } else {
                        Message::Metadata(MetadataMsg::CardSectionChapterStart(index, sec.chapter_start))
                    }
                }),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let chapter_num_box = column![
            text(tr(lang, "section_chapter_group")).size(11).color(theme::ACCENT),
            row![
                text(tr(lang, "section_chapter_style")).size(11).width(120),
                combo_row(
                    num_style_opts,
                    cur_chapter_style,
                    220.0,
                    move |o| Message::Metadata(MetadataMsg::CardSectionChapterStyle(index, o.id)),
                ),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
            chapter_mode_row,
        ]
        .spacing(6);

        let apply_forward_btn = super::widgets::tooltip(
            button(text(tr(lang, "section_apply_forward")).size(11))
                .padding([4, 12])
                .on_press(Message::Metadata(MetadataMsg::CardSectionApplyForward(index)))
                .style(|_theme: &Theme, status| {
                    let hovered = matches!(status, button::Status::Hovered);
                    button::Style {
                        background: Some(
                            if hovered {
                                theme::ACCENT.scale_alpha(0.25)
                            } else {
                                theme::ACCENT.scale_alpha(0.12)
                            }
                            .into(),
                        ),
                        text_color: theme::ACCENT,
                        border: iced::Border {
                            radius: (4.0).into(),
                            width: 1.0,
                            color: theme::ACCENT.scale_alpha(0.4),
                        },
                        ..Default::default()
                    }
                }),
            tr(lang, "tip_section_apply_forward"),
        );

        let section_settings = column![
            check_start_sec,
            numbering_mode_row,
            container(page_num_box)
                .padding(6)
                .style(|theme: &Theme| container::Style {
                    background: Some(theme.palette().text.scale_alpha(0.015).into()),
                    border: iced::Border {
                        radius: (4.0).into(),
                        width: 1.0,
                        color: theme.palette().text.scale_alpha(0.08),
                    },
                    ..Default::default()
                }),
            container(chapter_num_box)
                .padding(6)
                .style(|theme: &Theme| container::Style {
                    background: Some(theme.palette().text.scale_alpha(0.015).into()),
                    border: iced::Border {
                        radius: (4.0).into(),
                        width: 1.0,
                        color: theme.palette().text.scale_alpha(0.08),
                    },
                    ..Default::default()
                }),
            row![apply_forward_btn].spacing(8).align_y(iced::Alignment::Center),
        ]
        .spacing(8);

        card_col = card_col.push(
            column![
                text(tr(lang, "section_options_title"))
                    .size(12)
                    .color(theme::ACCENT),
                container(section_settings)
                    .padding(8)
                    .style(|theme: &Theme| container::Style {
                        background: Some(theme.palette().text.scale_alpha(0.02).into()),
                        border: iced::Border {
                            radius: (6.0).into(),
                            width: 1.0,
                            color: theme.palette().text.scale_alpha(0.12),
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(5),
        );
    }

    card_col.into()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_form::{
        CoverField, MarginSide, PageEntry, PageItemScope, PageKind,
    };

    #[test]
    fn test_source_option_display() {
        let none = SourceOption::new(None, UiLang::Id);
        assert_eq!(none.to_string(), tr(UiLang::Id, "card_no_source"));
        let none_en = SourceOption::new(None, UiLang::En);
        assert_eq!(none_en.to_string(), tr(UiLang::En, "card_no_source"));

        let some = SourceOption::new(Some("data/table.csv".into()), UiLang::Id);
        assert_eq!(some.to_string(), "data/table.csv");
    }

    #[test]
    fn test_metadata_update_margin_clamping() {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.margins.top = 99.5;

        // Step by 2 -> should clamp to MARGINS_MAX (100.0)
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::MarginStep(MarginSide::Top, 2.0)),
        );
        assert_eq!(ui.app.project.metadata.margins.top, 100.0);

        // Step down by 200 -> should clamp to MARGINS_MIN (5.0)
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::MarginStep(MarginSide::Top, -200.0)),
        );
        assert_eq!(ui.app.project.metadata.margins.top, 5.0);
    }

    #[test]
    fn test_metadata_card_crud_and_sync() {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.pages.clear();

        // 1. Add page card
        ui.add_page_kind = PageKind::Chart;
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardAdd(PageItemScope::NewPage)),
        );
        assert_eq!(ui.app.project.metadata.pages.len(), 1);
        assert_eq!(ui.app.project.metadata.pages[0].kind, PageKind::Chart);
        assert_eq!(ui.app.project.metadata.pages[0].scope, PageItemScope::NewPage);
        assert!(ui.app.project.metadata.pages[0].chart_type.is_some());

        // 2. Edit title
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardTitle(0, "Grafik 1".into())),
        );
        assert_eq!(ui.app.project.metadata.pages[0].title, "Grafik 1");

        // 3. Switch kind to Cover -> sync options
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardKind(0, PageKind::Cover)),
        );
        assert_eq!(ui.app.project.metadata.pages[0].kind, PageKind::Cover);
        assert!(ui.app.project.metadata.pages[0].chart_type.is_none());
        assert!(ui.app.project.metadata.pages[0].cover.is_some());
        assert!(ui.app.project.metadata.pages[0].toc.is_none());

        // 3b. Switch kind to Toc -> sync options
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardKind(0, PageKind::Toc)),
        );
        assert_eq!(ui.app.project.metadata.pages[0].kind, PageKind::Toc);
        assert!(ui.app.project.metadata.pages[0].cover.is_none());
        let toc = ui.app.project.metadata.pages[0].toc.as_ref().unwrap();
        assert_eq!(toc.title, "Daftar Isi");
        assert_eq!(toc.style, crate::metadata_form::TocStyle::Entry);

        // 4. Update cover field
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardCover(
                0,
                CoverField::Title,
                "Cover Title".into(),
            )),
        );
        assert_eq!(
            ui.app.project.metadata.pages[0].cover.as_ref().unwrap().title1,
            "Cover Title"
        );

        // 5. Remove card
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardRemove(0)),
        );
        assert_eq!(ui.app.project.metadata.pages.len(), 0);
    }

    #[test]
    fn test_card_editor_popup_state_tracks_open_remove_and_add() {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.pages = vec![
            PageEntry {
                title: "A".into(),
                ..Default::default()
            },
            PageEntry {
                title: "B".into(),
                ..Default::default()
            },
            PageEntry {
                title: "C".into(),
                ..Default::default()
            },
        ];

        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardOpen(1)),
        );
        assert_eq!(ui.editing_page, Some(1));

        // Removing an earlier card shifts the open index down.
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardRemove(0)),
        );
        assert_eq!(ui.editing_page, Some(0));

        // Removing the card being edited closes the popup.
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardRemove(0)),
        );
        assert_eq!(ui.editing_page, None);

        // Adding a card opens it directly for editing.
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardAdd(PageItemScope::NewPage)),
        );
        assert_eq!(ui.editing_page, Some(ui.app.project.metadata.pages.len() - 1));

        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardClose),
        );
        assert_eq!(ui.editing_page, None);
    }

    #[test]
    fn test_card_insert_before_and_after_positions() {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.pages = vec![
            PageEntry { title: "A".into(), ..Default::default() },
            PageEntry { title: "B".into(), ..Default::default() },
            PageEntry { title: "C".into(), ..Default::default() },
        ];
        ui.add_page_kind = PageKind::Toc;

        // Insert before card 1 ("B"): new card lands at 1, editor follows it.
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardInsertBefore(1)),
        );
        assert_eq!(ui.app.project.metadata.pages.len(), 4);
        assert_eq!(ui.app.project.metadata.pages[1].kind, PageKind::Toc);
        assert!(ui.app.project.metadata.pages[1].toc.is_some(), "kind options must sync on insert");
        assert_eq!(ui.editing_page, Some(1));
        assert_eq!(ui.app.project.metadata.pages[2].title, "B");
        assert_eq!(ui.app.project.metadata.pages[3].title, "C");

        // Insert after the edited card (index 1): lands at 2.
        ui.add_page_kind = PageKind::Table;
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardInsertAfter(1)),
        );
        assert_eq!(ui.app.project.metadata.pages.len(), 5);
        assert_eq!(ui.app.project.metadata.pages[2].kind, PageKind::Table);
        assert_eq!(ui.editing_page, Some(2));
        assert_eq!(ui.app.project.metadata.pages[3].title, "B");

        // Insert after the last card appends at the end.
        let last = ui.app.project.metadata.pages.len() - 1;
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardInsertAfter(last)),
        );
        assert_eq!(ui.app.project.metadata.pages.len(), 6);
        assert_eq!(ui.editing_page, Some(5));
        assert_eq!(ui.app.project.metadata.pages[5].kind, PageKind::Table);
    }

    // ---- G10 destructive-action safety ----

    fn open_with_three_cards() -> UiApp {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.pages = vec![
            PageEntry {
                title: "A".into(),
                ..Default::default()
            },
            PageEntry {
                title: "B".into(),
                ..Default::default()
            },
            PageEntry {
                title: "C".into(),
                ..Default::default()
            },
        ];
        ui.app.log.clear();
        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardOpen(1)),
        );
        ui
    }

    fn apply(ui: &mut UiApp, msg: MetadataMsg) {
        let _ = super::super::update::update(ui, Message::Metadata(msg));
    }

    #[test]
    fn first_delete_press_arms_without_mutating() {
        let mut ui = open_with_three_cards();

        apply(&mut ui, MetadataMsg::CardDeleteRequest);

        assert_eq!(ui.app.project.metadata.pages.len(), 3, "no page removed yet");
        assert!(ui.confirm_delete, "confirmation should be armed");
        assert_eq!(ui.editing_page, Some(1), "modal stays open");
        assert!(ui.app.log.is_empty(), "nothing logged before confirmation");
    }

    #[test]
    fn confirming_removes_exactly_one_page_and_logs() {
        let mut ui = open_with_three_cards();
        apply(&mut ui, MetadataMsg::CardDeleteRequest);

        apply(&mut ui, MetadataMsg::CardRemove(1));

        assert_eq!(ui.app.project.metadata.pages.len(), 2);
        assert_eq!(ui.app.project.metadata.pages[0].title, "A");
        assert_eq!(ui.app.project.metadata.pages[1].title, "C");
        assert_eq!(ui.editing_page, None, "AC10.4: modal closes");
        assert!(!ui.confirm_delete);
        assert_eq!(
            ui.app.log.last().map(String::as_str),
            Some("card removed: B (page 2)")
        );
    }

    #[test]
    fn cancelling_removes_none_and_keeps_modal() {
        let mut ui = open_with_three_cards();
        apply(&mut ui, MetadataMsg::CardDeleteRequest);

        apply(&mut ui, MetadataMsg::CardDeleteCancel);

        assert_eq!(ui.app.project.metadata.pages.len(), 3);
        assert!(!ui.confirm_delete);
        assert_eq!(ui.editing_page, Some(1));
        assert!(ui.app.log.is_empty());
    }

    #[test]
    fn esc_cancels_confirmation_before_closing_modal() {
        let mut ui = open_with_three_cards();
        apply(&mut ui, MetadataMsg::CardDeleteRequest);

        let _ = super::super::update::update(&mut ui, Message::EscapePressed);
        assert!(!ui.confirm_delete, "AC10.3: first Esc disarms");
        assert_eq!(ui.editing_page, Some(1), "modal stays open");
        assert_eq!(ui.app.project.metadata.pages.len(), 3);

        let _ = super::super::update::update(&mut ui, Message::EscapePressed);
        assert_eq!(ui.editing_page, None, "second Esc closes the modal");
    }

    #[test]
    fn confirmation_resets_on_card_switch_close_and_tab_switch() {
        let mut ui = open_with_three_cards();

        apply(&mut ui, MetadataMsg::CardDeleteRequest);
        apply(&mut ui, MetadataMsg::CardOpen(2));
        assert!(!ui.confirm_delete, "switching cards resets");

        apply(&mut ui, MetadataMsg::CardDeleteRequest);
        apply(&mut ui, MetadataMsg::CardClose);
        assert!(!ui.confirm_delete, "closing the modal resets");

        let _ = super::super::update::update(
            &mut ui,
            Message::Metadata(MetadataMsg::CardOpen(0)),
        );
        apply(&mut ui, MetadataMsg::CardDeleteRequest);
        let _ = super::super::update::update(
            &mut ui,
            Message::TabSelected(crate::main_ui::CenterTab::Preview),
        );
        assert!(!ui.confirm_delete, "switching tabs resets");
        assert_eq!(ui.editing_page, None);
        assert_eq!(ui.app.project.metadata.pages.len(), 3);
    }

    #[test]
    fn test_card_cover_tab_switching_and_updates() {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.pages = vec![PageEntry {
            kind: PageKind::Cover,
            scope: PageItemScope::NewPage,
            title: "Cover Page".into(),
            ..Default::default()
        }];

        // Initially on EditCover
        assert_eq!(ui.card_cover_tab, crate::ui::CardCoverTab::EditCover);

        // Edit Cover generator fields
        apply(&mut ui, MetadataMsg::CardCoverTemplate(0, "001_geometric_medallion.designcraft".into()));
        apply(&mut ui, MetadataMsg::CardCoverPalettePreset(0, 1));
        apply(&mut ui, MetadataMsg::CardCoverAiTheme(0, "Pertanian padi di sawah".into()));
        apply(&mut ui, MetadataMsg::CardCover(0, CoverField::Title2, "Kecamatan Dalam Angka".into()));
        apply(&mut ui, MetadataMsg::CardCover(0, CoverField::Year, "2026".into()));

        // Switch tab to CoverDetail
        apply(&mut ui, MetadataMsg::CardCoverTab(crate::ui::CardCoverTab::CoverDetail));
        assert_eq!(ui.card_cover_tab, crate::ui::CardCoverTab::CoverDetail);

        // Edit detail fields
        apply(&mut ui, MetadataMsg::CardCover(0, CoverField::PicName, "Yana Hendriana".into()));
        apply(&mut ui, MetadataMsg::CardCover(0, CoverField::Phone, "(022) 7272595".into()));

        let page = &ui.app.project.metadata.pages[0];
        let c = page.cover.as_ref().unwrap();
        assert_eq!(c.template_path, "001_geometric_medallion.designcraft");
        assert_eq!(c.title2, "Kecamatan Dalam Angka");
        assert_eq!(c.year, "2026");
        assert_eq!(c.pic_name, "Yana Hendriana");
        assert_eq!(c.phone, "(022) 7272595");

        // Metadata JSON round trip check
        let json = ui.app.project.metadata.to_json();
        let reloaded = crate::metadata_form::Metadata::from_json(&json);
        let reloaded_c = reloaded.pages[0].cover.as_ref().unwrap();
        assert_eq!(reloaded_c.template_path, "001_geometric_medallion.designcraft");
        assert_eq!(reloaded_c.pic_name, "Yana Hendriana");
    }

    #[test]
    fn test_section_options_and_apply_forward() {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.pages = vec![
            PageEntry {
                kind: PageKind::Paragraph,
                scope: PageItemScope::NewPage,
                title: "Page 1".into(),
                ..Default::default()
            },
            PageEntry {
                kind: PageKind::Table,
                scope: PageItemScope::NewPage,
                title: "Page 2".into(),
                ..Default::default()
            },
            PageEntry {
                kind: PageKind::Paragraph,
                scope: PageItemScope::NewObject,
                title: "Page 2 Item 2".into(),
                ..Default::default()
            },
            PageEntry {
                kind: PageKind::Chart,
                scope: PageItemScope::NewPage,
                title: "Page 3".into(),
                ..Default::default()
            },
        ];

        apply(&mut ui, MetadataMsg::CardSectionStart(0, true));
        apply(&mut ui, MetadataMsg::CardSectionAutomatic(0, false));
        apply(&mut ui, MetadataMsg::CardSectionStartPage(0, 5));
        apply(&mut ui, MetadataMsg::CardSectionPrefix(0, "SEC-".into()));
        apply(&mut ui, MetadataMsg::CardSectionStyle(0, "upper_roman".into()));
        apply(&mut ui, MetadataMsg::CardSectionIncludePrefix(0, true));
        apply(&mut ui, MetadataMsg::CardSectionApplyForward(0));

        let p1_sec = ui.app.project.metadata.pages[0].section.as_ref().unwrap();
        assert_eq!(p1_sec.start_page_number, 5);
        assert_eq!(p1_sec.section_prefix, "SEC-");
        assert_eq!(p1_sec.numbering_style, "upper_roman");

        let p2_sec = ui.app.project.metadata.pages[1].section.as_ref().unwrap();
        assert_eq!(p2_sec.start_page_number, 5);
        assert_eq!(p2_sec.section_prefix, "SEC-");

        assert!(ui.app.project.metadata.pages[2].section.is_none());

        let p3_sec = ui.app.project.metadata.pages[3].section.as_ref().unwrap();
        assert_eq!(p3_sec.section_prefix, "SEC-");
        assert_eq!(p3_sec.numbering_style, "upper_roman");
    }

    #[test]
    fn test_table_presets_and_apply_to_all() {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.pages = vec![
            PageEntry {
                kind: PageKind::Table,
                scope: PageItemScope::NewPage,
                title: "Table 1".into(),
                ..Default::default()
            },
            PageEntry {
                kind: PageKind::Paragraph,
                scope: PageItemScope::NewPage,
                title: "Paragraph 1".into(),
                ..Default::default()
            },
            PageEntry {
                kind: PageKind::Table,
                scope: PageItemScope::NewPage,
                title: "Table 2".into(),
                ..Default::default()
            },
        ];

        apply(&mut ui, MetadataMsg::CardTableStyle(0, "Horizontal Rules".into()));
        apply(&mut ui, MetadataMsg::CardTableTheme(0, "#FFCB56 - #FFE9A9".into()));
        apply(&mut ui, MetadataMsg::CardTableLayoutPreset(0, "Technical Spec Card".into()));
        apply(&mut ui, MetadataMsg::CardTableFirstRowHeader(0, true));
        apply(&mut ui, MetadataMsg::CardTableColNumbers(0, true));
        apply(&mut ui, MetadataMsg::CardTableFooter(0, true));
        apply(&mut ui, MetadataMsg::CardTableRowConditions(0, false));
        apply(&mut ui, MetadataMsg::CardTableApplyToAll(0));

        let def_opts = ui.app.project.metadata.default_table_options.as_ref().unwrap();
        assert_eq!(def_opts.style, "Horizontal Rules");
        assert_eq!(def_opts.theme, "#FFCB56 - #FFE9A9");
        assert!(def_opts.first_row_header);
        assert!(def_opts.col_numbers);
        assert!(def_opts.table_footer);
        assert!(!def_opts.row_conditions);

        let t1 = ui.app.project.metadata.pages[0].table_options.as_ref().unwrap();
        assert_eq!(t1.style, "Horizontal Rules");
        assert_eq!(t1.theme, "#FFCB56 - #FFE9A9");
        assert_eq!(t1.layout_preset, "Technical Spec Card");
        assert!(t1.first_row_header);
        assert!(!t1.row_conditions);

        assert!(ui.app.project.metadata.pages[1].table_options.is_none());

        let t2 = ui.app.project.metadata.pages[2].table_options.as_ref().unwrap();
        assert_eq!(t2.style, "Horizontal Rules");
        assert_eq!(t2.theme, "#FFCB56 - #FFE9A9");
        assert!(t2.first_row_header);
        assert!(!t2.row_conditions);
    }

    #[test]
    fn test_compute_spreads() {
        assert!(compute_spreads(0).is_empty());

        let sp1 = compute_spreads(1);
        assert_eq!(sp1.len(), 1);
        assert_eq!(sp1[0], SpreadItem { left: None, right: Some(0), label: "1".into() });

        let sp2 = compute_spreads(2);
        assert_eq!(sp2.len(), 2);
        assert_eq!(sp2[0], SpreadItem { left: None, right: Some(0), label: "1".into() });
        assert_eq!(sp2[1], SpreadItem { left: Some(1), right: None, label: "2".into() });

        let sp3 = compute_spreads(3);
        assert_eq!(sp3.len(), 2);
        assert_eq!(sp3[0], SpreadItem { left: None, right: Some(0), label: "1".into() });
        assert_eq!(sp3[1], SpreadItem { left: Some(1), right: Some(2), label: "2–3".into() });

        let sp22 = compute_spreads(22);
        assert_eq!(sp22.len(), 12);
        assert_eq!(sp22[0].label, "1");
        assert_eq!(sp22[1].label, "2–3");
        assert_eq!(sp22[2].label, "4–5");
        assert_eq!(sp22[3].label, "6–7");
        assert_eq!(sp22[4].label, "8–9");
        assert_eq!(sp22[5].label, "10–11");
        assert_eq!(sp22[6].label, "12–13");
        assert_eq!(sp22[7].label, "14–15");
        assert_eq!(sp22[8].label, "16–17");
        assert_eq!(sp22[9].label, "18–19");
        assert_eq!(sp22[10].label, "20–21");
        assert_eq!(sp22[11].label, "22");
        assert_eq!(sp22[11].left, Some(21));
        assert_eq!(sp22[11].right, None);
    }

    #[test]
    fn test_publication_cards_single_and_double() {
        let (mut ui, _) = UiApp::new();
        ui.app.project.metadata.pages = vec![
            PageEntry {
                kind: PageKind::Cover,
                title: "Cover".into(),
                ..Default::default()
            },
            PageEntry {
                kind: PageKind::Table,
                title: "Table 1".into(),
                ..Default::default()
            },
            PageEntry {
                kind: PageKind::Chart,
                title: "Chart 1".into(),
                ..Default::default()
            },
        ];

        // Single-sided mode
        ui.app.project.metadata.layout = PageLayout::Single;
        {
            let _elem_single = publication_cards(&ui);
        }

        // Double-sided mode
        ui.app.project.metadata.layout = PageLayout::Double;
        {
            let _elem_double = publication_cards(&ui);
        }

        // Active page editing in double-sided mode
        ui.editing_page = Some(1);
        {
            let _elem_double_selected = publication_cards(&ui);
        }
    }
}
