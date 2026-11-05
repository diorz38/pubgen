// Iced UI (migration target). See iced_migration_plan.md.
// Phase 2 skeleton: PosApp owns main_ui::App + UiState.

mod canvas_cache;
mod chrome;
pub mod dialogs;
mod home;
mod menu;
mod message;
mod metadata;
pub mod panels;
mod preview;
mod theme;
mod update;
mod view;
mod widgets;

use iced::{window, Size, Subscription, Task};

pub use message::{ChromeMsg, DialogKind, MenuKey, Message};
#[cfg(test)]
pub use message::MenuMsg;

/// Left-panel collapsible section ids, in render order.
pub const PANEL_IDS: [&str; 1] = [
    "metadata_form",
];

/// Section reopened by "Expand basics" (Document Settings).
pub const BASIC_PANEL: &str = "metadata_form";

/// Actions that require a live DesignCraft engine.
pub const ENGINE_ACTIONS: [&str; 4] = [
    "pub_discover",
    "pub_fill_live",
    "dc_kill",
    "dc_open_doc",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum CardCoverTab {
    #[default]
    EditCover,
    CoverDetail,
}

pub struct UiApp {
    pub app: crate::main_ui::App,
    pub card_cover_tab: CardCoverTab,
    pub open_menu: Option<usize>,
    /// Compile split-button dropdown (`PDF/X-4`, open last PDF).
    pub compile_menu_open: bool,
    pub open_panels: std::collections::BTreeSet<&'static str>,
    pub touched_panels: std::collections::BTreeSet<&'static str>,
    /// Keyboard focus on the menu bar (`F10`), independent of an open popover.
    pub menu_focus: Option<usize>,
    /// Highlighted row inside the open popover.
    pub menu_row: Option<usize>,
    /// Last known window size, used to keep popovers on screen.
    pub window_size: Size,
    pub open_slots: std::collections::BTreeSet<usize>,
    pub maximized: bool,
    pub system_dark: bool,
    pub busy: bool,
    pub busy_action: Option<String>,
    pub busy_tick: u32,
    pub profile_picker: iced::widget::combo_box::State<crate::profiles::PubgenProfile>,
    pub add_page_kind: crate::metadata_form::PageKind,
    /// Index of the publication card currently open in the editor popup.
    pub editing_page: Option<usize>,
    /// Inline "Hapus kartu ini?" confirmation armed inside the card modal.
    pub confirm_delete: bool,
    /// Active pop window / dialog.
    pub active_dialog: Option<DialogKind>,
    /// Persistent form options state for dialogs.
    pub dialog_state: dialogs::DialogState,
    /// A/B Test Telemetry Tracker & Variant Assignor.
    pub telemetry: crate::telemetry::TelemetryTracker,
}

impl UiApp {
    fn new() -> (Self, Task<Message>) {
        let app = crate::main_ui::App::new();
        (
            Self {
                app,
                card_cover_tab: CardCoverTab::default(),
                open_menu: None,
                compile_menu_open: false,
                open_panels: std::collections::BTreeSet::new(),
                touched_panels: std::collections::BTreeSet::new(),
                menu_focus: None,
                menu_row: None,
                window_size: Size::new(
                    crate::main_ui::DEFAULT_WINDOW_SIZE[0],
                    crate::main_ui::DEFAULT_WINDOW_SIZE[1],
                ),
                open_slots: std::collections::BTreeSet::new(),
                maximized: false,
                system_dark: true,
                busy: false,
                busy_action: None,
                busy_tick: 0,
                profile_picker: iced::widget::combo_box::State::new(
                    crate::profiles::ProfileManager::builtin_profiles(),
                ),
                add_page_kind: crate::metadata_form::PageKind::Paragraph,
                editing_page: None,
                confirm_delete: false,
                active_dialog: None,
                dialog_state: dialogs::DialogState::default(),
                telemetry: crate::telemetry::TelemetryTracker::init(),
            },
            Task::batch([
                iced::system::theme().map(|mode| Message::SystemTheme {
                    dark: !matches!(mode, iced::theme::Mode::Light),
                }),
                window::latest()
                    .and_then(|id| window::is_maximized(id))
                    .map(|max| Message::Chrome(ChromeMsg::Maximized(max))),
            ]),
        )
    }

    /// Close any open popover and drop menu-bar focus/highlight.
    pub fn close_menu(&mut self) {
        self.open_menu = None;
        self.menu_focus = None;
        self.menu_row = None;
    }
}

pub fn run() -> anyhow::Result<()> {
    let res = iced::application(UiApp::new, update::update, view::view)
        .title(crate::main_ui::APP_TITLE)
        .theme(|ui: &UiApp| theme::to_iced(ui.app.theme, ui.system_dark))
        .subscription(subscription)
        .window(window::Settings {
            size: Size::new(
                crate::main_ui::DEFAULT_WINDOW_SIZE[0],
                crate::main_ui::DEFAULT_WINDOW_SIZE[1],
            ),
            min_size: Some(Size::new(800.0, 500.0)),
            resizable: true,
            decorations: false,
            icon: crate::branding::iced_icon(),
            ..Default::default()
        })
        .run();
    res.map_err(|e| anyhow::anyhow!("iced: {e:?}"))
}

fn subscription(ui: &UiApp) -> Subscription<Message> {
    // Poll the DesignCraft control channel quickly only while we own a process
    // or hold a live client. An idle editor can tick once a second, which keeps
    // the connection probe working without forcing 2.5 full repaints per second.
    let tick_period = if ui.busy {
        std::time::Duration::from_millis(100)
    } else if ui.app.dc.is_some() || ui.app.client.is_some() {
        std::time::Duration::from_millis(400)
    } else {
        std::time::Duration::from_secs(1)
    };

    Subscription::batch([
        iced::time::every(tick_period).map(|_| Message::Tick),
        iced::system::theme_changes().map(|mode| Message::SystemTheme {
            dark: !matches!(mode, iced::theme::Mode::Light),
        }),
        iced::event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(window::Event::Resized(size)) => {
                Some(Message::WindowResized(size))
            }
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key,
                modifiers,
                repeat,
                ..
            }) if !repeat => shortcut_for(&key, modifiers)
                .or_else(|| menu_key(&key, modifiers).map(Message::MenuKey)),
            _ => None,
        }),
    ])
}

