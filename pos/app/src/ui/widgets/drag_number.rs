// Drag-number widget (DragValue replacement decision from Phase 0 spike).
// Scroll / press-release steps the value; +/- buttons as visible fallback.
// Will back the margins inputs in Phase 10.

use iced::{
    widget::{button, column, container, mouse_area, row, text},
    Element, Length, Theme,
};

use super::super::theme;
use super::super::Message;

/// Compact number field. `on_step` receives +1/-1 (drag direction folded in);
/// clamping lives in the caller's update handler against the field's range.
/// Consumed by the metadata margins port in Phase 10.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn drag_number(
    value: i32,
    dragging: bool,
    suffix: &str,
    width: f32,
    on_press: Message,
    on_release: Message,
    on_step: impl Fn(i32) -> Message + Clone + 'static,
    on_wheel: impl Fn(f32) -> Message + Clone + 'static,
) -> Element<'static, Message> {
    let display = container(
        text(format!("{value}{suffix}"))
            .size(13)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .width(width)
    .padding([4, 8])
    .style(|theme: &Theme| container::Style {
        background: Some(theme::lighten(theme.palette().background, 0.03).into()),
        border: iced::Border {
            radius: (4.0).into(),
            width: 1.0,
            color: theme.palette().text.scale_alpha(0.4),
        },
        ..Default::default()
    });

    let stepper: Element<'static, Message> = if dragging {
        mouse_area(
            container(text("⇕ release").size(11))
                .width(width)
                .padding([4, 8]),
        )
        .on_release(on_release)
        .into()
    } else {
        let minus = on_step.clone();
        let plus = on_step.clone();
        row![
            button("-").on_press(minus(-1)),
            button("+").on_press(plus(1)),
        ]
        .spacing(4)
        .into()
    };

    mouse_area(
        column![
            mouse_area(display).on_press(on_press),
            stepper,
        ]
        .spacing(4),
    )
    .on_scroll(move |delta| match delta {
        iced::mouse::ScrollDelta::Lines { y, .. } | iced::mouse::ScrollDelta::Pixels { y, .. } => {
            on_wheel(y)
        }
    })
    .into()
}
