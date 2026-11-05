// Onboarding Wizard modal dialog (A/B Test Variant B).
// Step-by-step guided setup for publication profile, layout rules, and metadata.

use iced::{
    widget::{button, column, container, row, rule, text, text_input},
    Element, Length, Theme,
};

use super::super::message::{DialogMsg, Message};
use super::super::theme;
use super::super::UiApp;
use super::{dialog_field_label, dialog_frame, dialog_section_title};

pub fn wizard_dialog<'a>(ui: &'a UiApp, dark: bool) -> Element<'a, Message> {
    let st = &ui.dialog_state;
    let step = st.wizard_step.max(1).min(3);
    let selected_profile = if st.wizard_profile_id.is_empty() {
        "bps_dda"
    } else {
        st.wizard_profile_id.as_str()
    };

    // Step indicators header
    let step_pills = row![
        step_badge(1, "1. Pilih Profil", step == 1, step > 1, dark),
        text("→").size(14).style(|theme: &Theme| text::Style {
            color: Some(theme.palette().text.scale_alpha(0.4)),
        }),
        step_badge(2, "2. Review Geometri", step == 2, step > 2, dark),
        text("→").size(14).style(|theme: &Theme| text::Style {
            color: Some(theme.palette().text.scale_alpha(0.4)),
        }),
        step_badge(3, "3. Judul & Konfirmasi", step == 3, false, dark),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    let (step_content, footer_actions) = match step {
        1 => step_1_profile_selection(selected_profile, dark),
        2 => step_2_geometry_review(selected_profile, dark),
        _ => step_3_metadata_confirmation(st, selected_profile, dark),
    };

    let body = column![
        step_pills,
        rule::horizontal(1).style(|theme: &Theme| rule::Style {
            color: theme.palette().text.scale_alpha(0.12),
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }),
        step_content,
    ]
    .spacing(16)
    .width(Length::Fill);

    dialog_frame(
        "✨",
        "Panduan Memulai Publikasi Baru (Onboarding Wizard)".to_string(),
        format!("Langkah {} dari 3 — Konfigurasi profil standar dan geometri publikasi", step),
        body.into(),
        footer_actions.into(),
        dark,
        640.0,
        620.0,
    )
}

fn step_badge(num: usize, label: &'static str, active: bool, completed: bool, _dark: bool) -> Element<'static, Message> {
    let bg = if active {
        theme::ACCENT
    } else if completed {
        iced::Color::from_rgb8(46, 125, 50) // Green
    } else {
        iced::Color::from_rgba(0.5, 0.5, 0.5, 0.2)
    };

    let fg = if active || completed {
        iced::Color::WHITE
    } else {
        iced::Color::from_rgba(0.7, 0.7, 0.7, 0.8)
    };

    let num_str = match num {
        1 => "1",
        2 => "2",
        3 => "3",
        _ => "?",
    };

    container(
        row![
            container(text(if completed { "✓" } else { num_str }).size(11))
                .padding([2, 6])
                .style(move |_theme: &Theme| container::Style {
                    background: Some(bg.into()),
                    border: iced::Border {
                        radius: (10.0).into(),
                        ..Default::default()
                    },
                    text_color: Some(fg),
                    ..Default::default()
                }),
            text(label).size(12).style(move |theme: &Theme| text::Style {
                color: if active {
                    Some(theme::ACCENT)
                } else {
                    Some(theme.palette().text.scale_alpha(0.7))
                },
            }),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .into()
}

fn step_1_profile_selection(selected_profile: &str, dark: bool) -> (Element<'static, Message>, Element<'static, Message>) {
    let dda_card = profile_select_card(
        "bps_dda",
        "BPS Daerah Dalam Angka (DDA)",
        "Format spreads ganda (Facing Pages), margin resmi BPS, penomoran Romawi/Arab, dwibahasa ID/EN.",
        "A4 • Facing Pages • 20.2mm / 19.0mm",
        selected_profile == "bps_dda",
        dark,
    );

    let brs_card = profile_select_card(
        "bps_brs",
        "BPS Berita Resmi Statistik (BRS)",
        "Format buletin rilis berkala halaman tunggal (Single Page), running header rilis resmi, Bahasa Indonesia.",
        "A4 • Single Page • Margin 18.0mm / 20.0mm",
        selected_profile == "bps_brs",
        dark,
    );

    let generic_card = profile_select_card(
        "generic_report",
        "Laporan Publikasi Umum (Custom)",
        "Tata letak fleksibel tanpa penimpaan aturan otomatis BPS, cocok untuk publikasi kustom.",
        "A4 • Single Page • Margin Standar",
        selected_profile == "generic_report",
        dark,
    );

    let content = column![
        dialog_section_title("Pilih Standar Format Publikasi:"),
        text("Pilihan profil akan menginisialisasi margin, header/footer, dan struktur halaman secara otomatis.")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.palette().text.scale_alpha(0.65)),
            }),
        dda_card,
        brs_card,
        generic_card,
    ]
    .spacing(12);

    let footer = row![
        button(text("Batal").size(12))
            .padding([7, 16])
            .on_press(Message::DialogMsg(DialogMsg::WizardCancel))
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(iced::Color::TRANSPARENT.into()),
                text_color: iced::Color::from_rgb8(180, 80, 80),
                ..Default::default()
            }),
        iced::widget::space().width(Length::Fill),
        button(text("Lanjut ke Penataan Geometri →").size(12))
            .padding([7, 20])
            .on_press(Message::DialogMsg(DialogMsg::WizardNext))
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(theme::ACCENT.into()),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .width(Length::Fill)
    .align_y(iced::Alignment::Center);

    (content.into(), footer.into())
}