/// Pure key → message mapping, kept separate so it can be unit tested.
///
/// Only `Esc` and `Ctrl`/`Cmd` combinations are global; plain characters and
/// shift/alt combinations return `None` so focused text inputs keep them.
pub fn shortcut_for(
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Option<Message> {
    use iced::keyboard::{key::Named, Key};

    if modifiers.command() {
        return match key.as_ref() {
            Key::Character(c) if c.eq_ignore_ascii_case("s") => {
                Some(Message::Action("file_save"))
            }
            Key::Named(Named::Enter) => Some(Message::CompilePressed),
            _ => None,
        };
    }

    if modifiers.shift() || modifiers.alt() || modifiers.logo() {
        return None;
    }

    match key.as_ref() {
        Key::Named(Named::Escape) => Some(Message::EscapePressed),
        _ => None,
    }
}

/// Menu-bar key mapping: `F10` plus arrows/`Enter`/`Space`/`Esc`.
///
/// Always maps, regardless of focus; the update layer ignores these unless the
/// bar or a popover is active, so text inputs keep their normal edits.
pub fn menu_key(
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Option<MenuKey> {
    use iced::keyboard::{key::Named, Key};

    if modifiers.command() || modifiers.alt() || modifiers.logo() || modifiers.shift() {
        return None;
    }

    match key.as_ref() {
        Key::Named(Named::F10) => Some(MenuKey::FocusBar),
        Key::Named(Named::Escape) => Some(MenuKey::Escape),
        Key::Named(Named::ArrowLeft) => Some(MenuKey::Left),
        Key::Named(Named::ArrowRight) => Some(MenuKey::Right),
        Key::Named(Named::ArrowDown) => Some(MenuKey::Down),
        Key::Named(Named::ArrowUp) => Some(MenuKey::Up),
        Key::Named(Named::Enter) | Key::Named(Named::Space) => Some(MenuKey::Activate),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> UiApp {
        let (ui, _task) = UiApp::new();
        ui
    }

    fn apply(ui: &mut UiApp, message: Message) {
        let _ = update::update(ui, message);
    }

    #[test]
    fn fresh_state_has_no_open_panels() {
        let ui = fresh();
        assert!(ui.open_panels.is_empty(), "open_panels should start empty");
        assert!(
            ui.touched_panels.is_empty(),
            "touched_panels should start empty"
        );
    }

    #[test]
    fn collapse_all_clears_and_marks_touched() {
        let mut ui = fresh();
        apply(&mut ui, Message::PanelToggled(BASIC_PANEL));
        assert_eq!(ui.open_panels.len(), 1);

        apply(&mut ui, Message::PanelsCollapseAll);

        assert!(ui.open_panels.is_empty());
        for id in PANEL_IDS {
            assert!(
                ui.touched_panels.contains(id),
                "collapse all should touch {id}"
            );
        }
        assert_eq!(ui.touched_panels.len(), PANEL_IDS.len());
    }

    #[test]
    fn expand_basics_restores_only_document_settings() {
        let mut ui = fresh();
        apply(&mut ui, Message::PanelsCollapseAll);
        assert!(ui.open_panels.is_empty());

        apply(&mut ui, Message::PanelsExpandBasics);

        assert_eq!(ui.open_panels.len(), 1);
        assert!(ui.open_panels.contains(BASIC_PANEL));
        for id in PANEL_IDS {
            assert!(
                ui.touched_panels.contains(id),
                "expand basics should touch {id}"
            );
        }
    }

    #[test]
    fn open_set_survives_home_round_trip() {
        let mut ui = fresh();
        apply(&mut ui, Message::PanelToggled(BASIC_PANEL));
        apply(&mut ui, Message::PanelToggled("glossary_panel"));
        let before = ui.open_panels.clone();

        apply(&mut ui, Message::GoHome);
        assert_eq!(ui.app.view, crate::main_ui::AppView::Home);
        assert_eq!(ui.open_panels, before, "Home must not clear panels");

        apply(&mut ui, Message::HomeToggled);
        assert_eq!(ui.app.view, crate::main_ui::AppView::Studio);
        assert_eq!(ui.open_panels, before, "returning must keep panels open");
    }

    // ---- G7 keyboard shortcuts ----

    fn key(k: &str) -> iced::keyboard::Key {
        iced::keyboard::Key::Character(k.into())
    }

    fn esc() -> iced::keyboard::Key {
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)
    }

    fn enter() -> iced::keyboard::Key {
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
    }

    #[test]
    fn esc_maps_to_escape_message() {
        assert!(matches!(
            shortcut_for(&esc(), iced::keyboard::Modifiers::empty()),
            Some(Message::EscapePressed)
        ));
    }

    #[test]
    fn ctrl_s_maps_to_file_save() {
        assert!(matches!(
            shortcut_for(&key("s"), iced::keyboard::Modifiers::CTRL),
            Some(Message::Action("file_save"))
        ));
        assert!(matches!(
            shortcut_for(&key("S"), iced::keyboard::Modifiers::CTRL),
            Some(Message::Action("file_save"))
        ));
    }

    #[test]
    fn ctrl_enter_maps_to_compile() {
        assert!(matches!(
            shortcut_for(&enter(), iced::keyboard::Modifiers::CTRL),
            Some(Message::CompilePressed)
        ));
    }

    #[test]
    fn plain_keys_are_ignored() {
        let none = iced::keyboard::Modifiers::empty();
        assert!(shortcut_for(&key("s"), none).is_none());
        assert!(shortcut_for(&key("Enter"), none).is_none());
        assert!(shortcut_for(&key("a"), iced::keyboard::Modifiers::SHIFT).is_none());
    }

    #[test]
    fn escape_with_no_modal_is_a_noop() {
        let mut ui = fresh();
        ui.open_menu = None;
        ui.compile_menu_open = false;
        let view = ui.app.view;
        let panels = ui.open_panels.clone();

        apply(&mut ui, Message::EscapePressed);

        assert_eq!(ui.app.view, view);
        assert_eq!(ui.open_panels, panels);
        assert!(ui.editing_page.is_none());
        assert!(ui.open_menu.is_none());
        assert!(!ui.compile_menu_open);
    }

    #[test]
    fn escape_closes_topmost_surface_only() {
        let mut ui = fresh();
        ui.editing_page = Some(0);
        ui.compile_menu_open = true;
        ui.open_menu = Some(1);

        apply(&mut ui, Message::EscapePressed);
        assert!(ui.editing_page.is_none(), "modal closes first");
        assert!(ui.compile_menu_open, "compile popover stays behind the modal");
        assert_eq!(ui.open_menu, Some(1));

        apply(&mut ui, Message::EscapePressed);
        assert!(!ui.compile_menu_open, "then the compile popover");
        assert_eq!(ui.open_menu, Some(1));

        apply(&mut ui, Message::EscapePressed);
        assert_eq!(ui.open_menu, None, "then the menu bar");
    }

    // ---- G11 menu keyboard ----

    fn named(k: iced::keyboard::key::Named) -> iced::keyboard::Key {
        iced::keyboard::Key::Named(k)
    }

    #[test]
    fn menu_key_maps_bar_navigation() {
        use iced::keyboard::key::Named;
        let none = iced::keyboard::Modifiers::empty();

        assert_eq!(menu_key(&named(Named::F10), none), Some(MenuKey::FocusBar));
        assert_eq!(
            menu_key(&named(Named::ArrowLeft), none),
            Some(MenuKey::Left)
        );
        assert_eq!(
            menu_key(&named(Named::ArrowRight), none),
            Some(MenuKey::Right)
        );
        assert_eq!(
            menu_key(&named(Named::ArrowDown), none),
            Some(MenuKey::Down)
        );
        assert_eq!(menu_key(&named(Named::ArrowUp), none), Some(MenuKey::Up));
        assert_eq!(
            menu_key(&named(Named::Enter), none),
            Some(MenuKey::Activate)
        );
        assert_eq!(
            menu_key(&named(Named::Space), none),
            Some(MenuKey::Activate)
        );
        assert_eq!(
            menu_key(&named(Named::Escape), none),
            Some(MenuKey::Escape)
        );
        // Esc with no menu is still the mapper's Escape, and the update layer
        // treats it as a no-op (covered by `escape_with_no_modal_is_a_noop`).
        assert_eq!(
            menu_key(&named(Named::Escape), none),
            Some(MenuKey::Escape)
        );
        assert!(menu_key(&key("a"), none).is_none());
        assert!(menu_key(&named(Named::ArrowDown), iced::keyboard::Modifiers::SHIFT).is_none());
    }

    #[test]
    fn f10_focuses_bar_and_arrows_cycle_titles() {
        let mut ui = fresh();

        apply(&mut ui, Message::MenuKey(MenuKey::FocusBar));
        assert_eq!(ui.menu_focus, Some(0));
        assert!(ui.open_menu.is_none());

        apply(&mut ui, Message::MenuKey(MenuKey::Right));
        assert_eq!(ui.menu_focus, Some(1));
        apply(&mut ui, Message::MenuKey(MenuKey::Left));
        assert_eq!(ui.menu_focus, Some(0));
        apply(&mut ui, Message::MenuKey(MenuKey::Left));
        assert_eq!(ui.menu_focus, Some(7), "Left wraps to the last menu");

        apply(&mut ui, Message::MenuKey(MenuKey::FocusBar));
        assert!(ui.menu_focus.is_none(), "F10 again leaves the bar");
    }

    #[test]
    fn down_opens_menu_and_up_down_move_row() {
        let mut ui = fresh();
        apply(&mut ui, Message::MenuKey(MenuKey::FocusBar));

        apply(&mut ui, Message::MenuKey(MenuKey::Down));
        assert_eq!(ui.open_menu, Some(0));
        assert_eq!(ui.menu_row, Some(0), "first enabled row is highlighted");

        apply(&mut ui, Message::MenuKey(MenuKey::Down));
        assert_eq!(ui.menu_row, Some(1));
        apply(&mut ui, Message::MenuKey(MenuKey::Up));
        assert_eq!(ui.menu_row, Some(0));

        // Left/Right switch menus while a popover is open.
        apply(&mut ui, Message::MenuKey(MenuKey::Right));
        assert_eq!(ui.open_menu, Some(1));
        assert_eq!(ui.menu_focus, Some(1));
    }

    #[test]
    fn enter_activates_highlighted_row_and_closes() {
        let mut ui = fresh();
        apply(&mut ui, Message::MenuKey(MenuKey::FocusBar));
        apply(&mut ui, Message::MenuKey(MenuKey::Down));
        assert_eq!(ui.open_menu, Some(0));

        apply(&mut ui, Message::MenuKey(MenuKey::Activate));

        assert!(ui.open_menu.is_none(), "activation closes the popover");
        assert!(ui.menu_focus.is_none());
        assert!(ui.menu_row.is_none());
        assert!(
            !ui.app
                .log
                .iter()
                .any(|line| line.starts_with("unhandled menu action")),
            "the activated row must be routed: {:?}",
            ui.app.log
        );
    }

    #[test]
    fn esc_closes_menu_and_navigation_is_ignored_when_inactive() {
        let mut ui = fresh();
        apply(&mut ui, Message::MenuKey(MenuKey::Escape));
        assert!(ui.menu_focus.is_none());

        apply(&mut ui, Message::MenuKey(MenuKey::Down));
        assert!(ui.open_menu.is_none(), "Down is ignored without focus");

        apply(&mut ui, Message::MenuKey(MenuKey::FocusBar));
        apply(&mut ui, Message::MenuKey(MenuKey::Down));
        assert!(ui.open_menu.is_some());
        apply(&mut ui, Message::MenuKey(MenuKey::Escape));
        assert!(ui.open_menu.is_none());
        assert!(ui.menu_focus.is_none());
    }

    #[test]
    fn tab_hover_switches_menu_only_while_open() {
        let mut ui = fresh();

        apply(&mut ui, Message::Menu(MenuMsg::Hovered(2)));
        assert!(ui.open_menu.is_none(), "hover is a no-op with no popover");

        apply(&mut ui, Message::Menu(MenuMsg::Toggled(0)));
        assert_eq!(ui.open_menu, Some(0));
        apply(&mut ui, Message::Menu(MenuMsg::Hovered(3)));
        assert_eq!(ui.open_menu, Some(3));
        assert_eq!(ui.menu_focus, Some(3));
    }
}
