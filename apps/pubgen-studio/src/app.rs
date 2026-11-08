use crate::mode::StudioMode;
use crate::project::PubgenContainer;
use crate::workspaces::orchestrator::{GuardDialogAction, OrchestratorTabs, RecompileGuardDialog};
use designcraft_ui_egui::DesignApp;
use egui::Context;
use std::path::PathBuf;

pub struct PubgenStudioApp {
    pub mode: StudioMode,
    pub app: DesignApp,
    pub project_path: Option<PathBuf>,
    pub container: Option<PubgenContainer>,
    /// AI translation provider config, loaded from settings.json on start;
    /// saved when the setup dialog is accepted (Task 6).
    pub settings: crate::settings::ProviderSettings,
    pub guard_dialog: RecompileGuardDialog,
    pub orchestrator_tabs: OrchestratorTabs,
    pending_open: Option<PathBuf>,
}

impl PubgenStudioApp {
    pub fn new(
        app: DesignApp,
        initial_mode: StudioMode,
        project_path: Option<PathBuf>,
        container: Option<PubgenContainer>,
        settings: crate::settings::ProviderSettings,
    ) -> Self {
        let pending_open =
            project_path.as_ref().filter(|path| path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("pubgen"))).cloned();
        Self {
            mode: initial_mode,
            app,
            project_path,
            container,
            settings,
            guard_dialog: RecompileGuardDialog::new(),
            orchestrator_tabs: OrchestratorTabs::default(),
            pending_open,
        }
    }
}

impl eframe::App for PubgenStudioApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Run background ticks and logic for DesignApp
        self.app.logic(ctx);
    }

    fn raw_input_hook(&mut self, ctx: &Context, raw: &mut egui::RawInput) {
        self.app.raw_input_hook(ctx, raw);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Sync local StudioMode with app_bar switcher state
        let is_orch = designcraft_ui_egui::menus::pubgen_bar::is_orchestrator_mode(ui.ctx());
        self.mode = if is_orch { StudioMode::Orchestrator } else { StudioMode::DesignCraft };

        if self.mode == StudioMode::Orchestrator {
            if let Some(path) = self.pending_open.take() {
                let _ = self.orchestrator_tabs.open(ui.ctx(), &path);
            } else {
                self.orchestrator_tabs.sync_from_ui(ui.ctx());
            }
            designcraft_ui_egui::dock::pubgen::workspace::set_host_tabs(
                ui.ctx(),
                self.orchestrator_tabs.host_titles(),
                self.orchestrator_tabs.active,
            );
        }

        // Render DesignCraft's complete shell:
        // - Undecorated borderless custom topbar with Home icon, Mode Switcher, Menu Bar, Search, Workspaces, and Caption buttons
        // - Tool palettes, status bar, docking system with Pubgen and standard panels, and document canvas
        self.app.ui(ui);
        if self.mode == StudioMode::Orchestrator {
            let ctx = ui.ctx().clone();
            // A File-menu open or Save As changes the visible workspace during the frame.
            self.orchestrator_tabs.sync_from_ui(&ctx);
            if let Some(action) = designcraft_ui_egui::dock::pubgen::workspace::take_host_tab_action(&ctx) {
                use designcraft_ui_egui::dock::pubgen::workspace::HostTabAction;
                match action {
                    HostTabAction::Activate(index) => self.orchestrator_tabs.activate(&ctx, index),
                    HostTabAction::Close(index) => self.orchestrator_tabs.request_close(&ctx, index),
                    HostTabAction::New => self.orchestrator_tabs.new_tab(&ctx),
                    HostTabAction::Open => {
                        if let Some(path) = rfd::FileDialog::new().add_filter("Pubgen Project", &["pubgen"]).pick_file() {
                            let _ = self.orchestrator_tabs.open(&ctx, &path);
                        }
                    }
                    HostTabAction::OpenPath(path) => {
                        let _ = self.orchestrator_tabs.open(&ctx, &path);
                    }
                    HostTabAction::Import => {
                        if let Some(path) = rfd::FileDialog::new().add_filter("Excel Workbook", &["xlsx", "xlsm"]).pick_file() {
                            let _ = self.orchestrator_tabs.import(&ctx, &path);
                        }
                    }
                    HostTabAction::ImportPath(path) => {
                        let _ = self.orchestrator_tabs.import(&ctx, &path);
                    }
                    HostTabAction::Save => {
                        let _ = self.app.run("pubgen.saveProject", serde_json::Value::Null);
                    }
                    HostTabAction::ExportXlsx => {
                        if let Some(path) = rfd::FileDialog::new().add_filter("Excel Workbook", &["xlsx"]).save_file()
                            && let Some(project) = self.orchestrator_tabs.active_project_mut()
                        {
                            let _ = project.export_xlsx(&path);
                        }
                    }
                }
            }
            crate::workspaces::orchestrator::render_close_dialog(ui, &mut self.orchestrator_tabs);
            self.orchestrator_tabs.publish_snapshot(&ctx);
        }

        // Handle Guard Dialog actions when triggered
        match self.guard_dialog.show(ui.ctx()) {
            GuardDialogAction::Overwrite => {
                let _ = self.app.run("pubgen.compileProject", serde_json::json!({"overwrite": true}));
                designcraft_ui_egui::menus::pubgen_bar::set_orchestrator_mode(ui.ctx(), false);
                self.mode = StudioMode::DesignCraft;
            }
            GuardDialogAction::SaveNewVersion => {
                let _ = self.app.run("pubgen.compileProject", serde_json::json!({"new_version": true}));
                designcraft_ui_egui::menus::pubgen_bar::set_orchestrator_mode(ui.ctx(), false);
                self.mode = StudioMode::DesignCraft;
            }
            GuardDialogAction::Cancel | GuardDialogAction::None => {}
        }

        self.app.show_file_dialogs(Some(&*frame));
    }
}
