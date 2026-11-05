use iced::{
    widget::{column, container, mouse_area, row, space, text},
    Element, Length, Theme,
};

use crate::i18n::tr;
use super::super::UiApp;
use super::super::Message;

pub fn busy_overlay<'a>(ui: &'a UiApp, dark: bool) -> Element<'a, Message> {
    let lang = ui.app.ui_lang;

    // Spinner animation frames: Braille dots spinner
    const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner_icon = SPINNER_FRAMES[(ui.busy_tick as usize) % SPINNER_FRAMES.len()];

    // Determine description based on busy_action
    let action_key = ui.busy_action.as_deref().unwrap_or("default");
    let desc = match action_key {
        "pub_generate" => tr(lang, "busy_generating_desc"),
        "pub_generate_publish" | "pub_batch_fill" | "pub_batch_x4" => tr(lang, "busy_export_pdf_desc"),
        "file_import_excel" | "file_import_sources" => tr(lang, "busy_import_desc"),
        "pub_translate" | "auto_translate" => tr(lang, "busy_translate_desc"),
        _ => tr(lang, "busy_generic_desc"),
    };

    // Smooth gliding indicator for indeterminate progress bar
    // Track width: 380px, Runner width: 90px, travel distance = 290px
    let cycle_ticks = 30u32; // ~3.0 seconds at 100ms/tick
    let phase = (ui.busy_tick % cycle_ticks) as f32 / cycle_ticks as f32;
    let t = if phase < 0.5 {
        phase * 2.0
    } else {
        (1.0 - phase) * 2.0
    };
    // Ease in-out cubic for realistic, silky-smooth native feel: 3*t^2 - 2*t^3
    let eased = t * t * (3.0 - 2.0 * t);
    let left_pad = (eased * 290.0).max(0.0);
    let right_pad = (290.0 - left_pad).max(0.0);

    let progress_runner = container(space())
        .width(Length::Fixed(90.0))
        .height(Length::Fixed(6.0))
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Color::from_rgb8(14, 116, 144).into()), // Teal/Cyan POS accent
            border: iced::Border {
                radius: (3.0).into(),
                ..Default::default()
            },
            ..Default::default()
        });

    let progress_track = container(
        row![
            space().width(Length::Fixed(left_pad)),
            progress_runner,
            space().width(Length::Fixed(right_pad)),
        ]
        .align_y(iced::Alignment::Center)
    )
    .width(Length::Fixed(380.0))
    .height(Length::Fixed(6.0))
    .style(move |_theme: &Theme| container::Style {
        background: Some(if dark {
            iced::Color::from_rgb8(35, 39, 50)
        } else {
            iced::Color::from_rgb8(226, 232, 240)
        }.into()),
        border: iced::Border {
            radius: (3.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let header_row = row![
        container(
            text(spinner_icon)
                .size(24)
                .color(iced::Color::from_rgb8(14, 116, 144))
        )
        .padding([0, 4]),
        column![
            text(tr(lang, "busy_generating_title"))
                .size(16)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
            text(desc)
                .size(12)
                .color(if dark {
                    iced::Color::from_rgb8(160, 166, 178)
                } else {
                    iced::Color::from_rgb8(100, 105, 115)
                }),
        ]
        .spacing(3),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    let hint_box = container(
        row![
            text("💡").size(14),
            text(tr(lang, "busy_auto_open_hint"))
                .size(11)
                .color(if dark {
                    iced::Color::from_rgb8(180, 185, 195)
                } else {
                    iced::Color::from_rgb8(75, 85, 100)
                }),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
    )
    .width(Length::Fill)
    .padding([8, 12])
    .style(move |_theme: &Theme| container::Style {
        background: Some(if dark {
            iced::Color::from_rgb8(28, 32, 42)
        } else {
            iced::Color::from_rgb8(241, 245, 249)
        }.into()),
        border: iced::Border {
            radius: (6.0).into(),
            width: 1.0,
            color: if dark {
                iced::Color::from_rgb8(45, 50, 65)
            } else {
                iced::Color::from_rgb8(220, 226, 235)
            },
        },
        ..Default::default()
    });

    let mut card_children: Vec<Element<'a, Message>> = vec![
        header_row.into(),
        space().height(Length::Fixed(6.0)).into(),
        progress_track.into(),
        space().height(Length::Fixed(4.0)).into(),
        hint_box.into(),
    ];

    // If there is a last log message, show it as live CLI activity
    if let Some(last_log) = ui.app.log.last() {
        let clean_log = if last_log.len() > 60 {
            format!("{}…", &last_log[..57])
        } else {
            last_log.clone()
        };
        let log_row = row![
            text("▶").size(10).color(iced::Color::from_rgb8(14, 116, 144)),
            text(clean_log)
                .size(10)
                .font(iced::font::Font {
                    family: iced::font::Family::Monospace,
                    ..Default::default()
                })
                .color(if dark {
                    iced::Color::from_rgb8(140, 145, 155)
                } else {
                    iced::Color::from_rgb8(120, 125, 135)
                }),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        card_children.push(log_row.into());
    }

    let card = container(
        column(card_children)
            .spacing(10)
            .width(Length::Fixed(430.0))
    )
    .padding(20)
    .style(move |_theme: &Theme| container::Style {
        background: Some(if dark {
            iced::Color::from_rgb8(24, 27, 35)
        } else {
            iced::Color::WHITE
        }.into()),
        border: iced::Border {
            radius: (10.0).into(),
            width: 1.0,
            color: if dark {
                iced::Color::from_rgb8(50, 56, 72)
            } else {
                iced::Color::from_rgb8(203, 213, 225)
            },
        },
        shadow: iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, if dark { 0.5 } else { 0.15 }),
            offset: iced::Vector::new(0.0, 6.0),
            blur_radius: 20.0,
        },
        ..Default::default()
    });

    let centered = container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    // Backdrop absorbs mouse clicks so user cannot accidentally trigger background actions
    let backdrop = mouse_area(
        container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.45).into()),
                ..Default::default()
            })
    );

    iced::widget::stack![backdrop, centered]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
