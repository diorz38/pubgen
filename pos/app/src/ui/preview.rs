// Visual Live Preview tab — port of render_visual_live_preview + the three
// Legacy canvases (cover / chart / spreads) as iced canvas::Programs.

use iced::{
    alignment,
    widget::{button, canvas, column, container, row, scrollable, text},
    Element, Length, Point, Size, Theme,
};

use crate::covers::CoverStyle;
use crate::i18n::UiLang;
use crate::main_ui::PreviewMode;
use crate::metadata_form::PageKind;
use crate::vector::ChartType;

use super::canvas_cache::{content_hash, CanvasCache};
use super::message::*;
use super::theme;
use super::UiApp;

pub fn preview(ui: &UiApp) -> Element<'_, Message> {
    let app = &ui.app;
    let dark = theme::is_dark(app.theme, ui.system_dark);
    let id = app.ui_lang == UiLang::Id;

    let mode_btn = |label: &'static str, target: PreviewMode| -> Element<'static, Message> {
        let selected = app.preview_mode == target;
        button(text(label).size(12))
            .padding([4, 10])
            .on_press(Message::PreviewModeSelected(target))
            .style(move |theme: &Theme, _status| {
                let bg = if selected {
                    theme.palette().text.scale_alpha(0.12)
                } else {
                    iced::Color::TRANSPARENT
                };
                button::Style {
                    background: Some(bg.into()),
                    text_color: theme.palette().text,
                    border: iced::Border {
                        radius: (4.0).into(),
                        width: if selected { 1.0 } else { 0.0 },
                        color: theme.palette().text.scale_alpha(0.3),
                    },
                    ..Default::default()
                }
            })
            .into()
    };

    let mut toolbar = row![
        mode_btn("🎨 Sampul Publikasi (Cover)", PreviewMode::Cover),
        mode_btn("📊 Grafik Vektor (Chart)", PreviewMode::Chart),
        mode_btn("📄 Alur Halaman (Spreads)", PreviewMode::Spreads),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    // Right-side controls per mode
    let tools: Element<'static, Message> = match app.preview_mode {
        PreviewMode::Cover => row![
            small_tool("Gaya Sampul:"),
            super::widgets::combo_row(
                CoverStyle::ALL.to_vec(),
                Some(app.cover_style),
                190.0,
                |st| Message::Combo(ComboMsg::CoverStyle(st)),
            ),
            small_tool_btn("📋 Salin SVG", Message::CoverSvgCopy),
            small_tool_btn("💾 Ekspor SVG...", Message::Action("tool_cover_svg")),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into(),
        PreviewMode::Chart => row![
            small_tool("Tipe Grafik:"),
            super::widgets::combo_row(
                vec![
                    ChartType::BarClustered,
                    ChartType::HorizontalBar,
                    ChartType::Line,
                    ChartType::Donut,
                    ChartType::Map,
                ],
                Some(app.chart_type),
                190.0,
                |ct| Message::Combo(ComboMsg::ChartType(ct)),
            ),
            small_tool_btn("+ Tambah ke Proyek", Message::Action("tool_vector_chart")),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into(),
        PreviewMode::Spreads => text(format!(
            "Total {} item halaman publikasi",
            app.project.metadata.pages.len()
        ))
        .size(11)
        .into(),
    };

    toolbar = toolbar.push(iced::widget::space().width(Length::Fill));
    toolbar = toolbar.push(tools);

    let area: Element<'_, Message> = match app.preview_mode {
        PreviewMode::Cover => {
            let meta = &app.project.metadata;
            let program = CoverCanvas {
                style: app.cover_style,
                title_id: blank_or(&meta.title, "KABUPATEN DALAM ANGKA"),
                title_en: blank_or(&meta.title_en, "REGENCY IN FIGURES"),
                year: blank_or(&meta.date, "2026"),
                agency: blank_or(&meta.agency, "BADAN PUSAT STATISTIK"),
                bilingual: meta.is_bilingual(),
                catalogue: meta.catalogue.clone(),
                code: meta.code.clone(),
            };
            centered_canvas(canvas(program).width(340).height(340.0 * std::f32::consts::SQRT_2))
        }
        PreviewMode::Chart => {
            let program = ChartCanvas {
                dark,
                title: app.chart_title.clone(),
                chart_type: app.chart_type,
            };
            centered_canvas(canvas(program).width(480).height(280))
        }
        PreviewMode::Spreads => {
            if app.project.metadata.pages.is_empty() {
                text(if id {
                    "Belum ada halaman dalam publikasi ini. Buka proyek atau impor berkas XLSX untuk menampilkan alur halaman."
                } else {
                    "No pages in this publication yet. Open a project or import an XLSX to show the page flow."
                })
                .size(12)
                .into()
            } else {
                spreads_grid(ui, dark)
            }
        }
    };

    column![toolbar, rule(), scrollable(container(area).padding([8, 4]))]
        .spacing(6)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn rule() -> Element<'static, Message> {
    iced::widget::rule::horizontal(1)
        .style(|theme: &Theme| iced::widget::rule::Style {
            color: theme.palette().text.scale_alpha(0.2),
            radius: iced::border::Radius::default(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        })
        .into()
}

fn small_tool(label: &'static str) -> Element<'static, Message> {
    text(label).size(12).into()
}

fn small_tool_btn(label: &'static str, msg: Message) -> Element<'static, Message> {
    button(text(label).size(11))
        .padding([3, 8])
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
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn centered_canvas<P>(canvas: iced::widget::Canvas<P, Message>) -> Element<'static, Message>
where
    P: iced::widget::canvas::Program<Message> + 'static,
{
    container(canvas)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .into()
}

fn blank_or(s: &str, fallback: &'static str) -> String {
    if s.trim().is_empty() {
        fallback.to_string()
    } else {
        s.trim().to_string()
    }
}

// ---- Cover canvas ----

struct CoverCanvas {
    style: CoverStyle,
    title_id: String,
    title_en: String,
    year: String,
    agency: String,
    bilingual: bool,
    catalogue: String,
    code: String,
}

const PAPER_W: f32 = 340.0;

impl canvas::Program<Message> for CoverCanvas {
    type State = CanvasCache;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        use iced::advanced::text::Alignment as XAlign;
        use iced::widget::canvas::{Frame, Path, Stroke, Text};

        let key = content_hash(&(
            self.style,
            &self.title_id,
            &self.title_en,
            &self.year,
            &self.agency,
            self.bilingual,
            &self.catalogue,
            &self.code,
        ));

        state.draw(renderer, bounds, key, |frame| {
            let paper_h = bounds.height;
            let w = bounds.width;

            // Shadow
            frame.fill_rectangle(
                Point::new(3.0, 3.0),
                Size::new(w, paper_h),
                iced::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 35.0 / 255.0,
                },
            );
            // White paper
            frame.fill_rectangle(Point::ORIGIN, Size::new(w, paper_h), iced::Color::WHITE);
            frame.stroke_rectangle(
                Point::ORIGIN,
                Size::new(w, paper_h),
                Stroke::default()
                    .with_color(iced::Color::from_rgb8(210, 210, 210))
                    .with_width(1.0),
            );

            let left = 24.0;
            let label = |frame: &mut Frame,
                         content: &str,
                         x: f32,
                         y: f32,
                         size: f32,
                         color: iced::Color,
                         align: XAlign| {
                frame.fill_text(Text {
                    content: content.to_string(),
                    position: Point::new(x, y),
                    size: size.into(),
                    color,
                    align_x: align,
                    align_y: alignment::Vertical::Top,
                    ..Default::default()
                });
            };

            let title_id = self.title_id.clone();
            let title_en = self.title_en.clone();
            let year = self.year.clone();
            let agency = self.agency.clone();

            match self.style {
                CoverStyle::Horizon => {
                    let banner_h = paper_h * 0.44;
                    frame.fill_rectangle(
                        Point::ORIGIN,
                        Size::new(w, banner_h),
                        iced::Color::from_rgb8(0, 114, 188),
                    );
                    frame.fill_rectangle(
                        Point::new(0.0, banner_h - 4.0),
                        Size::new(w, 4.0),
                        iced::Color::from_rgb8(245, 135, 31),
                    );

                    let agency_y = banner_h + 20.0;
                    label(frame, &agency, left, agency_y, 10.5, iced::Color::from_rgb8(0, 114, 188), XAlign::Left);
                    let title1_y = agency_y + 24.0;
                    label(frame, &title_id, left, title1_y, 17.0, iced::Color::from_rgb8(15, 23, 42), XAlign::Left);
                    let mut year_y = title1_y + 30.0;
                    if self.bilingual {
                        label(frame, &title_en, left, title1_y + 24.0, 12.0, iced::Color::from_rgb8(100, 116, 139), XAlign::Left);
                        year_y += 24.0;
                    }
                    label(frame, &year, left, year_y + 12.0, 32.0, iced::Color::from_rgb8(245, 135, 31), XAlign::Left);

                    let line_y = paper_h - 32.0;
                    frame.stroke(
                        &Path::line(Point::new(left, line_y), Point::new(w - left, line_y)),
                        Stroke::default()
                            .with_color(iced::Color::from_rgb8(226, 232, 240))
                            .with_width(1.0),
                    );
                    let cat = format!("Katalog: {} | No. Publikasi: {}", self.catalogue, self.code);
                    label(frame, &cat, left, line_y + 8.0, 8.5, iced::Color::from_rgb8(120, 120, 120), XAlign::Left);
                }
                CoverStyle::Modern => {
                    let col_w = w * 0.35;
                    frame.fill_rectangle(
                        Point::ORIGIN,
                        Size::new(col_w, paper_h),
                        iced::Color::from_rgb8(15, 23, 42),
                    );
                    let rx = col_w + 20.0;
                    label(frame, &agency, rx, 40.0, 10.5, iced::Color::from_rgb8(0, 114, 188), XAlign::Left);
                    label(frame, &title_id, rx, 68.0, 17.0, iced::Color::from_rgb8(15, 23, 42), XAlign::Left);
                    label(frame, &year, rx, 110.0, 34.0, iced::Color::from_rgb8(26, 115, 232), XAlign::Left);
                }
                CoverStyle::Tricolor => {
                    let b = 10.0;
                    frame.fill_rectangle(Point::ORIGIN, Size::new(w, b), iced::Color::from_rgb8(0, 114, 188));
                    frame.fill_rectangle(Point::new(0.0, b), Size::new(w, b), iced::Color::from_rgb8(245, 135, 31));
                    frame.fill_rectangle(Point::new(0.0, b * 2.0), Size::new(w, b), iced::Color::from_rgb8(46, 125, 50));
                    label(frame, &agency, w / 2.0, 60.0, 11.0, iced::Color::from_rgb8(0, 114, 188), XAlign::Center);
                    label(frame, &title_id, w / 2.0, 90.0, 17.0, iced::Color::from_rgb8(15, 23, 42), XAlign::Center);
                    label(frame, &year, w / 2.0, 130.0, 36.0, iced::Color::from_rgb8(245, 135, 31), XAlign::Center);
                }
                CoverStyle::Minimal => {
                    let inset = 16.0;
                    frame.stroke_rectangle(
                        Point::new(inset, inset),
                        Size::new(w - inset * 2.0, paper_h - inset * 2.0),
                        Stroke::default()
                            .with_color(iced::Color::from_rgb8(15, 23, 42))
                            .with_width(1.0),
                    );
                    label(frame, &agency, w / 2.0, 50.0, 10.5, iced::Color::from_rgb8(71, 85, 105), XAlign::Center);
                    label(frame, &title_id, w / 2.0, 90.0, 18.0, iced::Color::from_rgb8(15, 23, 42), XAlign::Center);
                    label(frame, &year, w / 2.0, 135.0, 28.0, iced::Color::from_rgb8(15, 23, 42), XAlign::Center);
                }
            }

            let _ = PAPER_W;
        })
    }
}

// ---- Chart canvas ----

struct ChartCanvas {
    dark: bool,
    title: String,
    chart_type: ChartType,
}

impl canvas::Program<Message> for ChartCanvas {
    type State = CanvasCache;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        use iced::advanced::text::Alignment as XAlign;
        use iced::widget::canvas::{Path, Stroke, Text};

        let key = content_hash(&(self.dark, &self.title, self.chart_type));

        state.draw(renderer, bounds, key, |frame| {
            let bg = if self.dark {
                iced::Color::from_rgb8(33, 36, 44)
            } else {
                iced::Color::WHITE
            };
            let stroke_color = if self.dark {
                iced::Color::from_rgb8(55, 55, 55)
            } else {
                iced::Color::from_rgb8(215, 215, 215)
            };
            let text_color = if self.dark {
                iced::Color::WHITE
            } else {
                iced::Color::from_rgb8(20, 20, 20)
            };
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg);
            frame.stroke_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Stroke::default().with_color(stroke_color).with_width(1.0),
            );

            frame.fill_text(Text {
                content: self.title.clone(),
                position: Point::new(bounds.width / 2.0, 14.0),
                size: 13.5.into(),
                color: text_color,
                align_x: XAlign::Center,
                align_y: alignment::Vertical::Top,
                ..Default::default()
            });

            let plot = iced::Rectangle::new(
                Point::new(bounds.x + 40.0, bounds.y + 45.0),
                Size::new(bounds.width - 70.0, bounds.height - 80.0),
            );
            let grid = Stroke::default()
                .with_color(if self.dark {
                    iced::Color::from_rgb8(50, 50, 50)
                } else {
                    iced::Color::from_rgb8(230, 230, 230)
                })
                .with_width(0.8);

            match self.chart_type {
                ChartType::Donut => {
                    let center = Point::new(bounds.width / 2.0 - 40.0, bounds.height / 2.0 + 12.0);
                    frame.fill(&canvas::Path::circle(center, 65.0), iced::Color::from_rgb8(26, 115, 232));
                    frame.fill(&canvas::Path::circle(center, 42.0), bg);
                    frame.fill_text(Text {
                        content: "100%".into(),
                        position: center,
                        size: 13.0.into(),
                        color: text_color,
                        align_x: XAlign::Center,
                        align_y: alignment::Vertical::Center,
                        ..Default::default()
                    });
                    let legend = [
                        ("Sektor A (45%)", iced::Color::from_rgb8(26, 115, 232)),
                        ("Sektor B (35%)", iced::Color::from_rgb8(245, 135, 31)),
                        ("Sektor C (20%)", iced::Color::from_rgb8(52, 168, 83)),
                    ];
                    let lx = bounds.width / 2.0 + 50.0;
                    let ly = bounds.height / 2.0 - 30.0;
                    for (i, (lbl, col)) in legend.iter().enumerate() {
                        let y = ly + (i as f32 * 22.0);
                        frame.fill_rectangle(Point::new(lx, y), Size::new(12.0, 12.0), *col);
                        frame.fill_text(Text {
                            content: (*lbl).to_string(),
                            position: Point::new(lx + 18.0, y),
                            size: 11.0.into(),
                            color: text_color,
                            align_x: XAlign::Left,
                            align_y: alignment::Vertical::Top,
                            ..Default::default()
                        });
                    }
                }
                ChartType::Line => {
                    for i in 0..=4 {
                        let y = plot.y + (plot.height / 4.0) * i as f32;
                        frame.stroke(
                            &Path::line(Point::new(plot.x, y), Point::new(plot.x + plot.width, y)),
                            grid,
                        );
                    }
                    let vals = [72.0f32, 78.0, 84.5, 91.2, 95.0];
                    let pts: Vec<Point> = vals
                        .iter()
                        .enumerate()
                        .map(|(idx, v)| {
                            let x = plot.x + (plot.width / 4.0) * idx as f32;
                            let norm_y = (v - 60.0) / 40.0;
                            Point::new(x, plot.y + plot.height - (plot.height * norm_y))
                        })
                        .collect();
                    for pair in pts.windows(2) {
                        frame.stroke(
                            &Path::line(pair[0], pair[1]),
                            Stroke::default()
                                .with_color(iced::Color::from_rgb8(26, 115, 232))
                                .with_width(2.5),
                        );
                    }
                    for p in &pts {
                        frame.fill(&canvas::Path::circle(*p, 4.0), iced::Color::from_rgb8(26, 115, 232));
                        frame.fill(&canvas::Path::circle(*p, 2.0), iced::Color::WHITE);
                    }
                }
                _ => {
                    for i in 0..=4 {
                        let y = plot.y + (plot.height / 4.0) * i as f32;
                        frame.stroke(
                            &Path::line(Point::new(plot.x, y), Point::new(plot.x + plot.width, y)),
                            grid,
                        );
                    }
                    let cats = ["2022", "2023", "2024", "2025", "2026"];
                    let vals = [72.0f32, 78.0, 84.5, 91.2, 95.0];
                    let cat_w = plot.width / 5.0;
                    for (idx, cat) in cats.iter().enumerate() {
                        let cx = plot.x + (idx as f32 * cat_w) + cat_w * 0.5;
                        let bar_h = (vals[idx] / 100.0) * plot.height;
                        frame.fill_rectangle(
                            Point::new(cx - 14.0, plot.y + plot.height - bar_h),
                            Size::new(28.0, bar_h),
                            iced::Color::from_rgb8(26, 115, 232),
                        );
                        frame.fill_text(Text {
                            content: (*cat).to_string(),
                            position: Point::new(cx, plot.y + plot.height + 6.0),
                            size: 10.5.into(),
                            color: iced::Color::from_rgb8(140, 140, 140),
                            align_x: XAlign::Center,
                            align_y: alignment::Vertical::Top,
                            ..Default::default()
                        });
                    }
                }
            }
        })
    }
}

