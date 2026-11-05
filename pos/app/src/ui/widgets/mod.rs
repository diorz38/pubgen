// Shared widget helpers for the iced UI.

pub mod busy_overlay;
pub mod drag_number;
pub mod pill;

pub use busy_overlay::busy_overlay;

use iced::{
    widget::{button, column, container, pick_list, row, text, text_input, Tooltip},
    Element, Theme,
};

use super::theme;
use super::Message;

/// Wrap a widget with a localized tooltip that appears after ~400 ms.
///
/// An empty label returns the widget untouched so callers can keep a single
/// optional `tooltip` parameter without branching.
pub fn tooltip<'a>(
    content: impl Into<Element<'a, Message>>,
    label: &str,
) -> Element<'a, Message> {
    let content = content.into();
    if label.trim().is_empty() {
        return content;
    }

    let surface = container(text(label.to_string()).size(11))
        .padding([3, 7])
        .style(|theme: &Theme| container::Style {
            text_color: Some(theme.palette().text),
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: iced::Border {
                radius: (4.0).into(),
                width: 1.0,
                color: theme.palette().text.scale_alpha(0.2),
            },
            ..Default::default()
        });

    Tooltip::new(
        content,
        surface,
        iced::widget::tooltip::Position::Bottom,
    )
    .delay(std::time::Duration::from_millis(400))
    .gap(4)
    .into()
}

/// Standard filled action button (accent-colored like the legacy Compile button when `primary`).
#[allow(dead_code)] // Phase 10/11 will use it for metadata/tool bars
pub fn action_button(label: &str, primary: bool, on_press: Message) -> Element<'_, Message> {
    button(text(label.to_string()).size(12))
        .padding([6, 14])
        .on_press(on_press)
        .style(move |theme: &Theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            if primary {
                let mut bg = theme::ACCENT;
                if hovered {
                    bg = theme::lighten(bg, 0.12);
                }
                button::Style {
                    background: Some(bg.into()),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        radius: (6.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            } else {
                let bg = if hovered {
                    theme.palette().text.scale_alpha(0.08)
                } else {
                    iced::Color::TRANSPARENT
                };
                button::Style {
                    background: Some(bg.into()),
                    text_color: theme.palette().text,
                    border: iced::Border {
                        radius: (6.0).into(),
                        width: 1.0,
                        color: theme.palette().text.scale_alpha(0.25),
                    },
                    ..Default::default()
                }
            }
        })
        .into()
}

/// Label + text input row (port of legacy TextEdit rows).
/// Consumed by the studio panels port (Phase 7) and metadata form (Phase 10).
#[allow(dead_code)]
pub fn text_row(
    label: &str,
    value: &str,
    width: f32,
    on_input: impl Fn(String) -> Message + 'static,
) -> Element<'static, Message> {
    let input = text_input(label, value)
        .width(width)
        .size(12)
        .on_input(on_input);

    row![text(format!("{label}:")).size(12), input]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
}
/// Pick list with fixed options (port of legacy ComboBox with fixed choices).
pub fn combo_row<V>(
    options: Vec<V>,
    selected: Option<V>,
    width: f32,
    on_select: impl Fn(V) -> Message + 'static,
) -> Element<'static, Message>
where
    V: PartialEq + ToString + Clone + 'static,
{
    pick_list(options, selected, on_select)
        .width(width)
        .text_size(12)
        .into()
}

/// Collapsible section (port of legacy CollapsingHeader).
/// Header row with ▸/▾ indicator; content shown only when open.
pub fn collapsible<'a>(
    title: &str,
    open: bool,
    on_toggle: Message,
    content: impl Into<iced::Element<'a, Message>>,
) -> iced::Element<'a, Message> {
    let indicator = if open { "▾" } else { "▸" };
    let header = button(
        row![text(indicator.to_string()).size(11), text(title.to_string()).size(12)]
            .spacing(6),
    )
    .padding([4, 2])
    .on_press(on_toggle)
    .style(|theme: &Theme, status| {
        let bg = if matches!(status, button::Status::Hovered) {
            theme.palette().text.scale_alpha(0.08)
        } else {
            iced::Color::TRANSPARENT
        };
        button::Style {
            background: Some(bg.into()),
            text_color: theme.palette().text,
            border: iced::Border::default(),
            ..Default::default()
        }
    });

    if open {
        column![header, content.into()].spacing(4).into()
    } else {
        column![header].into()
    }
}
