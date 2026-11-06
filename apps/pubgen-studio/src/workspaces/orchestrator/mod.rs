pub mod guard_dialog;
pub mod project;

use crate::mode::StudioMode;
use designcraft_ui_egui::DesignApp;
use egui::{Color32, RichText, Ui};
pub use guard_dialog::{GuardDialogAction, RecompileGuardDialog};
pub use project::OrchestratorProject;

pub fn render_workspace_controls(ui: &mut Ui, project: &mut OrchestratorProject) {
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        if ui.button("Open .pubgen").clicked() {
            let _ = rfd::FileDialog::new().add_filter("Pubgen Project", &["pubgen"]).pick_file().map(|path| { let _ = project.open(&path); });
        }
        if ui.button("Import XLSX").clicked() {
            let _ = rfd::FileDialog::new().add_filter("Excel Workbook", &["xlsx", "xlsm"]).pick_file().map(|path| {
                if let Ok(report) = pubgen_core::excel::ingest_workbook(&path) {
                    *project = OrchestratorProject { path: None, draft: Some(pubgen_core::WorkspaceDraft::from_ingest(path.file_stem().and_then(|name| name.to_str()).unwrap_or("publication").to_owned(), report)), status: "Workbook imported".into(), dirty: true, ..OrchestratorProject::default() };
                }
            });
        }
        if ui.button("Save .pubgen").clicked() {
            let path = project.path.clone().or_else(|| rfd::FileDialog::new().add_filter("Pubgen Project", &["pubgen"]).save_file());
            if let Some(path) = path { let _ = project.save(&path); }
        }
        if ui.button("Export XLSX").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("Excel Workbook", &["xlsx"]).save_file() { let _ = project.export_xlsx(&path); }
        }
    });
    if !project.status.is_empty() { ui.label(&project.status); }
    if let Some(draft) = &project.draft {
        ui.label(format!("{} · {} cards · {} CSV files", draft.metadata.title, draft.metadata.cards.len(), draft.csv.len()));
        for diagnostic in draft.validate() { ui.colored_label(if diagnostic.severity == pubgen_core::ValidationSeverity::Error { Color32::RED } else { Color32::YELLOW }, diagnostic.message); }
    }
}

pub fn render_orchestrator(
    app: &mut DesignApp,
    ui: &mut Ui,
    mode: &mut StudioMode,
    guard_dialog: &mut RecompileGuardDialog,
    has_existing_layout: bool,
) {
    ui.vertical(|ui| {
        // Orchestrator Action Header
        ui.horizontal(|ui| {
            ui.heading("📊 Publication Orchestrator");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(RichText::new("🚀 Compile & Open in Layout").strong().color(Color32::WHITE))
                    .on_hover_text("Compile the publication pages and switch to DesignCraft Layout canvas")
                    .clicked()
                {
                    if has_existing_layout {
                        guard_dialog.prompt("publication.designcraft");
                    } else {
                        // Trigger direct compilation
                        let _ = app.run("pubgen.compileProject", serde_json::json!({}));
                        *mode = StudioMode::DesignCraft;
                    }
                }
            });
        });

        ui.separator();

        // Main Pubgen Orchestrator Panel
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            designcraft_ui_egui::dock::pubgen::show(app, ui);
        });
    });
}
