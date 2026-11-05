// Translation Diff Review modal dialog.
// Adopted from `diff_list` in designcraft-pubgen.

use iced::{
    widget::{button, column, container, row, scrollable, text},
    Element, Length, Theme,
};

use super::super::message::{DialogMsg, Message};
use super::super::theme;
use super::super::UiApp;
use super::{dialog_frame, dialog_section_title};

pub fn diff_dialog<'a>(ui: &'a UiApp, dark: bool) -> Element<'a, Message> {
    let app = &ui.app;

    let translated_slots: Vec<_> = app
        .project
        .slots
        .iter()
        .filter(|s| !s.source.is_empty() && !s.text.is_empty())
        .collect();

    let total_slots = app.project.slots.len();
    let translated_count = translated_slots.len();

    let summary = row![
        text(format!("Total Slot: {total_slots} | Diterjemahkan: {translated_count}")).size(12),
        iced::widget::space().width(Length::Fill),
        text("Bahasa Proyek: ID → EN").size(11).color(theme::ACCENT),
    ]
    .align_y(iced::Alignment::Center);

    let mut list_col = column![].spacing(6);

    if translated_slots.is_empty() {
        list_col = list_col.push(
            container(
                text("Belum ada teks terjemahan yang dihasilkan. Klik tombol 'Terjemahkan Konten Proyek' di bawah untuk memulai.")
                    .size(11)
                    .color(if dark { iced::Color::from_rgb8(150, 155, 165) } else { iced::Color::from_rgb8(120, 125, 135) }),
            )
            .padding(20),
        );
    } else {
        for slot in translated_slots {
            let item_box = container(
                column![
                    row![
                        text(&slot.name).size(12).style(|_theme: &Theme| text::Style { color: Some(theme::ACCENT) }),
                        iced::widget::space().width(Length::Fill),
                        text("Terkait: Story / Slot").size(10).color(if dark { iced::Color::from_rgb8(140, 145, 155) } else { iced::Color::from_rgb8(110, 115, 125) }),
                    ],
                    row![
                        container(text("ID (Asli):").size(10)).width(60),
                        text(&slot.source).size(11),
                    ]
                    .spacing(4),
                    row![
                        container(text("EN (Hasil):").size(10)).width(60),
                        text(&slot.text)
                            .size(11)
                            .color(iced::Color::from_rgb8(25, 118, 210)),
                    ]
                    .spacing(4),
                ]
                .spacing(3),
            )
            .padding(8)
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.palette().text.scale_alpha(0.02).into()),
                border: iced::Border {
                    radius: (4.0).into(),
                    width: 1.0,
                    color: theme.palette().text.scale_alpha(0.15),
                },
                ..Default::default()
            });

            list_col = list_col.push(item_box);
        }
    }

    let scrollable_list = container(scrollable(list_col).height(300))
        .padding(4)
        .width(Length::Fill);

    let body = column![
        dialog_section_title("Ringkasan Terjemahan"),
        summary,
        iced::widget::rule::horizontal(1),
        dialog_section_title("Daftar Perbandingan Teks (Indonesian → English Diff)"),
        scrollable_list,
    ]
    .spacing(8);

    let footer_actions = row![
        button(text("Terjemahkan Ulang Konten Proyek").size(12))
            .padding([4, 14])
            .on_press(Message::DialogMsg(DialogMsg::DiffTranslate))
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
    ];

    dialog_frame(
        "🔍",
        "Komparasi Terjemahan (Translation Diff Review)".to_string(),
        "Tinjau perbandingan teks asli bahasa Indonesia dan hasil terjemahan bahasa Inggris".to_string(),
        body.into(),
        footer_actions.into(),
        dark,
        620.0,
        580.0,
    )
}

