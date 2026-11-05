// Declarative menu bar (iced has no built-in MenuBar).
//
// `menu_contract` is the single source of truth: the bar, the popovers, and
// the unit tests all read the same titles, ids, and labels. Runtime-only state
// (selected rows, busy, last-output) is layered on by `runtime_menus`.

use iced::{
    widget::{button, column, container, mouse_area, row, rule, scrollable, space, stack, text},
    Element, Length, Size, Theme,
};
use crate::i18n::{tr, UiLang};
use super::message::MenuMsg;
use super::theme;
use super::Message;

/// Gap between menu titles and between a bar edge and a title.
pub const MENU_GAP: f32 = 2.0;
/// Left edge of the menu bar inside the unified top bar (home button + gap).
pub const BAR_LEFT: f32 = 56.0;
/// Popover card width, inside the 260..=320 contract.
pub const POPOVER_WIDTH: f32 = 300.0;

#[derive(Clone)]
pub enum ItemKind {
    Action,
    Header,
    Separator,
    Selected(bool),
}

#[derive(Clone)]
pub struct MenuItem {
    pub id: &'static str,
    pub label: String,
    pub kind: ItemKind,
    pub enabled: bool,
    /// Localized explanation shown on hover when the row is disabled.
    pub reason: Option<String>,
}

impl MenuItem {
    pub fn action(id: &'static str, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            kind: ItemKind::Action,
            enabled: true,
            reason: None,
        }
    }

    pub fn action_disabled(
        id: &'static str,
        label: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            kind: ItemKind::Action,
            enabled: false,
            reason: Some(reason.into()),
        }
    }

    pub fn selected(id: &'static str, selected: bool, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            kind: ItemKind::Selected(selected),
            enabled: true,
            reason: None,
        }
    }

    pub fn header(label: impl Into<String>) -> Self {
        Self {
            id: "",
            label: label.into(),
            kind: ItemKind::Header,
            enabled: false,
            reason: None,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: "",
            label: String::new(),
            kind: ItemKind::Separator,
            enabled: false,
            reason: None,
        }
    }

    pub fn set_selected(&mut self, selected: bool) {
        if let ItemKind::Selected(_) = self.kind {
            self.kind = ItemKind::Selected(selected);
        }
    }

    /// Rows that can be highlighted and activated with the keyboard.
    pub fn is_actionable(&self) -> bool {
        matches!(self.kind, ItemKind::Action | ItemKind::Selected(_))
    }
}

#[derive(Clone)]
pub struct MenuDef {
    pub title: String,
    pub items: Vec<MenuItem>,
}

