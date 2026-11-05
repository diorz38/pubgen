// Table of Contents modal dialog.
// Adopted from `card_toc_ui` in designcraft-pubgen.

use iced::{
    widget::{button, checkbox, column, container, row, scrollable, text, text_input},
    Element, Length, Theme,
};

use crate::toc::TocKind;
use super::super::message::{DialogMsg, Message};
use super::super::theme;
use super::super::UiApp;
use super::{dialog_field_label, dialog_frame, dialog_section_title};

pub fn toc_dialog<'a>(ui: &'a UiApp, dark: bool) -> Element<'a, Message> {
    let app = &ui.app;
    let st = &ui.dialog_state;

    // 1. TOC Title & Style
    let title_row = row![
        dialog_field_label("Judul Daftar Isi:"),
        text_input("misal: DAFTAR ISI", &st.toc_title)
            .size(12)
            .width(260)
            .on_input(|s| Message::DialogMsg(DialogMsg::TocTitle(s))),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let layout_picker = row![
        dialog_field_label("Tata Letak (Layout):"),
        layout_button(st.toc_layout == 0, "Standar (Titik-titik)", 0),
        layout_button(st.toc_layout == 1, "BPS TablePage (Halaman Mandiri)", 1),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // 2. Options
    let check_hanging = checkbox(st.toc_hanging_indent)
        .label("Indentasi gantung untuk judul panjang (Hanging Indent)")
        .on_toggle(|b| Message::DialogMsg(DialogMsg::TocHangingIndent(b)));

    let check_self = checkbox(st.toc_include_self)
        .label("Sertakan halaman TOC sendiri dalam daftar isi")
        .on_toggle(|b| Message::DialogMsg(DialogMsg::TocIncludeSelf(b)));

    let check_bilingual = checkbox(st.toc_pair_bilingual)
        .label("Pasangkan judul dwibahasa (ID / EN) menjadi satu baris")
        .on_toggle(|b| Message::DialogMsg(DialogMsg::TocPairBilingual(b)));

    // 3. Extraction Action buttons
    let extract_buttons = row![
        button(text("📑 Daftar Isi (Bab)").size(12))
            .padding([5, 12])
            .on_press(Message::DialogMsg(DialogMsg::TocExtract(TocKind::Contents))),
        button(text("📊 Daftar Tabel").size(12))
            .padding([5, 12])
            .on_press(Message::DialogMsg(DialogMsg::TocExtract(TocKind::Tables))),
        button(text("📈 Daftar Grafik").size(12))
            .padding([5, 12])
            .on_press(Message::DialogMsg(DialogMsg::TocExtract(TocKind::Charts))),
    ]
    .spacing(8);

    // 4. Output / Preview area
    let mut preview_col = column![
        row![
            dialog_section_title("3. Pratinjau Teks Daftar Isi (TOC Story Text)"),
            iced::widget::space().width(Length::Fill),
            button(text("Salin Teks TOC").size(11))
                .padding([2, 8])
                .on_press(Message::DialogMsg(DialogMsg::TocCopy)),
        ]
        .align_y(iced::Alignment::Center),
    ]
    .spacing(4);

    if !app.toc_entries.is_empty() {
        let is_bilingual = app.project.metadata.lang == crate::metadata_form::Lang::IdEn;
        let text_out = crate::toc::format_toc_story_text(
            &app.toc_entries,
            is_bilingual,
            app.toc_entries[0].kind,
        );

        let preview_box = container(
            scrollable(text(text_out).size(11)).height(140),
        )
        .width(Length::Fill)
        .padding(8)
        .style(move |theme: &Theme| container::Style {
            background: Some(theme.palette().text.scale_alpha(0.04).into()),
            border: iced::Border {
                radius: (4.0).into(),
                width: 1.0,
                color: theme.palette().text.scale_alpha(0.15),
            },
            ..Default::default()
        });
        preview_col = preview_col.push(preview_box);
    } else {
        let empty_msg = container(
            text("Belum ada entri diekstrak. Klik salah satu tombol di atas untuk mengekstrak struktur dokumen.")
                .size(11)
                .color(if dark { iced::Color::from_rgb8(150, 155, 165) } else { iced::Color::from_rgb8(120, 125, 135) }),
        )
        .padding(12);
        preview_col = preview_col.push(empty_msg);
    }

    let body = column![
        dialog_section_title("1. Format & Gaya Naskah"),
        title_row,
        layout_picker,
        dialog_section_title("2. Opsi Penomoran & Tipografi"),
        check_hanging,
        check_self,
        check_bilingual,
        iced::widget::rule::horizontal(1),
        dialog_field_label("Pilih jenis ekstraksi struktur:"),
        extract_buttons,
        preview_col,
    ]
    .spacing(8);

    let footer_actions = row![
        button(text("Salin Teks").size(12))
            .padding([4, 12])
            .on_press(Message::DialogMsg(DialogMsg::TocCopy)),
    ];

    dialog_frame(
        "📑",
        "Generator Daftar Isi & Tabel (Table of Contents)".to_string(),
        "Ekstrak otomatis daftar isi bab, daftar tabel, dan daftar grafik dari dokumen".to_string(),
        body.into(),
        footer_actions.into(),
        dark,
        580.0,
        640.0,
    )
}

fn layout_button(selected: bool, label: &'static str, val: usize) -> Element<'static, Message> {
    button(text(label).size(11))
        .padding([2, 8])
        .on_press(Message::DialogMsg(DialogMsg::TocLayout(val)))
        .style(move |theme: &Theme, _status| {
            let bg = if selected {
                theme::ACCENT.scale_alpha(0.2)
            } else {
                theme.palette().text.scale_alpha(0.05)
            };
            button::Style {
                background: Some(bg.into()),
                text_color: if selected { theme::ACCENT } else { theme.palette().text },
                border: iced::Border {
                    radius: (3.0).into(),
                    width: if selected { 1.0 } else { 0.5 },
                    color: if selected { theme::ACCENT } else { theme.palette().text.scale_alpha(0.2) },
                },
                ..Default::default()
            }
        })
        .into()
}

