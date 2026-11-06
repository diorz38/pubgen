use crate::mode::StudioMode;
use crate::project::PubgenContainer;
use crate::workspaces::orchestrator::{GuardDialogAction, RecompileGuardDialog, OrchestratorProject};
use designcraft_ui_egui::DesignApp;
use egui::Context;
use std::path::PathBuf;

pub struct PubgenStudioApp {
    pub mode: StudioMode,
    pub app: DesignApp,
    pub project_path: Option<PathBuf>,
    pub container: Option<PubgenContainer>,
    pub guard_dialog: RecompileGuardDialog,
    pub orchestrator_project: OrchestratorProject,
}

impl PubgenStudioApp {
    pub fn new(app: DesignApp, initial_mode: StudioMode, project_path: Option<PathBuf>, container: Option<PubgenContainer>) -> Self {
        let mut orchestrator_project = OrchestratorProject::default();
        if let Some(path) = project_path.as_ref() {
            if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("pubgen")) {
                let _ = orchestrator_project.open(path);
            }
        }
        Self { mode: initial_mode, app, project_path, container, guard_dialog: RecompileGuardDialog::new(), orchestrator_project }
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

        // Render DesignCraft's complete shell:
        // - Undecorated borderless custom topbar with Home icon, Mode Switcher, Menu Bar, Search, Workspaces, and Caption buttons
        // - Tool palettes, status bar, docking system with Pubgen and standard panels, and document canvas
        self.app.ui(ui);
        if self.mode == StudioMode::Orchestrator {
            self.orchestrator_project.sync_ui(ui.ctx());
            crate::workspaces::orchestrator::render_workspace_controls(ui, &mut self.orchestrator_project);
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
