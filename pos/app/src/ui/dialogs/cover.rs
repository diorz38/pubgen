// Cover Studio modal dialog.
// Adopted from `card_cover_body_ui` in designcraft-pubgen.

use iced::{
    widget::{button, checkbox, column, container, row, text, text_input},
    Element, Length, Theme,
};

use crate::covers::CoverStyle;
use super::super::message::{DialogMsg, Message};
use super::super::theme;
use super::super::widgets;
use super::super::UiApp;
use super::{dialog_field_label, dialog_frame, dialog_section_title};

pub fn cover_dialog<'a>(ui: &'a UiApp, dark: bool) -> Element<'a, Message> {
    let app = &ui.app;
    let st = &ui.dialog_state;

    let style_row = row![
        dialog_field_label("Gaya Templat (Style):"),
        widgets::combo_row(
            CoverStyle::ALL.to_vec(),
            Some(app.cover_style),
            240.0,
            |s| Message::DialogMsg(DialogMsg::CoverStyle(s)),
        ),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let check_inside_back = checkbox(
        st.cover_include_inside_back,
    )
    .label("Tambahkan Sampul Dalam & Sampul Belakang (Inside & Back Cover)")
    .on_toggle(|b| Message::DialogMsg(DialogMsg::CoverIncludeBack(b)));

    let check_map = checkbox(
        st.cover_add_map,
    )
    .label("Tambahkan Peta Wilayah pada Sampul (Map Overlay)")
    .on_toggle(|b| Message::DialogMsg(DialogMsg::CoverAddMap(b)));

    let mut map_style_options = column![check_map].spacing(4);
    if st.cover_add_map {
        let style_picker = row![
            dialog_field_label("Gaya Peta:"),
            map_radio_button(st.cover_map_style == 0, "Watermark", 0),
            map_radio_button(st.cover_map_style == 1, "Accent Hero", 1),
            map_radio_button(st.cover_map_style == 2, "Outlines", 2),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);
        map_style_options = map_style_options.push(container(style_picker).padding([2, 16]));
    }

    // Color theme presets
    let theme_presets = [
        "Classic BPS (Blue/Gold)",
        "Corporate Navy & Amber",
        "Emerald & Gold",
        "Crimson & Slate",
        "Modern Teal & Cyan",
        "Modern Violet & Slate",
        "Report Navy & Orange",
        "Kustom (Custom Hex)",
    ];
    let mut theme_buttons = row![dialog_field_label("Preset Warna:")].spacing(6).align_y(iced::Alignment::Center);
    for (idx, name) in theme_presets.iter().enumerate() {
        if idx == 4 {
            // keep row compact
            break;
        }
        let is_selected = st.cover_theme_preset == idx;
        let btn = button(text(*name).size(11))
            .padding([3, 8])
            .on_press(Message::DialogMsg(DialogMsg::CoverThemePreset(idx)))
            .style(move |theme: &Theme, _status| {
                let bg = if is_selected {
                    theme::ACCENT.scale_alpha(0.25)
                } else {
                    theme.palette().text.scale_alpha(0.06)
                };
                button::Style {
                    background: Some(bg.into()),
                    text_color: if is_selected { theme::ACCENT } else { theme.palette().text },
                    border: iced::Border {
                        radius: (4.0).into(),
                        width: if is_selected { 1.5 } else { 1.0 },
                        color: if is_selected { theme::ACCENT } else { theme.palette().text.scale_alpha(0.2) },
                    },
                    ..Default::default()
                }
            });
        theme_buttons = theme_buttons.push(btn);
    }

    let color_inputs = row![
        dialog_field_label("Hex Warna:"),
        text_input("Primary", &st.cover_primary_color)
            .width(85)
            .size(11)
            .on_input(|s| Message::DialogMsg(DialogMsg::CoverPrimaryColor(s))),
        text_input("Accent", &st.cover_accent_color)
            .width(85)
            .size(11)
            .on_input(|s| Message::DialogMsg(DialogMsg::CoverAccentColor(s))),
        text_input("Dark", &st.cover_dark_color)
            .width(85)
            .size(11)
            .on_input(|s| Message::DialogMsg(DialogMsg::CoverDarkColor(s))),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Metadata Fields
    let title_input = text_input(
        if app.project.metadata.title.is_empty() { "KARAWANG DALAM ANGKA" } else { &app.project.metadata.title },
        if st.cover_title.is_empty() { &app.project.metadata.title } else { &st.cover_title },
    )
    .size(12)
    .on_input(|s| Message::DialogMsg(DialogMsg::CoverTitle(s)));

    let title2_input = text_input(
        "Sub-Judul / Baris Kedua",
        &st.cover_title2,
    )
    .size(12)
    .on_input(|s| Message::DialogMsg(DialogMsg::CoverTitle2(s)));

    let title_en_input = text_input(
        if app.project.metadata.title_en.is_empty() { "KARAWANG IN FIGURES" } else { &app.project.metadata.title_en },
        if st.cover_title_en.is_empty() { &app.project.metadata.title_en } else { &st.cover_title_en },
    )
    .size(12)
    .on_input(|s| Message::DialogMsg(DialogMsg::CoverTitleEn(s)));

    let row_meta_1 = row![
        column![
            dialog_field_label("Tahun:"),
            text_input("Tahun", &st.cover_year)
                .width(80)
                .size(12)
                .on_input(|s| Message::DialogMsg(DialogMsg::CoverYear(s))),
        ].spacing(2),
        column![
            dialog_field_label("Volume:"),
            text_input("Volume", &st.cover_volume)
                .width(80)
                .size(12)
                .on_input(|s| Message::DialogMsg(DialogMsg::CoverVolume(s))),
        ].spacing(2),
        column![
            dialog_field_label("No. Katalog:"),
            text_input("Katalog", &st.cover_catalogue)
                .width(130)
                .size(12)
                .on_input(|s| Message::DialogMsg(DialogMsg::CoverCatalogue(s))),
        ].spacing(2),
        column![
            dialog_field_label("No. Publikasi:"),
            text_input("No. Publikasi", &st.cover_pub_no)
                .width(130)
                .size(12)
                .on_input(|s| Message::DialogMsg(DialogMsg::CoverPubNo(s))),
        ].spacing(2),
    ]
    .spacing(12);

    let agency_row = row![
        dialog_field_label("Penerbit / Agency:"),
        text_input(
            if app.project.metadata.agency.is_empty() { "BADAN PUSAT STATISTIK" } else { &app.project.metadata.agency },
            &st.cover_agency,
        )
        .size(12)
        .width(Length::Fill)
        .on_input(|s| Message::DialogMsg(DialogMsg::CoverAgency(s))),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // AI Theme & Prompt
    let ai_theme_row = row![
        dialog_field_label("Tema AI:"),
        text_input("misal: pertanian, industri, maritim", &app.ai_cover_theme)
            .width(220)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::CoverAiTheme(s))),
        button(text("Salin Prompt AI").size(11))
            .padding([3, 10])
            .on_press(Message::CoverPromptCopy),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let body = column![
        dialog_section_title("1. Gaya & Tata Letak Sampul"),
        style_row,
        check_inside_back,
        map_style_options,
        dialog_section_title("2. Palet Warna & Tema Visual"),
        theme_buttons,
        color_inputs,
        dialog_section_title("3. Tipografi & Teks Sampul"),
        column![dialog_field_label("Judul Utama:"), title_input].spacing(2),
        column![dialog_field_label("Sub-Judul (Title 2):"), title2_input].spacing(2),
        column![dialog_field_label("Judul Bahasa Inggris (Title EN):"), title_en_input].spacing(2),
        row_meta_1,
        agency_row,
        dialog_section_title("4. Generator Prompt Sampul AI"),
        ai_theme_row,
    ]
    .spacing(8);

    let footer_actions = row![
        button(text("Ekspor Cover SVG").size(12))
            .padding([4, 14])
            .on_press(Message::DialogMsg(DialogMsg::CoverExportSvg))
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
    .spacing(8);

    dialog_frame(
        "🎨",
        "Studio Sampul Publikasi (Publication Cover Generator)".to_string(),
        "Sesuaikan gaya sampul, tema warna, peta vektor, dan tipografi publikasi".to_string(),
        body.into(),
        footer_actions.into(),
        dark,
        640.0,
        740.0,
    )
}

fn map_radio_button(selected: bool, label: &'static str, style: usize) -> Element<'static, Message> {
    button(text(label).size(11))
        .padding([2, 8])
        .on_press(Message::DialogMsg(DialogMsg::CoverMapStyle(style)))
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