/// The declarative menu contract: 8 menus, ids, order, and localized labels.
pub fn menu_contract(lang: UiLang) -> Vec<MenuDef> {
    use crate::main_ui::{AppTheme, DC_PORT};

    let item = |key: &str| tr(lang, key).to_string();

    let mut publication = vec![
        MenuItem::action("pub_generate", item("btn_generate")),
        MenuItem::action("pub_batch_fill", item("btn_generate_publish")),
        MenuItem::separator(),
        MenuItem::header(item("menu_header_profiles")),
    ];
    for (id, key) in [
        ("profile_bps_brs", "menu_profile_bps_brs"),
        ("profile_bps_dda", "menu_profile_bps_dda"),
        ("profile_generic_report", "menu_profile_generic"),
        ("profile_book", "menu_profile_book"),
        ("profile_infographic", "menu_profile_infographic"),
        ("profile_magazine", "menu_profile_magazine"),
        ("profile_newsletter", "menu_profile_newsletter"),
        ("profile_slide", "menu_profile_slide"),
    ] {
        publication.push(MenuItem::selected(id, false, item(key)));
    }
    publication.extend([
        MenuItem::separator(),
        MenuItem::action("tool_vector_chart", item("menu_item_vector_chart")),
        MenuItem::action("tool_toc", item("menu_item_toc")),
        MenuItem::action("tool_cover_svg", item("menu_item_cover")),
        MenuItem::action_disabled(
            "header_footer_info",
            item("menu_item_header_footer"),
            item("menu_reason_header_footer"),
        ),
        MenuItem::action("pub_qa_audit", item("menu_item_qa")),
    ]);

    let mut view = vec![
        MenuItem::selected("view_home", false, item("menu_item_view_home")),
        MenuItem::selected("view_studio", false, item("menu_item_view_studio")),
        MenuItem::separator(),
        MenuItem::selected("tab_content", false, item("menu_item_tab_content")),
        MenuItem::selected("tab_preview", false, item("menu_item_tab_preview")),
        MenuItem::selected("tab_log", false, item("menu_item_tab_log")),
        MenuItem::separator(),
    ];
    for theme in AppTheme::ALL {
        let id = match theme {
            AppTheme::System => "theme_system",
            AppTheme::Light => "theme_light",
            AppTheme::Dark => "theme_dark",
        };
        view.push(MenuItem::selected(
            id,
            false,
            format!("{} {}", theme.icon(), theme.label(lang)),
        ));
    }
    view.push(MenuItem::separator());
    for lang_option in UiLang::ALL {
        let id = match lang_option {
            UiLang::Id => "lang_id",
            UiLang::En => "lang_en",
        };
        view.push(MenuItem::selected(
            id,
            false,
            format!("{} {}", lang_option.flag(), lang_option.label()),
        ));
    }

    vec![
        MenuDef {
            title: item("menu_file"),
            items: vec![
                MenuItem::action("file_new", item("menu_item_new_project")),
                MenuItem::action("file_open", item("menu_item_open_project")),
                MenuItem::action("file_save", item("menu_item_save_project")),
                MenuItem::separator(),
                MenuItem::action("file_load_template", item("menu_item_load_template")),
                MenuItem::separator(),
                MenuItem::action_disabled(
                    "file_open_pdf",
                    item("menu_item_open_pdf"),
                    item("menu_reason_no_output"),
                ),
                MenuItem::separator(),
                MenuItem::action("app_exit", item("menu_item_exit")),
            ],
        },
        MenuDef {
            title: item("menu_publication"),
            items: publication,
        },
        MenuDef {
            title: item("menu_publication_tools"),
            items: vec![
                MenuItem::header(item("menu_header_ingest_tables")),
                MenuItem::action("file_import_sources", item("menu_item_import_sources")),
                MenuItem::action("file_import_excel", item("menu_item_import_excel")),
                MenuItem::separator(),
                MenuItem::header(item("menu_header_page_layout")),
                MenuItem::action(
                    "open_document_settings",
                    item("menu_item_open_settings"),
                ),
                MenuItem::separator(),
                MenuItem::action("pub_translate", item("menu_item_translate")),
                MenuItem::action(
                    "tool_glossary_import",
                    item("menu_item_glossary_import"),
                ),
                MenuItem::action(
                    "tool_glossary_export",
                    item("menu_item_glossary_export"),
                ),
            ],
        },
        MenuDef {
            title: item("menu_automation_content"),
            items: vec![
                MenuItem::action("auto_cover_studio", item("menu_item_cover_dialog")),
                MenuItem::action("auto_vector_chart", item("menu_item_chart_dialog")),
                MenuItem::action("auto_toc", item("menu_item_toc_dialog")),
                MenuItem::separator(),
                MenuItem::action("tool_ai_translate", item("menu_item_ai_translate_dialog")),
                MenuItem::action("tool_glossary", item("menu_item_glossary_dialog")),
                MenuItem::action("tool_diff", item("menu_item_diff_dialog")),
                MenuItem::separator(),
                MenuItem::action("auto_translate", item("menu_item_translate")),
            ],
        },
        MenuDef {
            title: item("menu_designcraft"),
            items: vec![
                MenuItem::action("dc_launch", item("menu_item_dc_launch")),
                MenuItem::action("dc_kill", item("menu_item_dc_kill")),
                MenuItem::action("dc_open_doc", item("menu_item_dc_open")),
                MenuItem::action(
                    "dc_reconnect",
                    format!("{} · {DC_PORT}", item("menu_item_dc_reconnect")),
                ),
                MenuItem::separator(),
                MenuItem::action("dc_auto_extract", item("menu_item_dc_extract")),
            ],
        },
        MenuDef {
            title: item("menu_view"),
            items: view,
        },
        MenuDef {
            title: item("menu_shortcut"),
            items: vec![
                MenuItem::action("pub_batch_x4", item("menu_item_export_x4")),
                MenuItem::action("file_open_pdf", item("menu_item_open_pdf")),
                MenuItem::action("file_save", item("menu_item_save_project")),
                MenuItem::action("file_open", item("menu_item_open_project")),
                MenuItem::action("file_new", item("menu_item_new_project")),
            ],
        },
        MenuDef {
            title: item("menu_help"),
            items: vec![
                MenuItem::action("help_bps", item("menu_item_help_bps")),
                MenuItem::action("help_ipc", item("menu_item_help_ipc")),
                MenuItem::separator(),
                MenuItem::action("help_about", item("menu_item_help_about")),
            ],
        },
    ]
}

