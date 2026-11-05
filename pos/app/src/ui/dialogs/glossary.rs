// Glossary & Terminology modal dialog.
// Adopted from `terminology_pane` in designcraft-pubgen.

use iced::{
    widget::{button, checkbox, column, container, row, scrollable, text, text_input},
    Element, Length, Theme,
};

use super::super::message::{DialogMsg, Message};
use super::super::UiApp;
use super::{dialog_field_label, dialog_frame, dialog_section_title};

pub fn glossary_dialog<'a>(ui: &'a UiApp, dark: bool) -> Element<'a, Message> {
    let app = &ui.app;
    let st = &ui.dialog_state;

    // 1. Search filter & Add Row
    let filter_row = row![
        dialog_field_label("Cari Istilah:"),
        text_input("Ketik untuk mencari istilah...", &st.glossary_filter)
            .width(240)
            .size(11)
            .on_input(|s| Message::DialogMsg(DialogMsg::GlossaryFilter(s))),
        iced::widget::space().width(Length::Fill),
        text(format!("Total: {} istilah", app.project.glossary.len())).size(11),
    ]
    .align_y(iced::Alignment::Center);

    let add_form = row![
        text_input("Istilah (ID)", &app.glossary_term)
            .width(130)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::GlossaryTerm(s))),
        text("→").size(12),
        text_input("Pengganti (EN)", &app.glossary_replacement)
            .width(150)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::GlossaryReplacement(s))),
        checkbox(app.glossary_dnt)
            .label("Keep (DNT)")
            .on_toggle(|b| Message::DialogMsg(DialogMsg::GlossaryDnt(b))),
        button(text("+ Tambah").size(12))
            .padding([3, 10])
            .on_press(Message::DialogMsg(DialogMsg::GlossaryAdd)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // 2. Terms table / list
    let filter = st.glossary_filter.trim().to_lowercase();
    let mut table_rows = column![].spacing(3);

    let filtered_entries: Vec<(usize, &crate::glossary::Entry)> = app
        .project
        .glossary
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            filter.is_empty()
                || e.term.to_lowercase().contains(&filter)
                || e.replacement.to_lowercase().contains(&filter)
        })
        .collect();

    if filtered_entries.is_empty() {
        table_rows = table_rows.push(
            container(
                text(if app.project.glossary.is_empty() {
                    "Glosarium masih kosong. Masukkan istilah baru di atas atau impor berkas CSV."
                } else {
                    "Tidak ada istilah yang cocok dengan kata pencarian."
                })
                .size(11)
                .color(if dark { iced::Color::from_rgb8(150, 155, 165) } else { iced::Color::from_rgb8(120, 125, 135) }),
            )
            .padding(16),
        );
    } else {
        for (idx, entry) in filtered_entries {
            let keep_badge = if entry.do_not_translate {
                container(text("keep").size(9).color(iced::Color::from_rgb8(200, 100, 40)))
                    .padding([1, 4])
                    .style(|theme: &Theme| container::Style {
                        background: Some(theme.palette().text.scale_alpha(0.08).into()),
                        border: iced::Border {
                            radius: (3.0).into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
            } else {
                container(iced::widget::space().width(0))
            };

            let row_item = row![
                text(&entry.term).size(12).width(Length::FillPortion(1)),
                text("→").size(11),
                text(&entry.replacement).size(12).width(Length::FillPortion(1)),
                keep_badge,
                button(text("×").size(12))
                    .padding([1, 6])
                    .on_press(Message::DialogMsg(DialogMsg::GlossaryRemove(idx)))
                    .style(|theme: &Theme, _status| button::Style {
                        background: Some(iced::Color::TRANSPARENT.into()),
                        text_color: theme.palette().text.scale_alpha(0.6),
                        ..Default::default()
                    }),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center);

            table_rows = table_rows.push(
                container(row_item)
                    .padding([3, 6])
                    .width(Length::Fill)
                    .style(|theme: &Theme| container::Style {
                        background: Some(theme.palette().text.scale_alpha(0.02).into()),
                        border: iced::Border {
                            radius: (3.0).into(),
                            width: 0.5,
                            color: theme.palette().text.scale_alpha(0.1),
                        },
                        ..Default::default()
                    }),
            );
        }
    }

    let scroll_table = container(scrollable(table_rows).height(240))
        .padding(4)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.palette().text.scale_alpha(0.03).into()),
            border: iced::Border {
                radius: (4.0).into(),
                width: 1.0,
                color: theme.palette().text.scale_alpha(0.15),
            },
            ..Default::default()
        });

    let body = column![
        dialog_section_title("1. Tambah Istilah Baru"),
        add_form,
        iced::widget::rule::horizontal(1),
        dialog_section_title("2. Memori Glosarium Resmi"),
        filter_row,
        scroll_table,
    ]
    .spacing(8);

    let footer_actions = row![
        button(text("Impor CSV").size(12))
            .padding([4, 10])
            .on_press(Message::DialogMsg(DialogMsg::GlossaryImport)),
        button(text("Ekspor CSV").size(12))
            .padding([4, 10])
            .on_press(Message::DialogMsg(DialogMsg::GlossaryExport)),
    ]
    .spacing(8);

    dialog_frame(
        "📖",
        "Glosarium Terminologi Resmi (BPS Official Glossary)".to_string(),
        "Kelola memori peristilahan resmi, kata serapan, dan istilah yang dilindungi".to_string(),
        body.into(),
        footer_actions.into(),
        dark,
        600.0,
        620.0,
    )
}
