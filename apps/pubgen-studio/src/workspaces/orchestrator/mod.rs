pub mod guard_dialog;
pub mod project;

use egui::Ui;
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
