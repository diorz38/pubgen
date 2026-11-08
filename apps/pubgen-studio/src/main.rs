//! PubGen Studio: Dual-Mode Publication Orchestrator & Layout Studio.
//!
//! Usage: `pubgen-studio [project.pubgen | document.designcraft]`
//!
//! - Opening a `*.pubgen` file boots into **Orchestrator Mode**.
//! - Opening a `*.designcraft` file boots into **DesignCraft Mode**.
//! - Launching without arguments defaults to **Orchestrator Mode**.
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
#![allow(dead_code, unused_variables, clippy::disallowed_types, clippy::collapsible_if)]

mod app;
mod domain;
mod mode;
mod project;
mod settings;
mod window_frame;
mod workspaces;

use app::PubgenStudioApp;
use designcraft_engine::Session;
use designcraft_ui_egui::{DesignApp, Services};
use mode::StudioMode;
use project::PubgenContainer;
use std::ffi::OsStr;
use std::path::PathBuf;

fn services() -> Services {
    Services {
        read: Some(Box::new(|p: &str| std::fs::read(p).map_err(|e| e.to_string()))),
        write: Some(Box::new(|p: &str, b: &[u8]| designcraft_engine::atomic::write_atomic(std::path::Path::new(p), b).map_err(|e| e.to_string()))),
        clipboard_text: Some(Box::new(|| arboard::Clipboard::new().and_then(|mut c| c.get_text()).ok())),
        ..Default::default()
    }
}

fn app_icon() -> Option<egui::IconData> {
    #[cfg(target_os = "macos")]
    let png: &[u8] = include_bytes!("../../../assets/app-icon/designcraft-macos-512.png");
    #[cfg(not(target_os = "macos"))]
    let png: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/256x256/apps/ai.storyteller.designcraft.png");
    eframe::icon_data::from_png_bytes(png).ok()
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let initial_file = args.first().map(PathBuf::from);

    let (initial_mode, project_path, container) = match &initial_file {
        Some(path) if path.extension() == Some(OsStr::new("pubgen")) => {
            let container = PubgenContainer::open(path).ok();
            (StudioMode::Orchestrator, Some(path.clone()), container)
        }
        Some(path) if path.extension() == Some(OsStr::new("designcraft")) => (StudioMode::DesignCraft, Some(path.clone()), None),
        Some(path) => (StudioMode::Orchestrator, Some(path.clone()), None),
        None => (StudioMode::Orchestrator, None, None),
    };

    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PubGen Studio")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([900.0, 560.0])
            .with_decorations(!cfg!(target_os = "windows"))
            .with_drag_and_drop(true)
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false),
        system_font_fallback: false,
        ..Default::default()
    };
    if let Some(icon) = app_icon() {
        options.viewport = options.viewport.with_icon(icon);
    }

    let file_to_open = initial_file.clone();
    let has_cached_layout = container.as_ref().and_then(|c| c.cached_layout.as_ref()).is_some();

    eframe::run_native(
        "PubGen Studio",
        options,
        Box::new(move |cc| {
            let session = Session::new();
            let mut app = DesignApp::new(session, services());
            app.integrated_titlebar = cfg!(target_os = "macos");
            app.custom_titlebar = cfg!(target_os = "windows");
            app.services.window_frame = window_frame::service(cc, app.custom_titlebar);

            // Sync initial mode with pubgen_bar switcher
            let is_orch = initial_mode == StudioMode::Orchestrator;
            designcraft_ui_egui::menus::pubgen_bar::set_orchestrator_mode(&cc.egui_ctx, is_orch);

            // If a .designcraft file was passed, open it immediately into the engine session
            if let Some(path) = &file_to_open {
                if path.extension() == Some(OsStr::new("designcraft")) {
                    let _ = app.open_file(
                        "file.open",
                        serde_json::json!({
                            "path": path.to_string_lossy().to_string()
                        }),
                    );
                } else if path.extension() == Some(OsStr::new("pubgen")) && has_cached_layout {
                    let sidecar = path.with_extension("designcraft");
                    if sidecar.is_file() {
                        let _ = app.open_file(
                            "file.open",
                            serde_json::json!({
                                "path": sidecar.to_string_lossy().to_string()
                            }),
                        );
                    }
                }
            }

            Ok(Box::new(PubgenStudioApp::new(app, initial_mode, project_path, container, settings::ProviderSettings::load())))
        }),
    )
}