/// Contract plus live selections, last-output availability, and busy state.
pub fn runtime_menus(ui: &super::UiApp) -> Vec<MenuDef> {
    let lang = ui.app.ui_lang;
    let mut defs = menu_contract(lang);

    for def in &mut defs {
        for item in &mut def.items {
            match item.id {
                "view_home" => item.set_selected(ui.app.view == crate::main_ui::AppView::Home),
                "view_studio" => {
                    item.set_selected(ui.app.view == crate::main_ui::AppView::Studio)
                }
                "tab_content" => {
                    item.set_selected(ui.app.center_tab == crate::main_ui::CenterTab::Content)
                }
                "tab_preview" => {
                    item.set_selected(ui.app.center_tab == crate::main_ui::CenterTab::Preview)
                }
                "tab_log" => item.set_selected(ui.app.center_tab == crate::main_ui::CenterTab::Log),
                "theme_system" => {
                    item.set_selected(ui.app.theme == crate::main_ui::AppTheme::System)
                }
                "theme_light" => item.set_selected(ui.app.theme == crate::main_ui::AppTheme::Light),
                "theme_dark" => item.set_selected(ui.app.theme == crate::main_ui::AppTheme::Dark),
                "lang_id" => item.set_selected(lang == UiLang::Id),
                "lang_en" => item.set_selected(lang == UiLang::En),
                "file_open_pdf" => {
                    let has_output = ui.app.last_output.is_some();
                    item.enabled = has_output;
                    item.reason = (!has_output).then(|| tr(lang, "menu_reason_no_output").to_string());
                }
                "pub_batch_fill" if ui.busy => {
                    item.label = format!("● {}", tr(lang, "btn_generate_publish"));
                    item.enabled = false;
                    item.reason = Some(tr(lang, "menu_reason_busy").to_string());
                }
                id if id.starts_with("profile_") => {
                    let profile = id.strip_prefix("profile_").unwrap_or_default();
                    item.set_selected(ui.app.selected_profile == profile);
                }
                _ => {}
            }
        }
    }

    defs
}

/// Localized title with the caret affordance.
pub fn title_with_caret(title: &str) -> String {
    format!("{title} ▾")
}

/// Estimated rendered width of one menu title (includes caret allowance).
pub fn title_width(title: &str) -> f32 {
    let raw = 30.0 + 7.2 * title.chars().count() as f32;
    (raw / 2.0).ceil() * 2.0
}

/// Left edge of every popover, derived from the same widths the bar renders.
pub fn menu_offsets(titles: &[&str], bar_left: f32, gap: f32) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(titles.len());
    let mut x = bar_left;
    for (index, title) in titles.iter().enumerate() {
        if index > 0 {
            x += gap;
        }
        offsets.push(x);
        x += title_width(title);
    }
    offsets
}

/// Keep a popover fully on screen, with an 8 px margin on both edges.
pub fn clamp_popover_left(left: f32, viewport_width: f32, popover_width: f32) -> f32 {
    let max_left = (viewport_width - popover_width - 8.0).max(8.0);
    left.clamp(8.0, max_left)
}

/// Menu titles bar: a container with a bottom rule so the row reads as a menu bar.
pub fn menu_bar(
    defs: &[MenuDef],
    open: Option<usize>,
    focus: Option<usize>,
    dark: bool,
) -> Element<'static, Message> {
    let titles = defs
        .iter()
        .enumerate()
        .map(|(i, def)| menu_title(def.title.as_str(), i, open == Some(i), focus == Some(i)))
        .collect::<Vec<_>>();

    let bar = row(titles)
        .spacing(MENU_GAP)
        .align_y(iced::Alignment::Center);

    let _ = dark;
    container(bar)
        .height(super::chrome::APP_BAR_HEIGHT)
        .padding([0, 2])
        .align_y(iced::alignment::Vertical::Center)
        .into()
}

