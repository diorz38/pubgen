// AI Translation Setup & Execution modal dialog.
// Adopted from `AiSetupDialog` in designcraft-pubgen.

use iced::{
    widget::{button, checkbox, column, container, row, text, text_input},
    Element, Length, Theme,
};

use super::super::message::{DialogMsg, Message};
use super::super::theme;
use super::super::UiApp;
use super::{dialog_field_label, dialog_frame, dialog_section_title, DialogKind};

pub fn ai_dialog<'a>(ui: &'a UiApp, dark: bool) -> Element<'a, Message> {
    let app = &ui.app;
    let st = &ui.dialog_state;

    // 1. Provider presets
    let providers = [
        "OpenAI / Compatible",
        "Groq Cloud (Fast)",
        "DeepSeek / Custom",
        "Offline Mock (Free)",
    ];
    let mut provider_row = row![dialog_field_label("Penyedia LLM:")].spacing(6).align_y(iced::Alignment::Center);
    for (idx, name) in providers.iter().enumerate() {
        let is_selected = st.ai_provider == idx;
        let btn = button(text(*name).size(11))
            .padding([3, 8])
            .on_press(Message::DialogMsg(DialogMsg::AiProvider(idx)))
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
        provider_row = provider_row.push(btn);
    }

    // 2. Base URL & Model
    let base_url_row = column![
        dialog_field_label("Base URL Endpoint:"),
        text_input("https://api.openai.com/v1", &app.llm_base)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::AiBaseUrl(s))),
    ]
    .spacing(2);

    let model_row = column![
        dialog_field_label("Model ID:"),
        text_input("e.g. gpt-4o-mini / llama-3.3-70b-versatile", &app.llm_model)
            .size(12)
            .on_input(|s| Message::DialogMsg(DialogMsg::AiModel(s))),
    ]
    .spacing(2);

    let key_row = column![
        dialog_field_label("API Key (Kunci Akses Rahasia):"),
        text_input("sk-...", &app.llm_key)
            .size(12)
            .secure(true)
            .on_input(|s| Message::DialogMsg(DialogMsg::AiApiKey(s))),
        text("Kosongkan API key untuk menggunakan penerjemah offline simulasi (ekspansi teks ~20%).")
            .size(10)
            .color(if dark { iced::Color::from_rgb8(150, 155, 165) } else { iced::Color::from_rgb8(120, 125, 135) }),
    ]
    .spacing(2);

    // 3. Mode options
    let check_bilingual = checkbox(st.ai_bilingual)
        .label("Mode Dwibahasa (Dual Language ID / EN)")
        .on_toggle(|b| Message::DialogMsg(DialogMsg::AiBilingual(b)));

    // 4. Project status summary
    let total_slots = app.project.slots.len();
    let translated_count = app.project.slots.iter().filter(|s| !s.source.is_empty() && !s.text.is_empty()).count();

    let stats_box = container(
        column![
            row![
                text("Status Konten Dokumen:").size(11).style(|_theme: &Theme| text::Style { color: Some(theme::ACCENT) }),
                iced::widget::space().width(Length::Fill),
                text(format!("{translated_count} dari {total_slots} slot telah diterjemahkan")).size(11),
            ],
            row![
                button(text("📖 Kelola Glosarium...").size(11))
                    .padding([2, 8])
                    .on_press(Message::OpenDialog(DialogKind::Glossary)),
                button(text("🔍 Lihat Diff Terjemahan...").size(11))
                    .padding([2, 8])
                    .on_press(Message::OpenDialog(DialogKind::Diff)),
            ]
            .spacing(8),
        ]
        .spacing(6),
    )
    .padding(8)
    .width(Length::Fill)
    .style(move |theme: &Theme| container::Style {
        background: Some(theme.palette().text.scale_alpha(0.04).into()),
        border: iced::Border {
            radius: (4.0).into(),
            width: 1.0,
            color: theme.palette().text.scale_alpha(0.15),
        },
        ..Default::default()
    });

    let body = column![
        dialog_section_title("1. Penyedia Layanan LLM & Kredensial"),
        provider_row,
        base_url_row,
        model_row,
        key_row,
        dialog_section_title("2. Opsi Terjemahan"),
        check_bilingual,
        dialog_section_title("3. Ringkasan & Alat Bantu"),
        stats_box,
    ]
    .spacing(8);

    let footer_actions = row![
        button(text("Terjemahkan Konten Proyek Sekarang").size(12))
            .padding([4, 14])
            .on_press(Message::DialogMsg(DialogMsg::AiTranslateProject))
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
        "🤖",
        "Pengaturan & Eksekusi Terjemahan AI (AI Translation)".to_string(),
        "Konfigurasi penyedia LLM (Groq, OpenAI, Custom) dan terjemahkan konten proyek".to_string(),
        body.into(),
        footer_actions.into(),
        dark,
        580.0,
        640.0,
    )
}

