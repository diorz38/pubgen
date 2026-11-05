use egui::{Color32, RichText, Ui, Vec2};
use crate::mode::StudioMode;

pub fn render_top_bar(
    ui: &mut Ui,
    active_mode: &mut StudioMode,
    project_name: Option<&str>,
    has_document: bool,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(12.0, 0.0);

        // App Branding
        ui.label(RichText::new("PUBGEN STUDIO").strong().size(15.0).color(Color32::from_rgb(0x3B, 0x82, 0xF6)));

        ui.separator();

        // Mode Switcher Pills
        let orch_selected = *active_mode == StudioMode::Orchestrator;
        let dc_selected = *active_mode == StudioMode::DesignCraft;

        if ui.selectable_label(orch_selected, StudioMode::Orchestrator.label())
            .on_hover_text(StudioMode::Orchestrator.description())
            .clicked()
        {
            *active_mode = StudioMode::Orchestrator;
        }

        let dc_label = if has_document {
            StudioMode::DesignCraft.label().to_string()
        } else {
            format!("{} (🔒)", StudioMode::DesignCraft.label())
        };

        if ui.selectable_label(dc_selected, dc_label)
            .on_hover_text(StudioMode::DesignCraft.description())
            .clicked()
        {
            *active_mode = StudioMode::DesignCraft;
        }

        ui.separator();

        // Project Info
        if let Some(name) = project_name {
            ui.label(RichText::new(format!("📁 {name}")).italics());
        } else {
            ui.label(RichText::new("📁 [No Project Loaded]").color(Color32::GRAY));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if has_document {
                ui.label(RichText::new("✔ Layout Active").color(Color32::from_rgb(0x10, 0xB9, 0x81)));
            } else {
                ui.label(RichText::new("⏳ Awaiting Compilation").color(Color32::from_rgb(0xF5, 0x9E, 0x0B)));
            }
        });
    });
}