// ---- Spreads grid ----

fn spreads_grid(app: &UiApp, dark: bool) -> Element<'_, Message> {
    let pages = &app.app.project.metadata.pages;
    let mut rows = column![].spacing(8);
    let mut i = 0;
    while i < pages.len() {
        let mut r = row![].spacing(8);
        for (j, p) in pages.iter().skip(i).take(4).enumerate() {
            r = r.push(spread_card(
                p.kind,
                i + j,
                short(&p.title, 18),
                p.source.clone().map(|s| short(&s, 18)),
                dark,
            ));
        }
        rows = rows.push(r);
        i += 4;
    }
    rows.into()
}

fn short(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn spread_card(
    kind: PageKind,
    page_no: usize,
    title: String,
    source: Option<String>,
    dark: bool,
) -> Element<'static, Message> {
    use iced::advanced::text::Alignment as XAlign;
    use iced::widget::canvas::{Frame, Stroke, Text};

    struct Card {
        kind: PageKind,
        page_no: usize,
        title: String,
        source: Option<String>,
        dark: bool,
    }

    impl canvas::Program<Message> for Card {
        type State = CanvasCache;

        fn draw(
            &self,
            state: &Self::State,
            renderer: &iced::Renderer,
            _theme: &Theme,
            bounds: iced::Rectangle,
            _cursor: iced::mouse::Cursor,
        ) -> Vec<canvas::Geometry> {
            let key = content_hash(&(
                self.kind,
                self.page_no,
                &self.title,
                &self.source,
                self.dark,
            ));

            state.draw(renderer, bounds, key, |frame| {
                let bg = if self.dark {
                    iced::Color::from_rgb8(34, 37, 44)
                } else {
                    iced::Color::WHITE
                };
                let stroke = Stroke::default()
                    .with_color(if self.dark {
                        iced::Color::from_rgb8(55, 55, 55)
                    } else {
                        iced::Color::from_rgb8(215, 215, 215)
                    })
                    .with_width(1.0);
                frame.fill_rectangle(Point::ORIGIN, bounds.size(), bg);
                frame.stroke_rectangle(Point::ORIGIN, bounds.size(), stroke);

                let badge_col = super::metadata::kind_color(self.kind);
                let kind_label = self.kind.label_en();

                let cx = bounds.width / 2.0;
                let paper = Point::new(cx - 19.0, 18.0);
                frame.fill_rectangle(paper, Size::new(38.0, 52.0), iced::Color::from_rgb8(248, 250, 252));
                frame.stroke_rectangle(paper, Size::new(38.0, 52.0), Stroke::default().with_color(iced::Color::from_rgb8(190, 190, 190)).with_width(0.8));
                frame.fill_rectangle(paper, Size::new(38.0, 6.0), badge_col);

                let label = |frame: &mut Frame,
                             content: String,
                             y: f32,
                             size: f32,
                             color: iced::Color| {
                    frame.fill_text(Text {
                        content,
                        position: Point::new(cx, y),
                        size: size.into(),
                        color,
                        align_x: XAlign::Center,
                        align_y: alignment::Vertical::Top,
                        ..Default::default()
                    });
                };

                label(frame, format!("Hal. {}", self.page_no + 1), 78.0, 11.0, badge_col);
                label(frame, kind_label.to_string(), 96.0, 12.0, if self.dark { iced::Color::WHITE } else { iced::Color::from_rgb8(20, 20, 20) });
                label(frame, self.title.clone(), 116.0, 10.5, iced::Color::from_rgb8(140, 140, 140));
                label(frame, self.source.clone().unwrap_or_else(|| "—".into()), 138.0, 9.5, iced::Color::from_rgb8(120, 120, 120));
            })
        }
    }

    let program = Card {
        kind,
        page_no,
        title,
        source,
        dark,
    };
    canvas(program)
        .width(145)
        .height(175)
        .into()
}
