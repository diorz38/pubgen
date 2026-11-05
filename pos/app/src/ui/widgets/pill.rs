// Engine status pill (port of the legacy status pill in the top bar).

use crate::i18n::{tr, UiLang};
use crate::main_ui::{DcStatus, DC_PORT};
use iced::{
    widget::{button, text},
    Element, Length, Theme,
};

use super::super::Message;

fn status_color(status: &DcStatus) -> iced::Color {
    match status {
        DcStatus::Ok => iced::Color::from_rgb8(76, 175, 125),
        DcStatus::Booting => iced::Color::from_rgb8(230, 178, 64),
        DcStatus::Off => iced::Color::from_rgb8(150, 150, 150),
    }
}

fn status_dot(status: &DcStatus) -> &'static str {
    match status {
        DcStatus::Ok | DcStatus::Booting => "●",
        DcStatus::Off => "○",
    }
}

fn status_text(status: &DcStatus, lang: UiLang) -> &'static str {
    match status {
        DcStatus::Ok => tr(lang, "status_engine_ready"),
        DcStatus::Booting => tr(lang, "status_engine_starting"),
        DcStatus::Off => tr(lang, "status_engine_off"),
    }
}

/// Clickable DesignCraft connection pill: colored dot + localized engine label.
/// Click action per legacy port: Off → launch; Ok/Booting → reprobe IPC.
pub fn dc_pill(
    status: &DcStatus,
    lang: UiLang,
    dark: bool,
    on_click: Message,
) -> Element<'static, Message> {
    let dot = status_color(status);
    let label = status_text(status, lang);
    let dot_glyph = status_dot(status);
    let tooltip = format!("{} · 127.0.0.1:{DC_PORT}", tr(lang, "tip_dc_pill"));

    let pill = button(
        text(format!("{dot_glyph} {label}"))
            .size(11)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .width(118)
    .height(24)
    .on_press(on_click)
    .style(move |theme: &Theme, status| {
        let hovered = matches!(status, button::Status::Hovered);
        let bg = if hovered {
            iced::Color {
                a: 45.0 / 255.0,
                ..dot
            }
        } else {
            iced::Color::TRANSPARENT
        };
        let _ = theme;
        button::Style {
            background: Some(bg.into()),
            text_color: if dark {
                iced::Color::from_rgb8(220, 220, 220)
            } else {
                iced::Color::from_rgb8(40, 40, 40)
            },
            border: iced::Border {
                radius: (12.0).into(),
                width: 1.0,
                color: dot,
            },
            ..Default::default()
        }
    });
    super::tooltip(pill, &tooltip)
}
