use iced::{
    widget::{button, column, container},
    Element, Length, Theme,
};

use super::menu::MenuDef;
use super::message::*;
use super::widgets;
use super::UiApp;

pub fn view(ui: &UiApp) -> Element<'_, Message> {
    let dark = super::theme::is_dark(ui.app.theme, ui.system_dark);
    let defs = super::menu::runtime_menus(ui);
    let top_bar = unified_top_bar(ui, &defs);

    let main_body: Element<'_, Message> = match ui.app.view {
        crate::main_ui::AppView::Home => super::home::home(ui),
        crate::main_ui::AppView::Studio => {
            let studio = iced::widget::row![
                super::panels::left(ui),
                vertical_rule(dark),
                super::panels::center(ui),
                vertical_rule(dark),
                super::panels::right(ui),
            ]
            .width(Length::Fill)
            .height(Length::Fill);

            column![studio, super::panels::bottom_bar(ui)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    };

    let root = column![top_bar, main_body]
        .width(Length::Fill)
        .height(Length::Fill);

    let overlay = super::chrome::resize_overlay(ui.maximized);

    let mut stack_layers = vec![root.into(), overlay];

    // Menu dropdown overlay when a menu header is open
    if let Some(open_idx) = ui.open_menu {
        if open_idx < defs.len() {
            stack_layers.push(super::menu::menu_overlay(
                &defs,
                open_idx,
                dark,
                ui.window_size,
                ui.menu_row,
            ));
        }
    }

    // Compile split-button dropdown (`▾`)
    if ui.compile_menu_open {
        stack_layers.push(super::menu::compile_menu_overlay(
            ui.app.ui_lang,
            ui.app.last_output.is_some(),
            ui.busy,
            dark,
        ));
    }

    if let Some(modal) = super::metadata::card_editor_modal(ui) {
        stack_layers.push(modal);
    }

    if let Some(dialog) = super::dialogs::active_dialog_modal(ui) {
        stack_layers.push(dialog);
    }

    if ui.busy {
        stack_layers.push(super::widgets::busy_overlay(ui, dark));
    }

    iced::widget::stack(stack_layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn vertical_rule(dark: bool) -> Element<'static, Message> {
    iced::widget::rule::vertical(1)
        .style(move |theme: &Theme| iced::widget::rule::Style {
            color: if dark {
                iced::Color::from_rgb8(42, 45, 52)
            } else {
                theme.palette().text.scale_alpha(0.15)
            },
            radius: iced::border::Radius::default(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        })
        .into()
}

/// Unified application top bar (replaces stacked OS titlebar and menu row).
/// Left: Home icon + Menus. Center: Draggable document title. Right: Compile + DC Pill + Lang + Window Captions.
fn unified_top_bar<'a>(ui: &'a UiApp, defs: &[MenuDef]) -> Element<'a, Message> {
    let dark = super::theme::is_dark(ui.app.theme, ui.system_dark);
    let is_home = ui.app.view == crate::main_ui::AppView::Home;

    // 1. Left side: Home button + Menus
    let home_btn = button(
        iced::widget::row![
            iced::widget::image(crate::branding::iced_handle())
                .width(16)
                .height(16),
            iced::widget::text("⌂").size(14),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 8])
    .on_press(Message::HomeToggled)
    .style(move |theme: &Theme, status| {
        let active_bg = if dark {
            iced::Color::from_rgb8(50, 50, 50)
        } else {
            iced::Color::from_rgb8(215, 215, 215)
        };
        let bg = if is_home {
            active_bg
        } else if matches!(status, button::Status::Hovered) {
            theme.palette().text.scale_alpha(0.08)
        } else {
            iced::Color::TRANSPARENT
        };
        button::Style {
            background: Some(bg.into()),
            text_color: if is_home {
                super::theme::ACCENT
            } else {
                theme.palette().text
            },
            border: iced::Border {
                radius: (4.0).into(),
                ..Default::default()
            },
            ..Default::default()
        }
    });

    let menu_bar = super::menu::menu_bar(defs, ui.open_menu, ui.menu_focus, dark);

    let left = iced::widget::row![home_btn, menu_bar]
        .spacing(4)
        .align_y(iced::Alignment::Center);

    // 2. Center: Draggable title area
    let title = iced::widget::text(format!("POS — {}", ui.app.project.name))
        .size(12)
        .color(if dark {
            iced::Color::from_rgb8(175, 175, 175)
        } else {
            iced::Color::from_rgb8(90, 90, 90)
        });

    let drag_center = iced::widget::mouse_area(
        container(title)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(Message::Chrome(ChromeMsg::DragTitlebar))
    .on_double_click(Message::Chrome(ChromeMsg::ToggleMaximize));

    // 3. Right side: DC status pill + Language picker + Caption buttons
    let lang = ui.app.ui_lang;
    let pill_msg = if matches!(ui.app.dc_status, crate::main_ui::DcStatus::Off) {
        Message::DcPillLaunched
    } else {
        Message::DcPillReprobe
    };
    let pill = widgets::pill::dc_pill(&ui.app.dc_status, lang, dark, pill_msg);

    let lang_pick = widgets::combo_row(
        crate::i18n::UiLang::ALL.to_vec(),
        Some(ui.app.ui_lang),
        60.0,
        |lang| Message::LangSelected(lang),
    );

    let caption = super::chrome::caption_buttons(ui.maximized, dark);

    let right = iced::widget::row![pill, lang_pick, caption]
        .spacing(6)
        .align_y(iced::Alignment::Center);

    container(
        iced::widget::row![left, drag_center, right]
            .width(Length::Fill)
            .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(super::chrome::APP_BAR_HEIGHT)
    .padding([0, 0])
    .style(move |theme: &Theme| container::Style {
        background: Some(if dark {
            iced::Color::from_rgb8(28, 30, 36)
        } else {
            iced::Color::from_rgb8(240, 242, 247)
        }.into()),
        border: iced::Border {
            width: 1.0,
            color: if dark {
                iced::Color::from_rgb8(42, 45, 52)
            } else {
                theme.palette().text.scale_alpha(0.12)
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

