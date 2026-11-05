use crate::mode::StudioMode;
use designcraft_ui_egui::DesignApp;
use egui::{Color32, RichText, Ui};

pub fn render_designcraft(app: &mut DesignApp, ui: &mut Ui, mode: &mut StudioMode) {
    if app.session.documents().is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.heading("🎨 DesignCraft Layout Canvas");
            ui.add_space(16.0);
            ui.label(RichText::new("No publication layout document is currently open.").color(Color32::GRAY));
            ui.add_space(24.0);
            if ui.button(RichText::new("⬅ Return to Orchestrator to Compile").size(16.0)).clicked() {
                *mode = StudioMode::Orchestrator;
            }
            ui.add_space(8.0);
            if ui.button(RichText::new("📄 Create Blank Document").size(14.0)).clicked() {
                let _ = app.run("file.new", serde_json::json!({}));
            }
        });
    } else {
        // Render DesignCraft's complete DTP canvas, dock, and panels
        app.ui(ui);
    }
}
