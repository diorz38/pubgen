pub mod guard_dialog;

use designcraft_ui_egui::DesignApp;
use egui::{Color32, RichText, Ui};
use crate::mode::StudioMode;
pub use guard_dialog::{GuardDialogAction, RecompileGuardDialog};

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
                if ui.button(RichText::new("🚀 Compile & Open in Layout").strong().color(Color32::WHITE))
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
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                designcraft_ui_egui::dock::pubgen::show(app, ui);
            });
    });
}