fn profile_select_card(
    id: &'static str,
    title: &'static str,
    desc: &'static str,
    specs: &'static str,
    is_selected: bool,
    dark: bool,
) -> Element<'static, Message> {
    let accent = theme::ACCENT;
    let card_content = column![
        row![
            text(if is_selected { "🔘" } else { "⚪" }).size(14),
            text(title).size(14),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        text(desc).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.palette().text.scale_alpha(0.75)),
        }),
        text(specs).size(11).style(move |_theme: &Theme| text::Style {
            color: if is_selected { Some(accent) } else { Some(iced::Color::from_rgba(0.5, 0.5, 0.5, 0.8)) },
        }),
    ]
    .spacing(4)
    .padding(12);

    button(card_content)
        .width(Length::Fill)
        .on_press(Message::DialogMsg(DialogMsg::WizardSelectProfile(id.to_string())))
        .style(move |theme: &Theme, _status| {
            let bg = if is_selected {
                if dark {
                    iced::Color::from_rgba(accent.r, accent.g, accent.b, 0.15)
                } else {
                    iced::Color::from_rgba(accent.r, accent.g, accent.b, 0.08)
                }
            } else if dark {
                iced::Color::from_rgb8(35, 38, 46)
            } else {
                iced::Color::from_rgb8(248, 249, 251)
            };

            let border_color = if is_selected {
                accent
            } else {
                theme.palette().text.scale_alpha(0.12)
            };

            button::Style {
                background: Some(bg.into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    radius: (6.0).into(),
                    width: if is_selected { 1.5 } else { 1.0 },
                    color: border_color,
                },
                ..Default::default()
            }
        })
        .into()
}

