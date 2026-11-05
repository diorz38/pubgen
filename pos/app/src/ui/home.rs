// Home / welcome page — port of panels::show_welcome_page,
// render_profile_cards_grid, render_recent_projects_list.

use iced::{
    alignment,
    widget::{button, column, container, mouse_area, row, scrollable, text},
    Element, Length, Point, Size, Theme,
};

use super::message::*;
use super::theme;
use super::UiApp;

pub fn home(ui: &UiApp) -> Element<'_, Message> {
    let dark = theme::is_dark(ui.app.theme, ui.system_dark);
    let lang = ui.app.ui_lang;
    let id = lang == crate::i18n::UiLang::Id;

    let hero_title = text("Welcome to Publication Orchestrator Studio")
        .size(26)
        .width(Length::Fill);

    let subtitle_text = if id {
        "Tata letak publikasi statistik cetak dan digital — cepat, terbuka, dan otomatis."
    } else {
        "Page layout for print and screen — fast, open and scriptable."
    };

    let hero = row![
        iced::widget::image(crate::branding::iced_handle())
            .width(56)
            .height(56),
        column![hero_title, text(subtitle_text).size(14)].spacing(4),
    ]
    .spacing(16)
    .align_y(iced::Alignment::Center);

    // 2. Action buttons row: Open file (.pubgen) and Import Manifest (XLSX)
    let open_label = if id {
        "📂 Buka Berkas (.pubgen)"
    } else {
        "📂 Open file (.pubgen)"
    };
    let import_label = if id {
        "📊 Impor Manifest (XLSX)"
    } else {
        "📊 Import Manifest (XLSX)"
    };

    let mut actions = row![
        pill_button(open_label, true, Message::HomeOpen),
        pill_button(import_label, false, Message::HomeImportXlsx),
    ]
    .spacing(10);

    if let Some(brs_path) = crate::actions::find_brs_sample() {
        let label = if id { "📰 Sampel BRS" } else { "📰 Sample BRS" };
        actions = actions.push(pill_button_style(
            label,
            false,
            Message::HomeSampleBrs(brs_path),
            theme::ACCENT,
            iced::Color::WHITE,
        ));
    }

    if let Some(dda_path) = crate::actions::find_dda_sample() {
        let label = if id { "📑 Sampel DDA" } else { "📑 Sample DDA" };
        actions = actions.push(pill_button_style(
            label,
            false,
            Message::HomeSampleDda(dda_path),
            theme::ACCENT,
            iced::Color::WHITE,
        ));
    }

    // 3. Profile cards (DDA BPS, BRS BPS, and General)
    let cards = row![
        profile_card(
            ProfileCard::Dda,
            "DDA BPS",
            "Daerah Dalam Angka",
            "A4 (210 × 297 mm) • Facing Pages",
            if dark {
                iced::Color::from_rgb8(130, 177, 255)
            } else {
                iced::Color::from_rgb8(25, 118, 210)
            },
            dark,
            Message::ProfileCardSelected("bps_dda", "BPS DDA"),
        ),
        profile_card(
            ProfileCard::Brs,
            "BRS BPS",
            "Berita Resmi Statistik",
            "A4 (210 × 297 mm) • Single Page",
            if dark {
                iced::Color::from_rgb8(255, 183, 77)
            } else {
                iced::Color::from_rgb8(230, 81, 0)
            },
            dark,
            Message::ProfileCardSelected("bps_brs", "BPS BRS"),
        ),
        profile_card(
            ProfileCard::Generic,
            "General",
            if id { "Laporan Standar Umum" } else { "Generic Publication Report" },
            "A4 (210 × 297 mm) • Single Page",
            if dark {
                iced::Color::from_rgb8(160, 160, 160)
            } else {
                iced::Color::from_rgb8(110, 110, 110)
            },
            dark,
            Message::ProfileCardSelected("generic_report", "Generic Report"),
        ),
    ]
    .spacing(16);

    // 4. Recent projects
    let recent = recent_projects(ui, dark, id);

    let content = column![
        hero,
        actions,
        section_title(if id {
            "Profil Publikasi"
        } else {
            "Publication Profiles"
        }),
        cards,
        section_title(if id {
            "Dokumen Terakhir Dibuka (*.pubgen)"
        } else {
            "Recent Opened Documents (*.pubgen)"
        }),
        recent,
    ]
    .spacing(16)
    .max_width(1080);

    let bg = if dark {
        iced::Color::from_rgb8(26, 28, 34)
    } else {
        iced::Color::from_rgb8(245, 247, 250)
    };

    let scroll = scrollable(
        container(content)
            .width(Length::Fill)
            .padding([28, 40]),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    container(scroll)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(bg.into()),
            ..Default::default()
        })
        .into()
}

fn section_title(label: &str) -> Element<'static, Message> {
    text(label.to_string())
        .size(16)
        .width(Length::Fill)
        .into()
}

/// Rounded pill action button (radius 16 like the legacy home buttons).
fn pill_button(label: &str, primary: bool, msg: Message) -> Element<'static, Message> {
    pill_button_style(
        label,
        primary,
        msg,
        theme::ACCENT,
        iced::Color::WHITE,
    )
}

