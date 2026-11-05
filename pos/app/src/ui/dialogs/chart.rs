// Vector Chart & Map modal dialog.
// Adopted from `charts/dialog.rs` in designcraft-pubgen.

use iced::{
    widget::{button, checkbox, column, row, text, text_input},
    Element, Length, Theme,
};

use crate::vector::ChartType;
use super::super::message::{DialogMsg, Message};
use super::super::theme;
use super::super::widgets;
use super::super::UiApp;
use super::{dialog_field_label, dialog_frame, dialog_section_title};

pub fn chart_dialog<'a>(ui: &'a UiApp, dark: bool) -> Element<'a, Message> {
    let app = &ui.app;
    let st = &ui.dialog_state;

    // 1. Chart Type & Titles
    let type_row = row![
        dialog_field_label("Jenis Grafik (Type):"),
        widgets::combo_row(
            vec![
                ChartType::BarClustered,
                ChartType::HorizontalBar,
                ChartType::Line,
                ChartType::Donut,
                ChartType::Map,
            ],
            Some(app.chart_type),
            230.0,
            |t| Message::DialogMsg(DialogMsg::ChartType(t)),
        ),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let title_id_row = column![
        dialog_field_label("Judul Bahasa Indonesia (Line 1):"),
        text_input("misal: Grafik 01.01 Indeks Pembangunan Manusia", &app.chart_title)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartTitle(s))),
    ]
    .spacing(2);

    let title_en_row = column![
        dialog_field_label("Judul Bahasa Inggris (Line 2 / EN):"),
        text_input("misal: Figure 01.01 Human Development Index", &st.chart_title_en)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartTitleEn(s))),
    ]
    .spacing(2);

    // 2. Data Series Mapping
    let cat_col = row![
        dialog_field_label("Kolom Kategori (X-Axis):"),
        text_input("Kolom", &st.chart_category_col)
            .width(60)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartCategoryCol(s))),
        dialog_field_label("Baris Data:"),
        text_input("Awal", &st.chart_data_start_row.to_string())
            .width(50)
            .size(12)
            .on_input(|s| {
                if let Ok(v) = s.parse::<usize>() {
                    Message::DialogMsg(DialogMsg::ChartDataStartRow(v))
                } else {
                    Message::DialogMsg(DialogMsg::ChartDataStartRow(2))
                }
            }),
        dialog_field_label("s/d"),
        text_input("Akhir", &st.chart_data_end_row.to_string())
            .width(50)
            .size(12)
            .on_input(|s| {
                if let Ok(v) = s.parse::<usize>() {
                    Message::DialogMsg(DialogMsg::ChartDataEndRow(v))
                } else {
                    Message::DialogMsg(DialogMsg::ChartDataEndRow(6))
                }
            }),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let series_1_row = row![
        dialog_field_label("Series 1:"),
        text_input("Kolom", &st.chart_series_1_col)
            .width(45)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartSeries1Col(s))),
        text_input("Label Seri 1", &st.chart_series_1_label)
            .width(Length::Fill)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartSeries1Label(s))),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let series_2_row = row![
        checkbox(st.chart_series_2_enabled)
            .label("Series 2:")
            .on_toggle(|b| Message::DialogMsg(DialogMsg::ChartSeries2Enabled(b))),
        text_input("Kolom", &st.chart_series_2_col)
            .width(45)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartSeries2Col(s))),
        text_input("Label Seri 2", &st.chart_series_2_label)
            .width(Length::Fill)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartSeries2Label(s))),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let series_3_row = row![
        checkbox(st.chart_series_3_enabled)
            .label("Series 3:")
            .on_toggle(|b| Message::DialogMsg(DialogMsg::ChartSeries3Enabled(b))),
        text_input("Kolom", &st.chart_series_3_col)
            .width(45)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartSeries3Col(s))),
        text_input("Label Seri 3", &st.chart_series_3_label)
            .width(Length::Fill)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::ChartSeries3Label(s))),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // 3. Visual formatting options
    let check_labels = checkbox(st.chart_show_labels)
        .label("Tampilkan Label Nilai")
        .on_toggle(|b| Message::DialogMsg(DialogMsg::ChartShowLabels(b)));

    let check_legend = checkbox(st.chart_show_legend)
        .label("Tampilkan Legenda")
        .on_toggle(|b| Message::DialogMsg(DialogMsg::ChartShowLegend(b)));

    let check_grid = checkbox(st.chart_show_grid)
        .label("Garis Kisi (Grid)")
        .on_toggle(|b| Message::DialogMsg(DialogMsg::ChartShowGrid(b)));

    let visual_toggles = row![check_labels, check_legend, check_grid].spacing(16);

    // 4. Map Palette
    let map_palettes = [
        "BPS Blue (Default)",
        "Flame Amber",
        "Emerald Green",
        "Crimson Red",
        "Purple Orchid",
        "Teal Cyan",
    ];
    let mut palette_row = row![dialog_field_label("Palet Peta:")].spacing(6).align_y(iced::Alignment::Center);
    for (idx, name) in map_palettes.iter().enumerate() {
        if idx >= 4 { break; }
        let selected = st.chart_map_palette == idx;
        let btn = button(text(*name).size(11))
            .padding([2, 6])
            .on_press(Message::DialogMsg(DialogMsg::ChartMapPalette(idx)))
            .style(move |theme: &Theme, _status| {
                let bg = if selected { theme::ACCENT.scale_alpha(0.25) } else { theme.palette().text.scale_alpha(0.05) };
                button::Style {
                    background: Some(bg.into()),
                    text_color: if selected { theme::ACCENT } else { theme.palette().text },
                    border: iced::Border {
                        radius: (3.0).into(),
                        width: if selected { 1.5 } else { 1.0 },
                        color: if selected { theme::ACCENT } else { theme.palette().text.scale_alpha(0.2) },
                    },
                    ..Default::default()
                }
            });
        palette_row = palette_row.push(btn);
    }

    // 5. Raw CSV Tabular data
    let csv_header = row![
        dialog_field_label("Editor Data Mentah (CSV):"),
        iced::widget::space().width(Length::Fill),
        button(text("Muat Sampel Data").size(11))
            .padding([2, 8])
            .on_press(Message::DialogMsg(DialogMsg::ChartLoadSample)),
    ]
    .align_y(iced::Alignment::Center);

    let csv_input = text_input(
        "Tahun,Nilai...",
        &st.chart_raw_csv,
    )
    .size(11)
    .on_input(|s| Message::DialogMsg(DialogMsg::ChartRawCsv(s)));

    let body = column![
        dialog_section_title("1. Jenis & Nomenklatur Grafik"),
        type_row,
        title_id_row,
        title_en_row,
        dialog_section_title("2. Pemetaan Kolom & Series Data"),
        cat_col,
        series_1_row,
        series_2_row,
        series_3_row,
        dialog_section_title("3. Opsi Visual & Legenda"),
        visual_toggles,
        palette_row,
        dialog_section_title("4. Sumber Data Tabular"),
        csv_header,
        csv_input,
    ]
    .spacing(8);

    let footer_actions = row![
        button(text("Generate Peta Choropleth").size(12))
            .padding([4, 12])
            .on_press(Message::DialogMsg(DialogMsg::ChartGenerateChoropleth)),
        button(text("Generate Grafik Vektor (SVG)").size(12))
            .padding([4, 14])
            .on_press(Message::DialogMsg(DialogMsg::ChartGenerateSvg))
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
        "📊",
        "Opsi Grafik & Peta Vektor Statistik (Vector Chart Options)".to_string(),
        "Konfigurasi jenis grafik statistik, kolom series, format angka, dan data spasial".to_string(),
        body.into(),
        footer_actions.into(),
        dark,
        640.0,
        720.0,
    )
}