fn step_2_geometry_review(selected_profile: &str, dark: bool) -> (Element<'static, Message>, Element<'static, Message>) {
    let (profile_name, layout_type, margins, folios, extra) = match selected_profile {
        "bps_dda" => (
            "BPS Daerah Dalam Angka (DDA)",
            "Facing Pages / Halaman Ganda Berpasangan (Double Spread)",
            "Atas: 20.2 mm • Bawah: 20.0 mm • Dalam: 19.0 mm • Luar: 15.0 mm",
            "Angka Romawi Kecil (Pengantar/TOC) & Angka Arab (Bab Isi)",
            "BPS DDA Bilingual Footer: Judul Indonesia + Judul English + Tahun + Nomor Halaman Luar.",
        ),
        "bps_brs" => (
            "BPS Berita Resmi Statistik (BRS)",
            "Single Page / Halaman Tunggal Berurutan",
            "Atas: 18.0 mm • Bawah: 18.0 mm • Kiri: 20.0 mm • Kanan: 20.0 mm",
            "Angka Arab Standar Berurutan (Mulai Halaman 1)",
            "BPS BRS Running Header: Nomor Rilis + Tanggal Rilis + Judul Singkat.",
        ),
        _ => (
            "Laporan Publikasi Umum",
            "Single Page / Halaman Tunggal Standar",
            "Atas: 20.0 mm • Bawah: 20.0 mm • Kiri: 20.0 mm • Kanan: 20.0 mm",
            "Angka Arab Standar",
            "Header & Footer Fleksibel (Dapat dikonfigurasi bebas di Studio).",
        ),
    };

    let review_box = column![
        dialog_section_title(&format!("Spesifikasi Baku: {}", profile_name)),
        geometry_spec_row("Tata Letak Halaman:", layout_type),
        geometry_spec_row("Ukuran Kertas:", "A4 (210 × 297 mm) Standar Internasional"),
        geometry_spec_row("Batas Margin (Margin Rules):", margins),
        geometry_spec_row("Sistem Penomoran:", folios),
        geometry_spec_row("Header / Footer Dinamis:", extra),
    ]
    .spacing(10)
    .padding(14);

    let container_box = container(review_box)
        .width(Length::Fill)
        .style(move |theme: &Theme| container::Style {
            background: Some(if dark {
                iced::Color::from_rgb8(32, 35, 42).into()
            } else {
                iced::Color::from_rgb8(245, 247, 250).into()
            }),
            border: iced::Border {
                radius: (6.0).into(),
                width: 1.0,
                color: theme.palette().text.scale_alpha(0.12),
            },
            ..Default::default()
        });

    let content = column![
        text("Konfigurasi di bawah ini telah disesuaikan dengan Pedoman Publikasi Resmi:")
            .size(12)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.palette().text.scale_alpha(0.7)),
            }),
        container_box,
    ]
    .spacing(12);

    let footer = row![
        button(text("← Kembali").size(12))
            .padding([7, 16])
            .on_press(Message::DialogMsg(DialogMsg::WizardPrev))
            .style(|theme: &Theme, _status| button::Style {
                background: Some(theme.palette().text.scale_alpha(0.08).into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        iced::widget::space().width(Length::Fill),
        button(text("Lanjut ke Metadata Publikasi →").size(12))
            .padding([7, 20])
            .on_press(Message::DialogMsg(DialogMsg::WizardNext))
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(theme::ACCENT.into()),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .width(Length::Fill)
    .align_y(iced::Alignment::Center);

    (content.into(), footer.into())
}

fn geometry_spec_row(label: &'static str, value: &'static str) -> Element<'static, Message> {
    column![
        text(label).size(11).style(|theme: &Theme| text::Style {
            color: Some(theme.palette().text.scale_alpha(0.6)),
        }),
        text(value).size(12).style(|theme: &Theme| text::Style {
            color: Some(theme.palette().text),
        }),
    ]
    .spacing(2)
    .into()
}

fn step_3_metadata_confirmation<'a>(
    st: &'a super::DialogState,
    _selected_profile: &str,
    _dark: bool,
) -> (Element<'a, Message>, Element<'a, Message>) {
    let title_input = text_input(
        "Contoh: Provinsi Jawa Tengah Dalam Angka 2026",
        &st.wizard_title,
    )
    .on_input(|s| Message::DialogMsg(DialogMsg::WizardTitle(s)))
    .padding(8);

    let year_input = text_input("Contoh: 2026", &st.wizard_year)
        .on_input(|s| Message::DialogMsg(DialogMsg::WizardYear(s)))
        .padding(8)
        .width(160);

    let content = column![
        dialog_section_title("Masukkan Informasi Awal Publikasi:"),
        column![
            dialog_field_label("Judul Publikasi / Dokumen:"),
            title_input,
        ]
        .spacing(4),
        column![
            dialog_field_label("Tahun Terbit Publikasi:"),
            year_input,
        ]
        .spacing(4),
        iced::widget::space().height(8),
        container(
            row![
                text("💡").size(16),
                text("Setelah menekan tombol 'Buat Proyek & Buka Studio', seluruh template, margin, dan sistem slot akan siap digunakan di kanvas editor.")
                    .size(12)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.palette().text.scale_alpha(0.75)),
                    }),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center)
        )
        .padding(10)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.palette().text.scale_alpha(0.05).into()),
            border: iced::Border {
                radius: (6.0).into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    ]
    .spacing(14);

    let footer = row![
        button(text("← Kembali").size(12))
            .padding([7, 16])
            .on_press(Message::DialogMsg(DialogMsg::WizardPrev))
            .style(|theme: &Theme, _status| button::Style {
                background: Some(theme.palette().text.scale_alpha(0.08).into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        iced::widget::space().width(Length::Fill),
        button(text("🚀 Buat Proyek & Buka Studio").size(12))
            .padding([7, 24])
            .on_press(Message::DialogMsg(DialogMsg::WizardFinish))
            .style(|_theme: &Theme, _status| button::Style {
                background: Some(theme::ACCENT.into()),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    ]
    .width(Length::Fill)
    .align_y(iced::Alignment::Center);

    (content.into(), footer.into())
}
