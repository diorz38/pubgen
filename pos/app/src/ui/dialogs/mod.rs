// Dialogs / popup windows for Pubgen automation & content functions.
// Adopted from app reference designcraft-pubgen.

pub mod ai;
pub mod chart;
pub mod cover;
pub mod diff;
pub mod glossary;
pub mod toc;
pub mod wizard;

use iced::{
    widget::{button, column, container, mouse_area, row, rule, scrollable, space, text},
    Element, Length, Theme,
};

use crate::toc::TocKind;
pub use super::message::{DialogKind, Message};
use super::theme;
use super::UiApp;

#[derive(Debug, Clone)]
pub struct DialogState {
    // Cover Studio form options
    pub cover_include_inside_back: bool,
    pub cover_add_map: bool,
    pub cover_map_style: usize,
    pub cover_theme_preset: usize,
    pub cover_title: String,
    pub cover_title2: String,
    pub cover_title_en: String,
    pub cover_year: String,
    pub cover_volume: String,
    pub cover_catalogue: String,
    pub cover_pub_no: String,
    pub cover_agency: String,
    pub cover_primary_color: String,
    pub cover_accent_color: String,
    pub cover_dark_color: String,

    // Vector Chart form options
    pub chart_title_en: String,
    pub chart_category_col: String,
    pub chart_series_1_col: String,
    pub chart_series_1_label: String,
    pub chart_series_2_enabled: bool,
    pub chart_series_2_col: String,
    pub chart_series_2_label: String,
    pub chart_series_3_enabled: bool,
    pub chart_series_3_col: String,
    pub chart_series_3_label: String,
    pub chart_data_start_row: usize,
    pub chart_data_end_row: usize,
    pub chart_value_format: usize,
    pub chart_show_labels: bool,
    pub chart_show_legend: bool,
    pub chart_show_grid: bool,
    pub chart_max_val: String,
    pub chart_map_palette: usize,
    pub chart_raw_csv: String,

    // TOC Generator form options
    pub toc_title: String,
    pub toc_kind: TocKind,
    pub toc_layout: usize,
    pub toc_hanging_indent: bool,
    pub toc_include_self: bool,
    pub toc_pair_bilingual: bool,

    // AI Translation form options
    pub ai_provider: usize,
    pub ai_bilingual: bool,

    // Glossary form options
    pub glossary_filter: String,

    // Onboarding Wizard (A/B Test Variant B)
    pub wizard_step: usize,
    pub wizard_profile_id: String,
    pub wizard_title: String,
    pub wizard_year: String,
    pub wizard_started_at: Option<std::time::Instant>,
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            cover_include_inside_back: true,
            cover_add_map: false,
            cover_map_style: 0,
            cover_theme_preset: 0,
            cover_title: String::new(),
            cover_title2: "Kecamatan Dalam Angka".to_string(),
            cover_title_en: String::new(),
            cover_year: "2026".to_string(),
            cover_volume: "1".to_string(),
            cover_catalogue: "1102001".to_string(),
            cover_pub_no: "32150.2601".to_string(),
            cover_agency: "BADAN PUSAT STATISTIK".to_string(),
            cover_primary_color: "#0072BC".to_string(),
            cover_accent_color: "#F5871F".to_string(),
            cover_dark_color: "#122647".to_string(),

            chart_title_en: "Statistical Figure".to_string(),
            chart_category_col: "A".to_string(),
            chart_series_1_col: "B".to_string(),
            chart_series_1_label: "Pertanian / Agriculture".to_string(),
            chart_series_2_enabled: true,
            chart_series_2_col: "C".to_string(),
            chart_series_2_label: "Industri / Industry".to_string(),
            chart_series_3_enabled: true,
            chart_series_3_col: "D".to_string(),
            chart_series_3_label: "Jasa / Services".to_string(),
            chart_data_start_row: 2,
            chart_data_end_row: 6,
            chart_value_format: 0,
            chart_show_labels: true,
            chart_show_legend: true,
            chart_show_grid: true,
            chart_max_val: String::new(),
            chart_map_palette: 0,
            chart_raw_csv: "Tahun,Pertanian,Industri,Jasa\n2021,3.8,4.2,5.1\n2022,4.1,4.5,5.6\n2023,3.9,4.8,5.9\n2024,4.3,5.1,6.2\n2025,4.5,5.4,6.5".to_string(),

            toc_title: "DAFTAR ISI".to_string(),
            toc_kind: TocKind::Contents,
            toc_layout: 0,
            toc_hanging_indent: true,
            toc_include_self: false,
            toc_pair_bilingual: true,

