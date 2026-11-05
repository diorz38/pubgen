// Left panel: sources, content tree, and metadata (Document Settings).
// Automation & content tools have moved to top-level menus and dialogs.

use iced::{
    widget::{button, column, container, row, scrollable, text},
    Element, Length, Theme,
};

use crate::i18n::tr;
use super::super::message::*;
use super::super::theme;
use super::super::widgets;
use super::super::UiApp;

pub fn left(ui: &UiApp) -> Element<'_, Message> {
    let app = &ui.app;
    let lang = app.ui_lang;
    let dark = theme::is_dark(app.theme, ui.system_dark);

    let header = row![
        text(tr(lang, "source_and_content")).size(16),
        space_fill(),
        icon_button("💾", Message::Action("file_save"), tr(lang, "save_project")),
        icon_button("⌂", Message::GoHome, tr(lang, "tip_home")),
    ]
    .align_y(iced::Alignment::Center);

    let import_row = row![
        text_btn(format!("+ {}", tr(lang, "import_sources")), tr(lang, "tip_import_sources"), Message::Action("file_import_sources")),
        text_btn(format!("📊 {}", tr(lang, "import_excel_meta")), tr(lang, "tip_import_excel"), Message::Action("file_import_excel")),
        text_btn(format!("🎨 {}", tr(lang, "load_template")), tr(lang, "tip_load_template"), Message::Action("file_load_template")),
    ]
    .spacing(4);

    // Content tree, collapsible like the metadata form below it: one toggle for
    // the whole tree, plus one per entry group (csv/, md/, everything else).
    let content_tree: Element<'_, Message> = if app.project.content.is_empty() {
        let empty_btn = button(
            column![
                text(tr(lang, "no_sources")).size(12).color(theme::muted(dark)),
                text(tr(lang, "empty_sources_cta")).size(11).color(theme::ACCENT),
            ]
            .spacing(4)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .padding([12, 10])
        .on_press(Message::Action("file_import_sources"))
        .style(move |theme: &Theme, status| {
            let bg = if matches!(status, button::Status::Hovered) {
                theme.palette().text.scale_alpha(0.06)
            } else {
                iced::Color::TRANSPARENT
            };
            button::Style {
                background: Some(bg.into()),
                text_color: theme.palette().text,
                border: iced::Border {
                    radius: (6.0).into(),
                    width: 1.0,
                    color: if matches!(status, button::Status::Hovered) {
                        theme::ACCENT
                    } else {
                        theme.palette().text.scale_alpha(0.2)
                    },
                },
                ..Default::default()
            }
        });

        column![
            text(tr(lang, "content_tree")).size(13),
            empty_btn,
        ]
        .spacing(6)
        .into()
    } else {
        let mut csv_group = column![].spacing(2);
        let mut md_group = column![].spacing(2);
        let mut other_group = column![].spacing(2);
        let mut n_csv = 0usize;
        let mut n_md = 0usize;
        let mut n_other = 0usize;
        for (entry, bytes) in &app.project.content {
            let item = row![
                text(format!("• {entry}")).size(12),
                weak(format!("{} B", bytes.len()), dark),
            ]
            .spacing(6);
            if entry.starts_with("csv/") {
                csv_group = csv_group.push(item);
                n_csv += 1;
            } else if entry.starts_with("md/") {
                md_group = md_group.push(item);
                n_md += 1;
            } else {
                other_group = other_group.push(item);
                n_other += 1;
            }
        }

        let mut groups = column![].spacing(2);
        if n_csv > 0 {
            groups = groups.push(collapsible_titled(
                ui,
                "content_tree_csv",
                true,
                &format!("{} ({n_csv})", tr(lang, "tree_group_csv")),
                csv_group,
            ));
        }
        if n_md > 0 {
            groups = groups.push(collapsible_titled(
                ui,
                "content_tree_md",
                true,
                &format!("{} ({n_md})", tr(lang, "tree_group_md")),
                md_group,
            ));
        }
        if n_other > 0 {
            groups = groups.push(collapsible_titled(
                ui,
                "content_tree_other",
                true,
                &format!("{} ({n_other})", tr(lang, "tree_group_other")),
                other_group,
            ));
        }

        collapsible_titled(
            ui,
            "content_tree",
            true,
            &format!("{} ({})", tr(lang, "content_tree"), app.project.content.len()),
            groups,
        )
    };

    let panel_actions = row![
        text_btn(
            tr(lang, "action_collapse_all").to_string(),
            "",
            Message::PanelsCollapseAll,
        ),
        text_btn(
            tr(lang, "action_expand_basics").to_string(),
            "",
            Message::PanelsExpandBasics,
        ),
    ]
    .spacing(6);

    let scroll = scrollable(
        column![
            content_tree,
            weak(tr(lang, "panel_start_in_settings"), dark),
            panel_actions,
            group_header(tr(lang, "group_basic_documents")),
            section(tr(lang, "document_settings")),
            collapsible(ui, "metadata_form", false, super::super::metadata::metadata_form(ui)),
        ]
        .spacing(6),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    container(
        column![header, import_row, rule(), scroll]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(6),
    )
    .width(340)
    .height(Length::Fill)
    .padding([4, 8])
    .style(|_theme: &Theme| container::Style::default())
    .into()
}

