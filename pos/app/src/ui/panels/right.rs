// Right panel: profile picker, DC controls, slot list. Port of panels::right + slot_row.

use iced::{
    widget::{button, column, container, row, scrollable, text, text_input},
    Element, Length, Theme,
};

use crate::i18n::tr;

use super::super::message::*;
use super::super::UiApp;
use super::left::{rule, section, text_btn, text_btn_maybe, text_btn_state, weak};

pub fn right(ui: &UiApp) -> Element<'_, Message> {
    let app = &ui.app;
    let lang = app.ui_lang;

    let current = crate::profiles::ProfileManager::get_profile(&app.selected_profile);
    let g = &current.geometry;

    let profile_combo = iced::widget::combo_box(
        &ui.profile_picker,
        "Publication profile",
        Some(&current),
        Message::ProfilePicked,
    )
    .width(280)
    .size(12);

    let profile_group = container(
        column![
            text(tr(lang, "publication_profile")).size(13),
            profile_combo,
            weak(
                format!(
                    "Margins: T:{:.1} B:{:.1} L:{:.1} R:{:.1} mm | Spreads: {}",
                    g.margins_mm.top,
                    g.margins_mm.bottom,
                    g.margins_mm.inside,
                    g.margins_mm.outside,
                    if g.facing_pages { tr(lang, "yes") } else { tr(lang, "no") },
                ),
                true,
            ),
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .padding(8)
    .style(|theme: &Theme| container::Style {
        border: iced::Border {
            radius: (4.0).into(),
            width: 1.0,
            color: theme.palette().text.scale_alpha(0.10),
        },
        ..Default::default()
    });

    let engine_off = matches!(ui.app.dc_status, crate::main_ui::DcStatus::Off);
    let busy = ui.busy;

    // Generate button (generates .designcraft pages only)
    let gen_btn = button(
        row![
            text(if busy { "⏳" } else { "⚡" }).size(13),
            text(if busy { tr(lang, "btn_generating") } else { tr(lang, "btn_generate") })
                .size(12)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                }),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([4, 10])
    .on_press_maybe((!busy).then_some(Message::Action("pub_generate")))
    .style(move |_theme, status| {
        let mut bg = iced::Color::from_rgb8(14, 116, 144); // Deep Cyan (#0E7490)
        if !busy {
            match status {
                button::Status::Hovered => bg = super::super::theme::lighten(bg, 0.15),
                button::Status::Pressed => bg = iced::Color {
                    r: bg.r * 0.85,
                    g: bg.g * 0.85,
                    b: bg.b * 0.85,
                    a: bg.a,
                },
                _ => {}
            }
        } else {
            bg = iced::Color::from_rgb8(100, 115, 130);
        }
        button::Style {
            background: Some(bg.into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: (2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    // Generate & Publish button (generates .designcraft and exports PDF)
    let gen_pub_btn = button(
        row![
            text(if busy { "⏳" } else { "🚀" }).size(13),
            text(if busy { tr(lang, "btn_generating_publishing") } else { tr(lang, "btn_generate_publish") })
                .size(12)
                .font(iced::font::Font {
                    weight: iced::font::Weight::Semibold,
                    ..Default::default()
                }),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([4, 10])
    .on_press_maybe((!busy).then_some(Message::Action("pub_batch_fill")))
    .style(move |_theme, status| {
        let mut bg = iced::Color::from_rgb8(22, 163, 74); // Vibrant Green (#16A34A)
        if !busy {
            match status {
                button::Status::Hovered => bg = super::super::theme::lighten(bg, 0.15),
                button::Status::Pressed => bg = iced::Color {
                    r: bg.r * 0.85,
                    g: bg.g * 0.85,
                    b: bg.b * 0.85,
                    a: bg.a,
                },
                _ => {}
            }
        } else {
            bg = iced::Color::from_rgb8(100, 115, 130);
        }
        button::Style {
            background: Some(bg.into()),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                radius: (2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    let gen_row = row![
        crate::ui::widgets::tooltip(gen_btn, tr(lang, "tip_generate")),
        crate::ui::widgets::tooltip(gen_pub_btn, tr(lang, "tip_generate_publish")),
    ]
    .spacing(8)
    .width(Length::Fill);
    let action_btn = |label: &str, tip: &str, engine_dep: bool, msg: Message| {
        if engine_dep && engine_off {
            text_btn_maybe(label.to_string(), tr(lang, "tip_engine_off"), None)
        } else {
            let tip = if busy { tr(lang, "tip_busy") } else { tip };
            text_btn_state(label.to_string(), tip, !busy, msg)
        }
    };

    let dc_row1 = row![
        action_btn(
            tr(lang, "launch_dc"),
            tr(lang, "tip_launch_dc"),
            false,
            Message::Action("dc_launch"),
        ),
        action_btn(
            tr(lang, "kill_dc"),
            tr(lang, "tip_kill_dc"),
            true,
            Message::Action("dc_kill"),
        ),
        action_btn(
            tr(lang, "open_in_designcraft"),
            tr(lang, "tip_open_dc"),
            true,
            Message::Action("dc_open_doc"),
        ),
    ]
    .spacing(6);

    let dc_row2 = row![
        action_btn(
            tr(lang, "discover_slots"),
            tr(lang, "tip_discover_slots"),
            true,
            Message::Action("pub_discover"),
        ),
        action_btn(
            tr(lang, "fill_live"),
            tr(lang, "tip_fill_live"),
            true,
            Message::Action("pub_fill_live"),
        ),
    ]
    .spacing(6);

    let mut slot_list = column![
        text(format!("{} ({})", tr(lang, "detected_slots"), app.project.slots.len())).size(13)
    ]
    .spacing(2);

    if app.project.slots.is_empty() {
        slot_list = slot_list.push(weak(tr(lang, "no_slots"), true));
    }
    for index in 0..app.project.slots.len() {
        slot_list = slot_list.push(slot_row(ui, index));
    }

    container(
        column![
            section(tr(lang, "profile_and_run")),
            profile_group,
            gen_row,
            rule(),
            section(tr(lang, "engine_tools")),
            dc_row1,
            rule(),
            dc_row2,
            rule(),
            scrollable(slot_list).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(6),
    )
    .width(360)
    .height(Length::Fill)
    .padding([4, 8])
    .into()
}

fn slot_row(ui: &UiApp, index: usize) -> Element<'_, Message> {
    let Some(slot) = ui.app.project.slots.get(index) else {
        return column![].into();
    };
    let dark_open = ui.open_slots.contains(&index);

    let header_btn = button(
        row![text(if dark_open { "▾" } else { "▸" }).size(11), text(format!("☑ {}", slot.name)).size(12)]
            .spacing(4),
    )
    .padding([2, 4])
    .on_press(Message::SlotToggled(index))
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

    if !dark_open {
        return header_btn.into();
    }

    let budget_value = slot.budget.map(|b| b.to_string()).unwrap_or_default();
    let body = column![
        row![
            text("frame / story").size(11).width(90),
            text(format!("{} / {}", slot.frame, slot.story)).size(11),
        ]
        .spacing(6),
        row![
            text("budget").size(11).width(90),
            text_input("0", &budget_value)
                .width(90)
                .size(11)
                .on_input(move |s| Message::SlotBudget(index, s)),
        ]
        .spacing(6),
        text_input("source text (original language)", &slot.source)
            .size(11)
            .on_input(move |s| Message::SlotSource(index, s)),
        text_input("translated text", &slot.text)
            .size(11)
            .on_input(move |s| Message::SlotText(index, s)),
        weak(format!("{} chars", slot.text.chars().count()), true),
        row![
            text_btn("Remove".into(), "", Message::SlotRemove(index)),
            text_btn("Unname".into(), "", Message::SlotUnname(index)),
        ]
        .spacing(6),
    ]
    .spacing(4)
    .padding(iced::Padding {
        top: 2.0,
        right: 12.0,
        bottom: 0.0,
        left: 12.0,
    });

    column![header_btn, body].spacing(2).into()
}
