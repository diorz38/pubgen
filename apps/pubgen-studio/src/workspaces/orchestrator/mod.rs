pub mod guard_dialog;
pub mod project;

use crate::mode::StudioMode;
use designcraft_ui_egui::DesignApp;
use egui::{Color32, RichText, Ui};
pub use guard_dialog::{GuardDialogAction, RecompileGuardDialog};
pub use project::OrchestratorTabs;

/// Close confirmation for a dirty workspace tab, like DesignCraft's unsaved-document prompt.
pub fn render_close_dialog(ui: &mut Ui, tabs: &mut OrchestratorTabs) {
    let Some(index) = tabs.pending_close else { return };
    let title = tabs.title(index);
    let mut confirm = false;
    let mut cancel = false;
    egui::Window::new("Close publication workspace?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ui.ctx(), |ui| {
            ui.label(format!("\"{title}\" has unsaved changes."));
            ui.horizontal(|ui| {
                if ui.button("Close without saving").clicked() {
                    confirm = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });
        });
    if confirm {
        tabs.close(ui.ctx(), index);
    } else if cancel {
        tabs.pending_close = None;
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