// ---- shared small helpers ----

pub(super) fn section(label: &str) -> Element<'static, Message> {
    text(label.to_string()).size(13).into()
}

/// Group header inside the left scroll area ("Dokumen Dasar").
pub(super) fn group_header(label: &str) -> Element<'static, Message> {
    container(text(label.to_string()).size(12))
        .width(Length::Fill)
        .padding([4, 0])
        .style(|theme: &Theme| container::Style {
            text_color: Some(theme.palette().text.scale_alpha(0.75)),
            border: iced::Border {
                width: 0.0,
                radius: iced::border::Radius::default(),
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

pub(super) fn weak(label: impl Into<String>, _dark: bool) -> Element<'static, Message> {
    text(label.into()).size(11).into()
}

pub(super) fn space_fill() -> Element<'static, Message> {
    iced::widget::space().width(Length::Fill).into()
}

pub(super) fn rule() -> Element<'static, Message> {
    iced::widget::rule::horizontal(1)
        .style(|theme: &Theme| iced::widget::rule::Style {
            color: theme.palette().text.scale_alpha(0.10),
            radius: iced::border::Radius::default(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        })
        .into()
}

pub(super) fn icon_button(glyph: &str, msg: Message, tooltip: &str) -> Element<'static, Message> {
    let button = button(text(glyph.to_string()).size(12))
        .padding([2, 7])
        .on_press(msg)
        .style(|theme: &Theme, status| {
            let (bg, text_color) = match status {
                button::Status::Hovered => (theme.palette().text.scale_alpha(0.1), theme.palette().text),
                button::Status::Pressed => (theme.palette().text.scale_alpha(0.2), theme::ACCENT),
                button::Status::Disabled => (
                    iced::Color::TRANSPARENT,
                    theme.palette().text.scale_alpha(0.4),
                ),
                _ => (iced::Color::TRANSPARENT, theme.palette().text),
            };
            button::Style {
                background: Some(bg.into()),
                text_color,
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });
    widgets::tooltip(button, tooltip)
}

/// Filled text button with explicit enabled state and press handler.
fn text_button(label: String, enabled: bool, press: Option<Message>) -> Element<'static, Message> {
    button(text(label).size(12))
        .padding([3, 8])
        .on_press_maybe(press)
        .style(move |theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered if enabled => theme.palette().text.scale_alpha(0.1),
                button::Status::Pressed if enabled => theme.palette().text.scale_alpha(0.2),
                _ => iced::Color::TRANSPARENT,
            };
            let text_color = if enabled {
                theme.palette().text
            } else {
                theme.palette().text.scale_alpha(0.4)
            };
            button::Style {
                background: Some(bg.into()),
                text_color,
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

pub(super) fn text_btn(label: String, tooltip: &str, msg: Message) -> Element<'static, Message> {
    widgets::tooltip(text_button(label, true, Some(msg)), tooltip)
}

pub(super) fn text_btn_maybe(
    label: String,
    tooltip: &str,
    msg: Option<Message>,
) -> Element<'static, Message> {
    let enabled = msg.is_some();
    widgets::tooltip(text_button(label, enabled, msg), tooltip)
}

pub(super) fn text_btn_state(
    label: String,
    tooltip: &str,
    enabled: bool,
    msg: Message,
) -> Element<'static, Message> {
    widgets::tooltip(text_button(label, enabled, Some(msg)), tooltip)
}

pub(super) fn collapsible<'a>(
    ui: &'a UiApp,
    id: &'static str,
    default_open: bool,
    content: impl Into<iced::Element<'a, Message>>,
) -> Element<'a, Message> {
    collapsible_titled(ui, id, default_open, id, content)
}

/// Like [`collapsible`] but with a dynamic header label (e.g. a group name with
/// its item count).
pub(super) fn collapsible_titled<'a>(
    ui: &'a UiApp,
    id: &'static str,
    default_open: bool,
    title: &str,
    content: impl Into<iced::Element<'a, Message>>,
) -> Element<'a, Message> {
    let touched = ui.touched_panels.contains(id);
    let open = if touched { ui.open_panels.contains(id) } else { default_open };
    widgets::collapsible(title, open, Message::PanelToggled(id), content)
}
