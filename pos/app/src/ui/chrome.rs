// Borderless window chrome: 8-edge resize zones, title bar drag, caption buttons.
// Port of the legacy chrome.rs (see iced_migration_plan.md Phase 3).

use iced::{
    mouse,
    widget::{button, canvas, column, image, mouse_area, row, space},
    window::Direction,
    Element, Length, Point, Size, Theme,
};

use super::{message::ChromeMsg, Message};

pub const APP_BAR_HEIGHT: f32 = 36.0;
const CAPTION_BUTTON_W: f32 = 46.0;
const EDGE: f32 = 4.0;
const CORNER: f32 = 10.0;

fn cursor_for(dir: Direction) -> mouse::Interaction {
    match dir {
        Direction::North | Direction::South => mouse::Interaction::ResizingVertically,
        Direction::East | Direction::West => mouse::Interaction::ResizingHorizontally,
        Direction::NorthEast | Direction::SouthWest => mouse::Interaction::ResizingDiagonallyUp,
        Direction::NorthWest | Direction::SouthEast => mouse::Interaction::ResizingDiagonallyDown,
    }
}

fn resize_zone(width: impl Into<Length>, height: impl Into<Length>, dir: Direction) -> Element<'static, Message> {
    mouse_area(space().width(width).height(height))
        .on_press(Message::Chrome(ChromeMsg::BeginResize(dir)))
        .interaction(cursor_for(dir))
        .into()
}

/// Full-window overlay with 8 resize zones around edges and corners.
/// Center cell stays pass-through so content receives clicks.
pub fn resize_overlay(maximized: bool) -> Element<'static, Message> {
    if maximized {
        return space().width(Length::Fill).height(Length::Fill).into();
    }

    column![
        row![
            resize_zone(CORNER, CORNER, Direction::NorthWest),
            resize_zone(Length::Fill, EDGE, Direction::North),
            resize_zone(CORNER, CORNER, Direction::NorthEast),
        ],
        row![
            resize_zone(EDGE, Length::Fill, Direction::West),
            space().width(Length::Fill).height(Length::Fill),
            resize_zone(EDGE, Length::Fill, Direction::East),
        ],
        row![
            resize_zone(CORNER, CORNER, Direction::SouthWest),
            resize_zone(Length::Fill, EDGE, Direction::South),
            resize_zone(CORNER, CORNER, Direction::SouthEast),
        ],
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn caption_buttons(maximized: bool, dark: bool) -> Element<'static, Message> {
    row![
        caption_button(
            GlyphKind::Minimize,
            maximized,
            dark,
            Message::Chrome(ChromeMsg::Minimize),
        ),
        caption_button(
            GlyphKind::Maximize,
            maximized,
            dark,
            Message::Chrome(ChromeMsg::ToggleMaximize),
        ),
        caption_button(
            GlyphKind::Close,
            maximized,
            dark,
            Message::Chrome(ChromeMsg::Close),
        ),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

/// Application bar: draggable/double-clickable zone plus caption buttons.
#[allow(dead_code)]
pub fn title_bar(label: &str, maximized: bool, dark: bool) -> Element<'_, Message> {
    let drag_zone = mouse_area(
        row![
            image(crate::branding::iced_handle())
                .width(18)
                .height(18),
            iced::widget::text(label).size(12),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .padding([6, 12]),
    )
    .on_press(Message::Chrome(ChromeMsg::DragTitlebar))
    .on_double_click(Message::Chrome(ChromeMsg::ToggleMaximize))
    .interaction(mouse::Interaction::Grabbing);

    let caption = caption_buttons(maximized, dark);

    row![drag_zone, caption]
        .width(Length::Fill)
        .height(APP_BAR_HEIGHT)
        .into()
}

#[derive(Clone, Copy)]
enum GlyphKind {
    Minimize,
    Maximize,
    Close,
}

struct Glyph {
    kind: GlyphKind,
    maximized: bool,
    dark: bool,
}

impl canvas::Program<Message> for Glyph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let color = if self.dark {
            iced::Color::from_rgb8(210, 210, 210)
        } else {
            iced::Color::from_rgb8(50, 50, 50)
        };
        let stroke = canvas::Stroke::default().with_color(color).with_width(1.0);
        let center = frame.center();
        let icon = Size::new(10.0, 10.0);
        let left = center.x - 5.0;
        let top = center.y - 5.0;

        match self.kind {
            GlyphKind::Minimize => {
                let y = center.y + 1.0;
                let path = canvas::Path::new(|p| {
                    p.move_to(Point::new(left, y));
                    p.line_to(Point::new(left + icon.width, y));
                });
                frame.stroke(&path, stroke);
            }
            GlyphKind::Maximize => {
                frame.stroke_rectangle(Point::new(left, top), icon, stroke);
                if self.maximized {
                    // Restore: offset window outline behind the front box.
                    let bx = left + 2.5;
                    let by = top - 2.5;
                    let path = canvas::Path::new(|p| {
                        p.move_to(Point::new(bx, by));
                        p.line_to(Point::new(bx + icon.width, by));
                        p.line_to(Point::new(bx + icon.width, by + icon.height));
                        p.line_to(Point::new(bx + icon.width - 2.5, by + icon.height));
                    });
                    frame.stroke(&path, stroke);
                }
            }
            GlyphKind::Close => {
                let path = canvas::Path::new(|p| {
                    p.move_to(Point::new(left, top));
                    p.line_to(Point::new(left + icon.width, top + icon.height));
                    p.move_to(Point::new(left + icon.width, top));
                    p.line_to(Point::new(left, top + icon.height));
                });
                frame.stroke(&path, stroke);
            }
        }

        vec![frame.into_geometry()]
    }
}

fn caption_button(
    kind: GlyphKind,
    maximized: bool,
    dark: bool,
    msg: Message,
) -> Element<'static, Message> {
    let is_close = matches!(kind, GlyphKind::Close);
    let glyph = Glyph {
        kind,
        maximized,
        dark,
    };

    let btn = button(canvas(glyph).width(24).height(14))
        .width(CAPTION_BUTTON_W)
        .height(APP_BAR_HEIGHT)
        .padding(0)
        .on_press(msg)
        .style(move |theme: &Theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            let bg = if hovered {
                if is_close {
                    iced::Color::from_rgb8(232, 17, 35) // Native Windows Close Crimson
                } else if dark {
                    iced::Color::from_rgb8(55, 55, 55)
                } else {
                    iced::Color::from_rgb8(225, 225, 225)
                }
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

    btn.into()
}