            ai_provider: 0,
            ai_bilingual: true,

            glossary_filter: String::new(),

            wizard_step: 1,
            wizard_profile_id: "bps_dda".to_string(),
            wizard_title: String::new(),
            wizard_year: "2026".to_string(),
            wizard_started_at: None,
        }
    }
}

/// Renders the active modal dialog popup over the UI stack.
pub fn active_dialog_modal(ui: &UiApp) -> Option<Element<'_, Message>> {
    let kind = ui.active_dialog?;
    let dark = theme::is_dark(ui.app.theme, ui.system_dark);

    let content: Element<'_, Message> = match kind {
        DialogKind::CoverStudio => cover::cover_dialog(ui, dark),
        DialogKind::VectorChart => chart::chart_dialog(ui, dark),
        DialogKind::TocGenerator => toc::toc_dialog(ui, dark),
        DialogKind::AiTranslation => ai::ai_dialog(ui, dark),
        DialogKind::Glossary => glossary::glossary_dialog(ui, dark),
        DialogKind::Diff => diff::diff_dialog(ui, dark),
        DialogKind::OnboardingWizard => wizard::wizard_dialog(ui, dark),
    };

    let centered = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    let backdrop = mouse_area(
        container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.50).into()),
                ..Default::default()
            }),
    )
    .on_press(Message::CloseDialog);

    Some(iced::widget::stack![backdrop, centered].into())
}

/// Common card container used by all popup dialogs.
pub fn dialog_frame<'a>(
    icon: &'static str,
    title: String,
    subtitle: String,
    body: Element<'a, Message>,
    footer_actions: Element<'a, Message>,
    dark: bool,
    max_w: f32,
    max_h: f32,
) -> Element<'a, Message> {
    let accent = theme::ACCENT;
    let muted = if dark {
        iced::Color::from_rgb8(150, 155, 165)
    } else {
        iced::Color::from_rgb8(105, 115, 130)
    };

    let header = row![
        text(icon).size(20),
        column![
            text(title).size(15).style(move |_theme: &Theme| text::Style {
                color: Some(accent),
            }),
            text(subtitle).size(11).color(muted),
        ]
        .spacing(1),
        space().width(Length::Fill),
        close_icon_button(),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let footer = row![
        space().width(Length::Fill),
        footer_actions,
        button(text("Tutup").size(12))
            .padding([4, 12])
            .on_press(Message::CloseDialog)
            .style(move |_theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered => iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2),
                    button::Status::Pressed => iced::Color::from_rgba(0.5, 0.5, 0.5, 0.3),
                    _ => iced::Color::TRANSPARENT,
                };
                button::Style {
                    background: Some(bg.into()),
                    text_color: if dark { iced::Color::WHITE } else { iced::Color::BLACK },
                    border: iced::Border {
                        radius: (4.0).into(),
                        width: 1.0,
                        color: if dark { iced::Color::from_rgb8(70, 75, 85) } else { iced::Color::from_rgb8(200, 205, 215) },
                    },
                    ..Default::default()
                }
            })
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    container(
        column![
            header,
            rule::horizontal(1),
            scrollable(container(body).padding([2, 4])).height(Length::Fill),
            rule::horizontal(1),
            footer,
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .max_width(max_w)
    .height(Length::Fill)
    .max_height(max_h)
    .padding(14)
    .style(move |_theme: &Theme| container::Style {
        background: Some(
            (if dark {
                iced::Color::from_rgb8(28, 31, 38)
            } else {
                iced::Color::WHITE
            })
            .into(),
        ),
        border: iced::Border {
            radius: (10.0).into(),
            width: 1.0,
            color: accent.scale_alpha(0.6),
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, if dark { 0.6 } else { 0.3 }),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 28.0,
        },
        ..Default::default()
    })
    .into()
}

pub fn close_icon_button() -> Element<'static, Message> {
    button(text("×").size(16))
        .padding([2, 8])
        .on_press(Message::CloseDialog)
        .style(|theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered => theme.palette().text.scale_alpha(0.12),
                button::Status::Pressed => theme.palette().text.scale_alpha(0.25),
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

pub fn dialog_section_title(label: &str) -> Element<'static, Message> {
    container(
        text(label.to_string())
            .size(12)
            .style(|_theme: &Theme| text::Style {
                color: Some(theme::ACCENT),
            }),
    )
    .padding([2, 0])
    .into()
}

pub fn dialog_field_label(label: &str) -> Element<'static, Message> {
    text(label.to_string()).size(11).into()
}