fn menu_title(title: &str, index: usize, is_open: bool, is_focus: bool) -> Element<'static, Message> {
    let label = title_with_caret(title);
    let width = title_width(title);
    let btn = button(
        container(text(label).size(12))
            .width(width)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .padding([3, 0])
    .on_press(Message::Menu(MenuMsg::Toggled(index)))
    .style(move |theme: &Theme, status| {
        let (bg, text_color) = if is_open {
            (theme.palette().text.scale_alpha(0.16), theme::ACCENT)
        } else if is_focus || matches!(status, button::Status::Hovered) {
            (theme.palette().text.scale_alpha(0.1), theme.palette().text)
        } else {
            (iced::Color::TRANSPARENT, theme.palette().text)
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

    mouse_area(btn)
        .on_enter(Message::Menu(MenuMsg::Hovered(index)))
        .into()
}

/// Floating root-level overlay for the open menu dropdown.
/// The click catcher starts below the bar so titles stay hoverable.
pub fn menu_overlay(
    defs: &[MenuDef],
    index: usize,
    dark: bool,
    viewport: Size,
    highlight: Option<usize>,
) -> Element<'static, Message> {
    let titles: Vec<&str> = defs.iter().map(|def| def.title.as_str()).collect();
    let offsets = menu_offsets(&titles, BAR_LEFT, MENU_GAP);
    let wanted = offsets.get(index).copied().unwrap_or(BAR_LEFT);
    let left = clamp_popover_left(wanted, viewport.width, POPOVER_WIDTH);
    let max_height = (viewport.height - super::chrome::APP_BAR_HEIGHT - 16.0).max(120.0);

    let backdrop = column![
        space().height(super::chrome::APP_BAR_HEIGHT),
        mouse_area(space().width(Length::Fill).height(Length::Fill))
            .on_press(Message::Menu(MenuMsg::Toggled(index))),
    ];

    let items = defs
        .get(index)
        .map(|def| def.items.as_slice())
        .unwrap_or(&[]);
    let card = popover(items, dark, max_height, highlight);

    let dropdown = column![
        space().height(super::chrome::APP_BAR_HEIGHT),
        row![space().width(left), card],
    ];

    stack![backdrop, dropdown]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn popover(
    items: &[MenuItem],
    dark: bool,
    max_height: f32,
    highlight: Option<usize>,
) -> Element<'static, Message> {
    let rows = items
        .iter()
        .enumerate()
        .map(|(index, item)| match &item.kind {
            ItemKind::Separator => rule::horizontal(1)
                .style(move |theme: &Theme| rule::Style {
                    color: if dark {
                        iced::Color::from_rgb8(45, 48, 56)
                    } else {
                        theme.palette().text.scale_alpha(0.15)
                    },
                    radius: iced::border::Radius::default(),
                    fill_mode: iced::widget::rule::FillMode::Full,
                    snap: true,
                })
                .into(),
            ItemKind::Header => container(
                text(item.label.clone())
                    .size(11)
                    .color(iced::Color::from_rgb8(140, 140, 140)),
            )
            .padding([3, 6])
            .into(),
            ItemKind::Action => item_button(item, false, highlight == Some(index)),
            ItemKind::Selected(selected) => {
                item_button(item, *selected, highlight == Some(index))
            }
        })
        .collect::<Vec<_>>();

    dropdown_card(rows, dark, max_height)
}

fn dropdown_card(
    rows: Vec<Element<'static, Message>>,
    dark: bool,
    max_height: f32,
) -> Element<'static, Message> {
    let bg = if dark {
        theme::POPOVER_BG_DARK
    } else {
        theme::POPOVER_BG_LIGHT
    };

    let body = scrollable(column(rows).spacing(1)).width(Length::Fill);

    container(body)
        .max_height(max_height)
        .padding(5)
        .width(POPOVER_WIDTH)
        .style(move |theme: &Theme| iced::widget::container::Style {
            background: Some(bg.into()),
            border: iced::Border {
                radius: (6.0).into(),
                width: 1.0,
                color: if dark {
                    iced::Color::from_rgb8(48, 51, 60)
                } else {
                    theme.palette().text.scale_alpha(0.25)
                },
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.28),
                offset: iced::Vector::new(0.0, 3.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .into()
}

/// Compile split-button dropdown: PDF/X-4 export and the last PDF output.
///
/// Anchored under the compile control. G11 replaces this fixed right margin with
/// the shared anchored-popover helper.
pub fn compile_menu_overlay(
    lang: UiLang,
    has_output: bool,
    busy: bool,
    dark: bool,
) -> Element<'static, Message> {
    let backdrop = column![
        space().height(super::chrome::APP_BAR_HEIGHT),
        mouse_area(space().width(Length::Fill).height(Length::Fill))
            .on_press(Message::CompileMenuToggled),
    ];

    let rows = vec![
        compile_row(
            format!("📄  {}", tr(lang, "export_pdf_x4")),
            tr(lang, "tip_export_x4"),
            !busy,
            Message::Action("pub_batch_x4"),
        ),
        compile_row(
            format!("📑  {}", tr(lang, "open_last_pdf")),
            tr(lang, "tip_open_last_pdf"),
            has_output && !busy,
            Message::Action("file_open_pdf"),
        ),
    ];

    let dropdown = column![
        space().height(super::chrome::APP_BAR_HEIGHT),
        row![space().width(Length::Fill), dropdown_card(rows, dark, 320.0)].padding([0, 250]),
    ];

    stack![backdrop, dropdown]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn compile_row(
    label: String,
    tooltip: &str,
    enabled: bool,
    msg: Message,
) -> Element<'static, Message> {
    let row = button(text(label).size(12).width(Length::Fill))
        .padding([5, 8])
        .width(Length::Fill)
        .on_press_maybe(enabled.then_some(msg))
        .style(move |theme: &Theme, status| {
            let bg = if !enabled {
                iced::Color::TRANSPARENT
            } else {
                match status {
                    button::Status::Hovered => theme.palette().text.scale_alpha(0.1),
                    button::Status::Pressed => theme.palette().text.scale_alpha(0.2),
                    _ => iced::Color::TRANSPARENT,
                }
            };
            button::Style {
                background: Some(bg.into()),
                text_color: if enabled {
                    theme.palette().text
                } else {
                    theme.palette().text.scale_alpha(0.4)
                },
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });
    super::widgets::tooltip(row, tooltip)
}

fn item_button(
    item: &MenuItem,
    selected: bool,
    highlighted: bool,
) -> Element<'static, Message> {
    let label = if selected {
        format!("✓ {}", item.label)
    } else {
        item.label.clone()
    };
    let enabled = item.enabled;
    let row = button(text(label).size(12).width(Length::Fill))
        .padding([5, 8])
        .width(Length::Fill)
        .on_press_maybe(enabled.then_some(Message::Menu(MenuMsg::Chosen(item.id))))
        .style(move |theme: &Theme, status| {
            let bg = if highlighted {
                theme.palette().text.scale_alpha(0.16)
            } else if !enabled {
                iced::Color::TRANSPARENT
            } else {
                match status {
                    button::Status::Hovered => theme.palette().text.scale_alpha(0.1),
                    button::Status::Pressed => theme.palette().text.scale_alpha(0.2),
                    _ => iced::Color::TRANSPARENT,
                }
            };
            button::Style {
                background: Some(bg.into()),
                text_color: if enabled {
                    theme.palette().text
                } else {
                    theme.palette().text.scale_alpha(0.4)
                },
                border: iced::Border {
                    radius: (4.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    match &item.reason {
        Some(reason) => super::widgets::tooltip(row, reason),
        None => row.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn ids(def: &MenuDef) -> Vec<&'static str> {
        def.items
            .iter()
            .filter(|item| !item.id.is_empty())
            .map(|item| item.id)
            .collect()
    }

    #[test]
    fn every_title_has_caret() {
        for lang in UiLang::ALL {
            let defs = menu_contract(lang);
            assert_eq!(defs.len(), 8);
            for def in &defs {
                let rendered = title_with_caret(&def.title);
                assert!(
                    rendered.ends_with(" ▾"),
                    "title should end with a caret: {rendered}"
                );
                assert_eq!(rendered, format!("{} ▾", def.title));
            }
        }
    }

    #[test]
    fn menu_contract_matches_target_order_and_ids() {
        let expected: [(&str, &[&str]); 8] = [
            (
                "menu_file",
                &[
                    "file_new",
                    "file_open",
                    "file_save",
                    "file_load_template",
                    "file_open_pdf",
                    "app_exit",
                ],
            ),
            (
                "menu_publication",
                &[
                    "pub_generate",
                    "pub_batch_fill",
                    "profile_bps_brs",
                    "profile_bps_dda",
                    "profile_generic_report",
                    "profile_book",
                    "profile_infographic",
                    "profile_magazine",
                    "profile_newsletter",
                    "profile_slide",
                    "tool_vector_chart",
                    "tool_toc",
                    "tool_cover_svg",
                    "header_footer_info",
                    "pub_qa_audit",
                ],
            ),
            (
                "menu_publication_tools",
                &[
                    "file_import_sources",
                    "file_import_excel",
                    "open_document_settings",
                    "pub_translate",
                    "tool_glossary_import",
                    "tool_glossary_export",
                ],
            ),
            (
                "menu_automation_content",
                &[
                    "auto_cover_studio",
                    "auto_vector_chart",
                    "auto_toc",
                    "tool_ai_translate",
                    "tool_glossary",
                    "tool_diff",
                    "auto_translate",
                ],
            ),
            (
                "menu_designcraft",
                &[
                    "dc_launch",
                    "dc_kill",
                    "dc_open_doc",
                    "dc_reconnect",
                    "dc_auto_extract",
                ],
            ),
            (
                "menu_view",
                &[
                    "view_home",
                    "view_studio",
                    "tab_content",
                    "tab_preview",
                    "tab_log",
                    "theme_system",
                    "theme_light",
                    "theme_dark",
                    "lang_id",
                    "lang_en",
                ],
            ),
            (
                "menu_shortcut",
                &[
                    "pub_batch_x4",
                    "file_open_pdf",
                    "file_save",
                    "file_open",
                    "file_new",
                ],
            ),
            (
                "menu_help",
                &["help_bps", "help_ipc", "help_about"],
            ),
        ];

        for lang in UiLang::ALL {
            let defs = menu_contract(lang);
            for (index, (title_key, rows)) in expected.iter().enumerate() {
                assert_eq!(
                    defs[index].title,
                    tr(lang, title_key),
                    "menu {index} title for {lang}"
                );
                assert_eq!(&ids(&defs[index]), rows, "menu {index} rows for {lang}");
            }
        }
    }

    #[test]
    fn shortcut_repeats_are_allowlisted_and_others_unique() {
        const ALLOWED: [&str; 5] = [
            "pub_batch_x4",
            "file_open_pdf",
            "file_save",
            "file_open",
            "file_new",
        ];

        let defs = menu_contract(UiLang::Id);
        let mut seen = HashSet::new();
        for (index, def) in defs.iter().enumerate() {
            for item in &def.items {
                if item.id.is_empty() {
                    continue;
                }
                if index == 6 {
                    assert!(
                        ALLOWED.contains(&item.id),
                        "Shortcut repeats a non-allowlisted id: {}",
                        item.id
                    );
                    continue;
                }
                assert!(seen.insert(item.id), "duplicate menu id: {}", item.id);
            }
        }

        // Every allowlisted shortcut id must be an actual repeat of a base menu,
        // except `pub_batch_x4`, which intentionally lives only in Shortcut.
        let base: HashSet<&str> = defs
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 6)
            .flat_map(|(_, def)| ids(def))
            .collect();
        for id in ALLOWED {
            if id == "pub_batch_x4" {
                continue;
            }
            assert!(base.contains(id), "Shortcut id has no base route: {id}");
        }
    }

    #[test]
    fn disabled_rows_carry_a_reason() {
        let defs = menu_contract(UiLang::Id);
        for def in &defs {
            for item in &def.items {
                if item.is_actionable() && !item.enabled {
                    assert!(
                        item.reason.as_deref().is_some_and(|r| !r.is_empty()),
                        "disabled row {} must explain why",
                        item.id
                    );
                }
            }
        }
        let header_footer = defs[1]
            .items
            .iter()
            .find(|item| item.id == "header_footer_info")
            .expect("header_footer_info row");
        assert!(!header_footer.enabled);
    }

    #[test]
    fn offsets_match_declared_widths() {
        for lang in UiLang::ALL {
            let defs = menu_contract(lang);
            let titles: Vec<&str> = defs.iter().map(|def| def.title.as_str()).collect();
            let offsets = menu_offsets(&titles, BAR_LEFT, MENU_GAP);
            assert_eq!(offsets.len(), 8);

            let mut expected = BAR_LEFT;
            for (index, title) in titles.iter().enumerate() {
                assert_eq!(
                    offsets[index], expected,
                    "offset {index} drifted for {lang}"
                );
                expected += title_width(title);
                if index + 1 < titles.len() {
                    expected += MENU_GAP;
                }
            }

            for viewport in [1024.0_f32, 1280.0, 1440.0] {
                for left in &offsets {
                    let clamped = clamp_popover_left(*left, viewport, POPOVER_WIDTH);
                    assert!(clamped >= 8.0, "popover clipped on the left at {viewport}");
                    assert!(
                        clamped + POPOVER_WIDTH <= viewport - 8.0,
                        "popover clipped on the right at {viewport}"
                    );
                }
            }
        }
    }
}
