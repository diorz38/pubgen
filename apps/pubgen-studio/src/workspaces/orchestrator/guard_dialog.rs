use egui::{Color32, RichText, Window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardDialogAction {
    None,
    Overwrite,
    SaveNewVersion,
    Cancel,
}

pub struct RecompileGuardDialog {
    pub open: bool,
    pub existing_filename: String,
}

impl Default for RecompileGuardDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl RecompileGuardDialog {
    pub fn new() -> Self {
        Self { open: false, existing_filename: "publication.designcraft".to_string() }
    }

    pub fn prompt(&mut self, filename: impl Into<String>) {
        self.existing_filename = filename.into();
        self.open = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> GuardDialogAction {
        if !self.open {
            return GuardDialogAction::None;
        }

        let mut action = GuardDialogAction::None;

        Window::new("⚠️ Existing Layout Detected").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(
            ctx,
            |ui| {
                ui.set_width(420.0);
                ui.add_space(8.0);

                ui.label(RichText::new("A compiled layout already exists for this publication:").size(14.0));
                ui.add_space(4.0);
                ui.label(RichText::new(&self.existing_filename).strong().color(Color32::from_rgb(0x3B, 0x82, 0xF6)));
                ui.add_space(8.0);

                ui.label(
                    RichText::new(
                        "Re-compiling will generate fresh pages from source Excel/markdown data. \
                         Any manual typography, frame positioning, or styling adjustments made \
                         in DesignCraft mode might be lost if overwritten.",
                    )
                    .color(Color32::GRAY),
                );
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("💾 Save as Version (v2)").strong())
                        .on_hover_text("Preserve existing layout and compile into a new version")
                        .clicked()
                    {
                        self.open = false;
                        action = GuardDialogAction::SaveNewVersion;
                    }

                    if ui
                        .button(RichText::new("⚠️ Overwrite Existing").color(Color32::from_rgb(0xEF, 0x44, 0x44)))
                        .on_hover_text("Replace the current layout with freshly compiled pages")
                        .clicked()
                    {
                        self.open = false;
                        action = GuardDialogAction::Overwrite;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.open = false;
                            action = GuardDialogAction::Cancel;
                        }
                    });
                });
                ui.add_space(8.0);
            },
        );

        action
    }
}