fn pill_button_style(
    label: &str,
    primary: bool,
    msg: Message,
    bg: iced::Color,
    fg: iced::Color,
) -> Element<'static, Message> {
    button(text(label.to_string()).size(13))
        .padding([7, 16])
        .on_press(msg)
        .style(move |theme: &Theme, status| {
            if primary {
                let base = bg;
                let fill = if matches!(status, button::Status::Hovered) {
                    theme::lighten(base, 0.12)
                } else {
                    base
                };
                button::Style {
                    background: Some(fill.into()),
                    text_color: fg,
                    border: iced::Border {
                        radius: (16.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            } else {
                let fill = if matches!(status, button::Status::Hovered) {
                    theme.palette().text.scale_alpha(0.08)
                } else {
                    iced::Color::TRANSPARENT
                };
                button::Style {
                    background: Some(fill.into()),
                    text_color: theme.palette().text,
                    border: iced::Border {
                        radius: (16.0).into(),
                        width: 1.0,
                        color: theme.palette().text.scale_alpha(0.3),
                    },
                    ..Default::default()
                }
            }
        })
        .into()
}

// ---- Profile cards (canvas-drawn mini page mockups) ----

#[derive(Clone, Copy)]
enum ProfileCard {
    Dda,
    Brs,
    Generic,
}

struct CardProgram {
    kind: ProfileCard,
    dark: bool,
}

impl iced::widget::canvas::Program<Message> for CardProgram {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<iced::widget::canvas::Action<Message>> {
        if let iced::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
            if cursor.is_over(bounds) {
                return Some(iced::widget::canvas::Action::request_redraw());
            }
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke};

        let hovered = cursor.is_over(bounds);
        let mut frame = Frame::new(renderer, bounds.size());

        // Card background
        let bg = if hovered {
            if self.dark {
                iced::Color::from_rgb8(46, 50, 60)
            } else {
                iced::Color::from_rgb8(230, 234, 242)
            }
        } else if self.dark {
            iced::Color::from_rgb8(36, 39, 47)
        } else {
            iced::Color::from_rgb8(238, 241, 246)
        };
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg);

        // Border
        let border_color = if hovered {
            theme::ACCENT
        } else if self.dark {
            iced::Color::from_rgb8(45, 48, 56)
        } else {
            iced::Color::from_rgb8(210, 210, 210)
        };
        frame.stroke_rectangle(
            Point::new(0.5, 0.5),
            Size::new(bounds.width - 1.0, bounds.height - 1.0),
            Stroke::default()
                .with_color(border_color)
                .with_width(if hovered { 1.5 } else { 1.0 }),
        );

        let page_white = iced::Color::WHITE;
        let page_edge = Stroke::default()
            .with_color(iced::Color::from_rgb8(190, 190, 190))
            .with_width(0.8);
        let accent = Stroke::default().with_color(theme::ACCENT).with_width(1.0);
        let center_x = bounds.width / 2.0;
        let page_h = 42.0;
        let p_top = 12.0;

        match self.kind {
            ProfileCard::Dda => {
                // Two white facing pages
                let p_w = 26.0;
                let left_x = center_x - p_w - 2.0;
                let right_x = center_x + 2.0;
                let left = Size::new(p_w, page_h);
                let right = Size::new(p_w, page_h);
                frame.fill_rectangle(Point::new(left_x, p_top), left, page_white);
                frame.stroke_rectangle(Point::new(left_x, p_top), left, page_edge);
                frame.fill_rectangle(Point::new(right_x, p_top), right, page_white);
                frame.stroke_rectangle(Point::new(right_x, p_top), right, page_edge);
                // Guide marks
                frame.stroke(
                    &Path::line(
                        Point::new(left_x + 4.0, p_top + page_h - 6.0),
                        Point::new(left_x + p_w - 4.0, p_top + page_h - 6.0),
                    ),
                    accent,
                );
                frame.stroke(
                    &Path::line(
                        Point::new(right_x + 4.0, p_top + 6.0),
                        Point::new(right_x + p_w - 4.0, p_top + 6.0),
                    ),
                    accent,
                );
            }
            ProfileCard::Brs => {
                // Single page with hero banner
                let s_w = 32.0;
                let sx = center_x - s_w / 2.0;
                let sy = p_top;
                let s = Size::new(s_w, page_h);
                frame.fill_rectangle(Point::new(sx, sy), s, page_white);
                frame.stroke_rectangle(Point::new(sx, sy), s, page_edge);
                // Blue header banner + orange line
                frame.fill_rectangle(
                    Point::new(sx, sy),
                    Size::new(s_w, 11.0),
                    iced::Color::from_rgb8(0, 114, 188),
                );
                frame.stroke(
                    &Path::line(
                        Point::new(sx, sy + 11.0),
                        Point::new(sx + s_w, sy + 11.0),
                    ),
                    Stroke::default()
                        .with_color(iced::Color::from_rgb8(245, 135, 31))
                        .with_width(1.0),
                );
                // Text blocks
                frame.fill_rectangle(
                    Point::new(sx + 3.0, sy + 15.0),
                    Size::new(s_w - 6.0, 3.0),
                    iced::Color::from_rgb8(180, 180, 180),
                );
                frame.fill_rectangle(
                    Point::new(sx + 3.0, sy + 21.0),
                    Size::new(s_w - 6.0, 14.0),
                    iced::Color::from_rgb8(240, 244, 250),
                );
            }
            ProfileCard::Generic => {
                // Single page with ruled lines
                let s_w = 32.0;
                let sx = center_x - s_w / 2.0;
                let sy = p_top;
                let s = Size::new(s_w, page_h);
                frame.fill_rectangle(Point::new(sx, sy), s, page_white);
                frame.stroke_rectangle(Point::new(sx, sy), s, page_edge);
                let line = Stroke::default()
                    .with_color(iced::Color::from_rgb8(210, 210, 210))
                    .with_width(1.0);
                for i in 0..4 {
                    let ly = sy + 9.0 + (i as f32 * 7.0);
                    frame.stroke(
                        &Path::line(
                            Point::new(sx + 4.0, ly),
                            Point::new(sx + s_w - 4.0, ly),
                        ),
                        line,
                    );
                }
            }
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if cursor.is_over(bounds) {
            iced::mouse::Interaction::Pointer
        } else {
            iced::mouse::Interaction::default()
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn profile_card(
    kind: ProfileCard,
    title: &'static str,
    line1: &'static str,
    line2: &'static str,
    accent_text: iced::Color,
    dark: bool,
    on_click: Message,
) -> Element<'static, Message> {
    let muted = iced::Color::from_rgb8(140, 140, 140);
    let program = CardProgram { kind, dark };
    let canvas = iced::widget::canvas(program)
        .width(Length::Fill)
        .height(66);

    let body = column![
        canvas,
        text(title)
            .size(13)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
        text(line1)
            .size(11)
            .color(muted)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
        text(line2)
            .size(11)
            .color(accent_text)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    ]
    .spacing(6)
    .padding([0, 8]);

    mouse_area(container(body).width(210).height(165))
        .on_press(on_click)
        .into()
}

// ---- Recent projects ----

fn recent_projects(ui: &UiApp, dark: bool, id: bool) -> Element<'_, Message> {
    if ui.app.recent_projects.is_empty() {
        let empty_bg = if dark {
            iced::Color::from_rgb8(34, 37, 44)
        } else {
            iced::Color::from_rgb8(240, 242, 247)
        };
        return container(
            row![
                text("ℹ").size(14).color(theme::ACCENT),
                text(if id {
                    "Belum ada riwayat berkas *.pubgen. Buka berkas proyek atau impor XLSX untuk memulai."
                } else {
                    "No *.pubgen history yet. Open a project file or import XLSX to get started."
                })
                .size(12),
            ]
            .spacing(8),
        )
        .width(Length::Fill)
        .padding([18, 24])
        .style(move |_theme: &Theme| container::Style {
            background: Some(empty_bg.into()),
            border: iced::Border {
                radius: (8.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into();
    }

    let list_bg = if dark {
        iced::Color::from_rgb8(33, 36, 43)
    } else {
        iced::Color::from_rgb8(240, 243, 248)
    };
    let border = if dark {
        iced::Color::from_rgb8(45, 48, 56)
    } else {
        iced::Color::from_rgb8(215, 215, 215)
    };

    let name_color = if dark {
        iced::Color::WHITE
    } else {
        iced::Color::from_rgb8(20, 20, 20)
    };
    let path_color = if dark {
        iced::Color::from_rgb8(150, 150, 150)
    } else {
        iced::Color::from_rgb8(110, 110, 110)
    };

    let rows = ui
        .app
        .recent_projects
        .iter()
        .map(|path| {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("project")
                .to_string();
            let path_display = path.display().to_string();
            let open_path = path.clone();
            row![
                text("📄").size(14),
                column![
                    text(name).size(13).color(name_color),
                    text(path_display).size(11).color(path_color),
                ]
                .spacing(1)
                .width(Length::Fill),
                small_button("Buka Proyek", Message::RecentOpened(open_path)),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding([6, 0])
            .into()
        })
        .collect::<Vec<_>>();

    container(column(rows).spacing(4))
        .width(Length::Fill)
        .padding([10, 14])
        .style(move |_theme: &Theme| container::Style {
            background: Some(list_bg.into()),
            border: iced::Border {
                radius: (8.0).into(),
                width: 1.0,
                color: border,
            },
            ..Default::default()
        })
        .into()
}

fn small_button(label: &str, msg: Message) -> Element<'static, Message> {
    button(text(label.to_string()).size(11))
        .padding([4, 10])
        .on_press(msg)
        .style(|theme: &Theme, status| {
            let bg = if matches!(status, button::Status::Hovered) {
                theme.palette().text.scale_alpha(0.1)
            } else {
                iced::Color::TRANSPARENT
            };
            button::Style {
                background: Some(bg.into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    radius: (4.0).into(),
                    width: 1.0,
                    color: theme.palette().text.scale_alpha(0.3),
                },
                ..Default::default()
            }
        })
        .into()
}
