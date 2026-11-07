//! PubGen's own menu bar (fork addition).
//!
//! Orchestrator Mode displays the 8 main menus matching the POS contract:
//! 1. File
//! 2. Publication
//! 3. Publication Tools
//! 4. Automation & Content
//! 5. DesignCraft
//! 6. View
//! 7. Shortcut
//! 8. Help
//!
//! When switched to DesignCraft Mode, the full host menu bar (File..Help) is restored.

use crate::DesignApp;

/// The egui memory key holding the active Studio mode ("designcraft" or "orchestrator").
const STUDIO_MODE: &str = "pubgen_studio_active_mode";

/// The egui memory key holding whether the PubGen menu bar is shown (temporary: resets to on).
const ON: &str = "pubgen_menus_on";

/// Whether Orchestrator mode is currently active (default is false: DesignCraft mode).
pub fn is_orchestrator_mode(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::unique(STUDIO_MODE))).unwrap_or(false)
}

/// Set active studio mode: Orchestrator (`true`) or DesignCraft (`false`).
pub fn set_orchestrator_mode(ctx: &egui::Context, orchestrator: bool) {
    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::unique(STUDIO_MODE), orchestrator);
        d.insert_temp(egui::Id::unique(ON), orchestrator);
    });
}

/// Whether the PubGen menu bar replaces DesignCraft's.
pub fn pubgen_menus_on(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::unique(ON))).unwrap_or_else(|| is_orchestrator_mode(ctx))
}

/// Show the PubGen menu bar (`true`) or DesignCraft's full one (`false`).
pub fn set_pubgen_menus_on(ctx: &egui::Context, on: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(ON), on));
}

fn mode_tab_button(ui: &mut egui::Ui, label: &str, active: bool, tip: &str) -> egui::Response {
    let t = crate::theme::Tokens::get(ui.ctx());
    let font = crate::theme::semibold(11.5);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font.clone(), if active { t.text_strong } else { t.text_dim });
    let size = egui::vec2(galley.size().x + 14.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label.to_string()));
    let p = ui.painter();

    if active {
        // ponytail: a single selected-pill treatment (row_selected + accent rim); if the modes ever
        // need distinct identities again, add one token per mode instead of a literal here.
        p.rect_filled(rect, 3.0, t.row_selected);
        p.rect_stroke(rect, 3.0, egui::Stroke::new(1.0, t.accent), egui::StrokeKind::Inside);
        p.galley(rect.center() - galley.size() / 2.0, galley, t.text_strong);
    } else if resp.hovered() {
        p.rect_filled(rect, 3.0, t.hover);
        let hover_galley = ui.painter().layout_no_wrap(label.to_string(), font, t.text);
        p.galley(rect.center() - hover_galley.size() / 2.0, hover_galley, t.text);
    } else {
        p.galley(rect.center() - galley.size() / 2.0, galley, t.text_dim);
    }

    resp.on_hover_text(tip)
}

/// Renders the mode switcher ("DesignCraft" | "Orchestrator"); the active pill uses the theme's
/// selection colours so both modes share one chrome treatment in every brightness.
pub fn render_mode_switch(app: &mut DesignApp, ui: &mut egui::Ui) {
    let orchestrator = is_orchestrator_mode(ui.ctx());

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

        // --- DesignCraft Button ---
        let dc_active = !orchestrator;
        let dc_tip = crate::i18n::tr(&app.ui.language, "DesignCraft Layout Studio");

        if mode_tab_button(ui, "DesignCraft", dc_active, &dc_tip).clicked() {
            set_orchestrator_mode(ui.ctx(), false);
        }

        // --- Orchestrator Button ---
        let orch_active = orchestrator;
        let orch_tip = crate::i18n::tr(&app.ui.language, "Publication Orchestrator");

        if mode_tab_button(ui, "Orchestrator", orch_active, &orch_tip).clicked() {
            set_orchestrator_mode(ui.ctx(), true);
            let _ = app.run("window.panel.activate", serde_json::json!({"panel": "pubgen"}));
        }

        ui.add_space(8.0);
    });
}

/// Top-level menus for Orchestrator Mode matching the POS main menu contract:
/// 1. File
/// 2. Publication
/// 3. Publication Tools
/// 4. Automation & Content
/// 5. DesignCraft
/// 6. View
/// 7. Shortcut
/// 8. Help
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Top {
    File,
    Publication,
    PublicationTools,
    AutomationContent,
    DesignCraft,
    View,
    Shortcut,
    Help,
}

/// Draws the Orchestrator menu bar in place of DesignCraft's. Returns `false`
/// when DesignCraft's menu bar should be drawn instead.
pub fn menu_bar(app: &mut DesignApp, ui: &mut egui::Ui) -> bool {
    if !pubgen_menus_on(ui.ctx()) {
        return false;
    }
    let rtl = crate::i18n::is_rtl(&app.ui.language);
    let top = [Top::File, Top::Publication, Top::PublicationTools, Top::AutomationContent, Top::DesignCraft, Top::View, Top::Shortcut, Top::Help];

    for entry in if rtl { top.into_iter().rev().collect::<Vec<_>>() } else { top.to_vec() } {
        match entry {
            Top::File => file_menu(app, ui),
            Top::Publication => publication_menu(app, ui),
            Top::PublicationTools => content_menu(app, ui),
            Top::AutomationContent => automation_content_menu(app, ui),
            Top::DesignCraft => designcraft_orchestrator_menu(app, ui),
            Top::View => view_menu(app, ui),
            Top::Shortcut => shortcut_menu(app, ui),
            Top::Help => help_menu(app, ui),
        }
    }
    true
}

/// 1. File Menu: project creation, open, save, template loading, and output export.
fn file_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let lang = app.ui.language.clone();
    let rtl = crate::i18n::is_rtl(&lang);
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(&lang, "File")), |ui| {
        popup(ui, rtl, 240.0, |ui| file_actions(app, ui));
    });
}

/// The File menu's rows: publication package open/save, template, output and quit.
fn file_actions(app: &mut DesignApp, ui: &mut egui::Ui) {
    let items: Vec<crate::menus::Item> =
        ["pubgen.newProject", "pubgen.openProject", "pubgen.saveProject", "pubgen.saveProjectAs", "pubgen.importXlsx", "pubgen.exportXlsx"]
            .iter()
            .map(|id| super::cmd_item(id, serde_json::Value::Null))
            .collect();
    super::bar::menu_column(app, ui, &[("Publication Project", items)]);
    ui.separator();

    let tmpl: Vec<crate::menus::Item> = ["app.openDialog"].iter().map(|id| super::cmd_item(id, serde_json::Value::Null)).collect();
    super::bar::menu_column(app, ui, &[("Template", tmpl)]);
    ui.separator();

    let pdf: Vec<crate::menus::Item> = ["app.exportPdf"].iter().map(|id| super::cmd_item(id, serde_json::Value::Null)).collect();
    super::bar::menu_column(app, ui, &[("Output", pdf)]);
    ui.separator();

    let exit: Vec<crate::menus::Item> = ["app.quit"].iter().map(|id| super::cmd_item(id, serde_json::Value::Null)).collect();
    super::bar::menu_column(app, ui, &[("Application", exit)]);
}

/// 2. Publication Menu: compilation pipeline and publication profiles.
fn publication_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let lang = app.ui.language.clone();
    let rtl = crate::i18n::is_rtl(&lang);
    let has_doc = app.session.active().is_some();
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(&lang, "Publication")), |ui| {
        popup(ui, rtl, 260.0, |ui| {
            pipeline(app, ui);
            ui.separator();
            submenu(ui, &lang, rtl, "Publication Profiles", 240.0, |ui| profiles_submenu(app, ui, &lang, has_doc));
        });
    });
}

/// 3. Publication Tools Menu: manual ingest, layout utilities, language mode and AI translation.
fn content_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let lang = app.ui.language.clone();
    let rtl = crate::i18n::is_rtl(&lang);
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(&lang, "Publication Tools")), |ui| {
        popup(ui, rtl, 260.0, |ui| content(app, ui, &lang, rtl));
    });
}

/// 4. Automation & Content Menu: cover generator, vector chart, TOC, and batch translation.
fn automation_content_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let lang = app.ui.language.clone();
    let rtl = crate::i18n::is_rtl(&lang);
    let has_doc = app.session.active().is_some();
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(&lang, "Automation & Content")), |ui| {
        popup(ui, rtl, 260.0, |ui| {
            let items: Vec<crate::menus::Item> = ["pubgen.publicationCover", "pubgen.vectorChart", "pubgen.tableOfContents"]
                .iter()
                .map(|id| super::cmd_item(id, serde_json::Value::Null))
                .collect();
            super::bar::menu_column(app, ui, &[("Automation", items)]);
            ui.separator();

            submenu(ui, &lang, rtl, "AI Translation & Terminology", 260.0, |ui| ai_translate(app, ui, &lang, has_doc));
            ui.separator();

            let batch: Vec<crate::menus::Item> = ["pubgen.generateFromExcel"].iter().map(|id| super::cmd_item(id, serde_json::Value::Null)).collect();
            super::bar::menu_column(app, ui, &[("Batch Generation", batch)]);
        });
    });
}

/// 5. DesignCraft Menu: seamless mode transfer and document opening.
fn designcraft_orchestrator_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let lang = app.ui.language.clone();
    let rtl = crate::i18n::is_rtl(&lang);
    crate::menus::menu_button(ui, crate::rtl::widget(ui, "DesignCraft"), |ui| {
        popup(ui, rtl, 240.0, |ui| {
            if ui.button(crate::rtl::widget(ui, "🎨 Switch to DesignCraft Studio")).clicked() {
                set_orchestrator_mode(ui.ctx(), false);
                ui.close();
            }
            ui.separator();

            let dc_items: Vec<crate::menus::Item> =
                ["app.openDialog", "app.save", "app.saveDialog"].iter().map(|id| super::cmd_item(id, serde_json::Value::Null)).collect();
            super::bar::menu_column(app, ui, &[("Layout Documents", dc_items)]);
            ui.separator();

            let prefs: Vec<crate::menus::Item> =
                ["window.shortcuts", "app.preferences"].iter().map(|id| super::cmd_item(id, serde_json::Value::Null)).collect();
            super::bar::menu_column(app, ui, &[("DesignCraft Settings", prefs)]);
        });
    });
}

/// 6. View Menu: mode switcher, panel docks, theme and language selection.
fn view_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let lang = app.ui.language.clone();
    let rtl = crate::i18n::is_rtl(&lang);
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(&lang, "View")), |ui| {
        popup(ui, rtl, 240.0, |ui| {
            let orch = is_orchestrator_mode(ui.ctx());
            if ui.selectable_label(orch, "✓  Publication Orchestrator").clicked() {
                set_orchestrator_mode(ui.ctx(), true);
                ui.close();
            }
            if ui.selectable_label(!orch, "    DesignCraft Layout Studio").clicked() {
                set_orchestrator_mode(ui.ctx(), false);
                ui.close();
            }
            ui.separator();

            let panels: Vec<crate::menus::Item> = ["window.pubgenPanel"].iter().map(|id| super::cmd_item(id, serde_json::Value::Null)).collect();
            super::bar::menu_column(app, ui, &[("Panels", panels)]);
            ui.separator();

            submenu(ui, &lang, rtl, "Language / Bahasa", 180.0, |ui| {
                let is_id = app.ui.language == "id";
                let is_en = app.ui.language.is_empty() || app.ui.language == "en";
                if ui.selectable_label(is_id, "🇮🇩 Bahasa Indonesia").clicked() {
                    let _ = app.run("app.language", serde_json::json!({"lang": "id"}));
                    ui.close();
                }
                if ui.selectable_label(is_en, "🇬🇧 English").clicked() {
                    let _ = app.run("app.language", serde_json::json!({"lang": ""}));
                    ui.close();
                }
            });

            submenu(ui, &lang, rtl, "Theme / Tema", 180.0, |ui| {
                for b in crate::theme::Brightness::ALL {
                    let is_active = app.ui.brightness == b;
                    let label = format!("{} {}", if is_active { "✓" } else { "  " }, crate::i18n::tr(&lang, b.label()));
                    if ui.selectable_label(is_active, label).clicked() {
                        let _ = app.run("window.brightness", serde_json::json!({"brightness": b.id()}));
                        ui.close();
                    }
                }
            });
        });
    });
}

/// 7. Shortcut Menu: quick access to export and project operations.
fn shortcut_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let items: Vec<crate::menus::Item> =
        [("app.exportPdf", "{}"), ("pubgen.saveProject", "{}"), ("pubgen.openProject", "{}"), ("preflight.run", "{}"), ("app.packageDialog", "{}")]
            .iter()
            .map(|(id, params)| super::cmd_item(id, serde_json::from_str(params).unwrap_or_default()))
            .collect();
    let lang = app.ui.language.clone();
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(&lang, "Shortcut")), |ui| {
        super::bar::menu_column(app, ui, &[("Shortcut", items)]);
    });
}

/// 8. Help Menu: documentation, guidelines, and about dialog.
fn help_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let tree = crate::menus::menu_tree();
    let Some((name, entries)) = tree.iter().find(|(n, _)| *n == "Help") else { return };
    let lang = app.ui.language.clone();
    let rtl = crate::i18n::is_rtl(&lang);
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(&lang, name)), |ui| {
        super::bar::menu_column(app, ui, &[(name, entries.to_vec())]);
        ui.separator();
        popup(ui, rtl, 240.0, |ui| about(ui, &lang));
    });
}

/// About PubGen… and the menu-bar toggle.
fn about(ui: &mut egui::Ui, lang: &str) {
    if item(ui, lang, "About PubGen…", true) {
        crate::dock::pubgen::with_state(ui.ctx(), |st| {
            st.show_about_dialog = true;
        });
        ui.close();
    }
    let pubgen_on = pubgen_menus_on(ui.ctx());
    let toggle_label = if pubgen_on { "Show DesignCraft Menus" } else { "Show PubGen Menus" };
    if item(ui, lang, toggle_label, true) {
        set_pubgen_menus_on(ui.ctx(), !pubgen_on);
        ui.close();
    }
}

/// The Popup layout every PubGen menu uses: right-to-left aware, at least `min_width` wide.
fn popup(ui: &mut egui::Ui, rtl: bool, min_width: f32, contents: impl FnOnce(&mut egui::Ui)) {
    if rtl {
        ui.set_max_width(320.0);
    }
    ui.with_layout(egui::Layout::top_down_justified(if rtl { egui::Align::Max } else { egui::Align::Min }), |ui| {
        ui.set_min_width(min_width);
        contents(ui);
    });
}

/// A menu row: label, enabled, clicked. A disabled row says why (the audit's P1-5).
fn item(ui: &mut egui::Ui, lang: &str, label: &str, enabled: bool) -> bool {
    let resp = ui.add_enabled(enabled, egui::Button::new(crate::rtl::widget(ui, crate::i18n::tr(lang, label))));
    let resp = if !enabled { resp.on_hover_text(crate::i18n::tr(lang, "Open a document first")) } else { resp };
    resp.clicked()
}

/// A submenu row that opens `contents`.
fn submenu(ui: &mut egui::Ui, lang: &str, rtl: bool, label: &str, min_width: f32, contents: impl FnOnce(&mut egui::Ui)) {
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(lang, label)), |ui| popup(ui, rtl, min_width, contents));
}

/// The generation pipeline: Excel generate, compile, charts, ToC, cover, headers and the audit.
fn pipeline(app: &mut DesignApp, ui: &mut egui::Ui) {
    let covering = crate::dock::pubgen::with_state(ui.ctx(), |st| st.cover_job.is_some());
    #[cfg(not(target_arch = "wasm32"))]
    let compiling = crate::dock::pubgen::with_state(ui.ctx(), |st| st.compile_job.is_some());
    #[cfg(target_arch = "wasm32")]
    let compiling = false;
    let items: Vec<crate::menus::Item> = [
        "pubgen.generateFromExcel",
        "pubgen.compileProject",
        "pubgen.profileCustomize",
        "pubgen.vectorChart",
        "pubgen.tableOfContents",
        "pubgen.publicationCover",
        "pubgen.headerFooter",
        "pubgen.audit",
    ]
    .iter()
    .map(|id| {
        let mut it = super::cmd_item(id, serde_json::Value::Null);
        if let crate::menus::Item::Cmd { label, id, .. } = &mut it {
            let running = match id.as_str() {
                "pubgen.compileProject" => compiling,
                "pubgen.publicationCover" => covering,
                _ => false,
            };
            if running {
                *label = format!("●  {label}");
            }
        }
        it
    })
    .collect();
    super::bar::menu_column(app, ui, &[("Publication", items)]);
}

/// Manual content: CSV ingest, layout utilities, the dual-language switch and AI translation.
fn content(app: &mut DesignApp, ui: &mut egui::Ui, lang: &str, rtl: bool) {
    let has_doc = app.session.active().is_some();
    submenu_items(app, ui, lang, "Manual Ingest & Tables", &["pubgen.pageFromCsv", "pubgen.tableFromCsv"]);
    submenu(ui, lang, rtl, "Publication Profiles", 240.0, |ui| profiles_submenu(app, ui, lang, has_doc));
    submenu_items(
        app,
        ui,
        lang,
        "Page & Layout Utilities",
        &["pubgen.profileCustomize", "pubgen.profileMargins", "pubgen.bpsMargins", "pubgen.marginsDialog"],
    );
    ui.separator();
    let dual = [super::cmd_item("pubgen.dualLanguage", serde_json::Value::Null)];
    super::tree::menu_items(app, ui, &dual, "Publication Tools");
    submenu(ui, lang, rtl, "AI Translate", 260.0, |ui| ai_translate(app, ui, lang, has_doc));
}

/// Submenu for choosing built-in or custom publication profiles, customizing, and applying profile margins.
fn profiles_submenu(app: &mut DesignApp, ui: &mut egui::Ui, lang: &str, has_doc: bool) {
    let (active_id, custom) = crate::dock::pubgen::with_state(ui.ctx(), |st| (st.active_profile_id.clone(), st.profile_custom.clone()));
    for p in crate::dock::pubgen::ProfileManager::builtin_profiles() {
        let is_active = active_id == p.id;
        let prefix = if is_active { "✓ " } else { "   " };
        let label = format!("{prefix}{}", p.name);
        if ui.button(crate::rtl::widget(ui, &label)).clicked() {
            crate::dock::pubgen::with_state(ui.ctx(), |st| {
                st.apply_profile(p.clone(), true);
            });
            ui.close();
        }
    }
    if let Some(cust) = custom {
        let is_active = active_id == "custom";
        let prefix = if is_active { "✓ " } else { "   " };
        let label = format!("{prefix}{}", cust.name);
        if ui.button(crate::rtl::widget(ui, &label)).clicked() {
            crate::dock::pubgen::with_state(ui.ctx(), |st| {
                st.apply_profile(cust.clone(), true);
            });
            ui.close();
        }
    }
    ui.separator();
    if item(ui, lang, "Customize Active Profile…", true) {
        crate::dock::pubgen::with_state(ui.ctx(), |st| {
            st.dialog_profile_customize_open = true;
        });
        ui.close();
    }
    if item(ui, lang, "Apply Profile Margins", has_doc) {
        let _ = crate::dock::pubgen::execute_set_page_margins(app);
        ui.close();
    }
}

/// A submenu whose rows are `UI_COMMANDS` ids: upstream rendering (gates, chords, checkmarks).
fn submenu_items(app: &mut DesignApp, ui: &mut egui::Ui, lang: &str, label: &str, ids: &[&str]) {
    let items: Vec<crate::menus::Item> = ids.iter().map(|id| super::cmd_item(id, serde_json::Value::Null)).collect();
    crate::menus::menu_button(ui, crate::rtl::widget(ui, crate::i18n::tr(lang, label)), |ui| {
        super::bar::menu_column(app, ui, &[(label, items)]);
    });
}

/// Submenu of AI translation actions for the current page.
fn ai_translate(app: &mut DesignApp, ui: &mut egui::Ui, lang: &str, has_doc: bool) {
    let backend_ready = crate::dock::pubgen::ai::backend_from_prefs(app).is_some();
    let is_bilingual = crate::dock::pubgen::with_state(ui.ctx(), |st| st.bilingual_output);
    let cur_page = app.session.active().and_then(|s| s.selection.page).or_else(|| crate::canvas::current_page(app));
    let review_target = cur_page.and_then(|p| crate::dock::pubgen::ai::detect_page_target(app, p));
    let table_target = cur_page.and_then(|p| crate::dock::pubgen::ai::detect_table_page_target(app, p));
    let is_review = review_target.is_some();
    let is_table = table_target.is_some();
    let can_page_ai = has_doc && (is_review || is_table) && backend_ready && is_bilingual;

    let page_label = if is_table {
        "Translate Table on this Page"
    } else if is_review {
        "Translate Review on this Page"
    } else {
        "Translate Selected Page"
    };
    if item(ui, lang, page_label, can_page_ai) {
        crate::dock::pubgen::with_state(ui.ctx(), |st| {
            crate::dock::pubgen::ai::execute_translate_selected_page(app, st, ui.ctx().clone());
        });
        ui.close();
    }

    let total_pages = app.session.active().map(|s| s.doc.page_count()).unwrap_or(0);
    let can_batch_tables = has_doc && total_pages > 0;
    if item(ui, lang, "Translate Table Pages…", can_batch_tables) {
        crate::dock::pubgen::with_state(ui.ctx(), |st| {
            st.show_table_range_popup = true;
        });
        ui.close();
    }
}

/// The PubGen menu of the DesignCraft menu bar: every PubGen action in one menu.
pub fn pubgen_menu(app: &mut DesignApp, ui: &mut egui::Ui) {
    let lang = app.ui.language.clone();
    let rtl = crate::i18n::is_rtl(&lang);
    crate::menus::menu_button(ui, crate::rtl::widget(ui, "Pubgen"), |ui| {
        popup(ui, rtl, 260.0, |ui| {
            pipeline(app, ui);
            ui.separator();
            content(app, ui, &lang, rtl);
            ui.separator();
            about(ui, &lang);
        });
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    fn frame(ctx: &egui::Context, app: &mut DesignApp, size: egui::Vec2, events: Vec<egui::Event>) -> Vec<String> {
        // Changing the menu set discards a pass to re-lay-out the bar; the discarded pass has no

        // shapes, so draw until a pass with shapes comes back.

        let mut texts = Vec::new();

        for _ in 0..3 {
            let time = ctx.cumulative_pass_nr() as f64 / 60.0;

            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),

                time: Some(time),

                events: if texts.is_empty() { events.clone() } else { Vec::new() },

                ..Default::default()
            };

            let mut output = ctx.run_ui(input, |ui| {
                // The real entry point: `menu_bar` here would be this module's own wrapper.

                egui::MenuBar::new().ui(ui, |ui| crate::menus::menu_bar(app, ui));
            });

            output.textures_delta.clear();

            texts = collect_texts(&output.shapes);

            if !texts.is_empty() {
                break;
            }
        }

        texts
    }

    fn collect_texts(shapes: &[egui::epaint::ClippedShape]) -> Vec<String> {
        fn collect(shape: &egui::Shape, out: &mut Vec<String>) {
            match shape {
                egui::Shape::Text(t) => out.push(t.galley.job.text.clone()),

                egui::Shape::Vec(shapes) => {
                    for s in shapes {
                        collect(s, out);
                    }
                }

                _ => {}
            }
        }

        let mut texts = Vec::new();

        for clipped in shapes {
            collect(&clipped.shape, &mut texts);
        }

        texts
    }

    fn app() -> DesignApp {
        DesignApp::new(designcraft_engine::Session::new(), Default::default())
    }

    #[test]

    fn pubgen_bar_keeps_the_host_menus_and_adds_the_pubgen_ones() {
        let ctx = egui::Context::default();

        set_pubgen_menus_on(&ctx, true);

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        let texts = frame(&ctx, &mut app, egui::vec2(1920.0, 200.0), Vec::new());

        let order = ["File", "Publication", "Publication Tools", "Automation & Content", "DesignCraft", "View", "Shortcut", "Help"];

        // The mode pills render first in the bar, and the DesignCraft pill's label is the same
        // string as the later "DesignCraft" menu, so match the *last* occurrence of each name.
        let positions: Vec<Option<usize>> = order.iter().map(|m| texts.iter().rposition(|t| t == m)).collect();

        for (i, top) in order.iter().enumerate() {
            assert!(positions[i].is_some(), "missing {top} in {texts:?}");
        }

        for (i, pair) in order.windows(2).enumerate() {
            assert!(positions[i] < positions[i + 1], "{} must come before {}: {:?}", pair[0], pair[1], positions);
        }
    }

    #[test]

    fn toggling_brings_designcrafts_menus_back_and_pubgens_away() {
        let ctx = egui::Context::default();

        set_pubgen_menus_on(&ctx, true);

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        let texts = frame(&ctx, &mut app, egui::vec2(1600.0, 200.0), Vec::new());

        let publication = texts.iter().position(|t| t == "Publication").expect("PubGen bar");

        assert!(publication < texts.iter().position(|t| t == "Help").expect("Help"), "Publication comes before Help");

        set_pubgen_menus_on(&ctx, false);

        assert!(!pubgen_menus_on(&ctx));

        let texts = frame(&ctx, &mut app, egui::vec2(1600.0, 200.0), Vec::new());

        assert!(texts.iter().any(|t| t == "Edit"), "DesignCraft's Edit menu is back: {texts:?}");

        assert!(!texts.iter().any(|t| t == "Publication"), "PubGen's menus are gone: {texts:?}");

        let pubgen = texts.iter().position(|t| t == "Pubgen").expect("the PubGen menu stays reachable");

        assert!(pubgen < texts.iter().position(|t| t == "Help").expect("Help"));

        set_pubgen_menus_on(&ctx, true);

        let texts = frame(&ctx, &mut app, egui::vec2(1920.0, 200.0), Vec::new());

        assert!(texts.iter().any(|t| t == "Publication"), "PubGen's menus are back: {texts:?}");

        assert!(!texts.iter().any(|t| t == "Edit"), "DesignCraft's Edit menu is replaced in Orchestrator mode: {texts:?}");

        assert!(!texts.iter().any(|t| t == "Pubgen"), "the fallback Pubgen menu hides while its own bar shows: {texts:?}");
    }

    #[test]

    fn every_pubgen_action_renders() {
        let ctx = egui::Context::default();

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };

        let mut output = ctx.run_ui(input, |ui| {
            egui::MenuBar::new().ui(ui, |ui| pubgen_menu(&mut app, ui));
        });

        output.textures_delta.clear();

        let texts = collect_texts(&output.shapes);

        assert!(texts.iter().any(|t| t == "Pubgen"), "the menu button is drawn: {texts:?}");
    }

    /// The Publication menu's rows, drawn like the popup draws them (the popup itself needs a
    /// click; the contents function is what the click opens).
    fn publication_rows(app: &mut DesignApp, ctx: &egui::Context) -> Vec<String> {
        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };

        let mut output = ctx.run_ui(input, |ui| {
            pipeline(app, ui);
        });

        output.textures_delta.clear();

        collect_texts(&output.shapes)
    }

    /// One action, one route: the Publication menu is exactly the pipeline — no second Excel row,
    /// no About (§2.4-1/2 of the accessibility audit).
    #[test]
    fn publication_menu_is_exactly_the_pipeline() {
        let ctx = egui::Context::default();

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        let rows = publication_rows(&mut app, &ctx);

        let excel = rows.iter().filter(|t| t.contains("Excel") || t.contains("XLSX")).count();

        assert_eq!(excel, 1, "one manifest import route only: {rows:?}");

        assert!(!rows.iter().any(|t| t.contains("Compile Wizard")), "the duplicate route is gone: {rows:?}");

        assert!(!rows.iter().any(|t| t.contains("About")), "About lives in Help: {rows:?}");
    }

    /// The File menu's rows, drawn like the popup draws them.
    fn file_rows(app: &mut DesignApp, ctx: &egui::Context) -> Vec<String> {
        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };

        let mut output = ctx.run_ui(input, |ui| {
            file_actions(app, ui);
        });

        output.textures_delta.clear();

        collect_texts(&output.shapes)
    }

    /// Open/Save/Save As live in File only: the Publication menu must not repeat them.
    #[test]
    fn publication_package_file_actions_live_in_the_file_menu_only() {
        let ctx = egui::Context::default();

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        let publication = publication_rows(&mut app, &ctx);

        assert!(!publication.iter().any(|t| t.contains("Open Publication")), "Open is in File: {publication:?}");

        assert!(!publication.iter().any(|t| t.contains("Save Publication")), "Save is in File: {publication:?}");

        let rows = file_rows(&mut app, &ctx);

        let open = rows.iter().position(|t| t.contains("Open Publication")).expect("Open row: {rows:?}");

        let save = rows.iter().position(|t| t.contains("Save Publication (")).expect("Save row: {rows:?}");

        let save_as = rows.iter().position(|t| t.contains("Save Publication As")).expect("Save As row: {rows:?}");

        assert!(open < save && save < save_as, "Save follows Open: {rows:?}");

        assert!(rows.iter().any(|t| t.contains(".pubgen")), "file actions name the .pubgen package: {rows:?}");
    }

    /// Help carries About PubGen… and the menu-bar toggle — the one place both live.

    #[test]

    fn help_menu_ends_with_about_and_the_bar_toggle() {
        let ctx = egui::Context::default();

        crate::theme::install_fonts(&ctx, "en");

        let app = app();

        let lang = app.ui.language.clone();

        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };

        let mut output = ctx.run_ui(input, |ui| about(ui, &lang));

        output.textures_delta.clear();

        let texts = collect_texts(&output.shapes);

        assert!(texts.iter().any(|t| t.contains("About PubGen")), "{texts:?}");

        assert!(texts.iter().any(|t| t.contains("DesignCraft Menus") || t.contains("PubGen Menus")), "{texts:?}");
    }

    /// The dual-language toggle is an upstream `Item::Cmd` row: its checkmark comes from
    /// `menus::state::checked`, not a hand-drawn prefix (the audit's P1-3).
    #[test]
    fn the_dual_language_toggle_draws_an_upstream_checkmark() {
        let ctx = egui::Context::default();

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        app.egui_ctx = Some(ctx.clone());

        let content = |app: &mut DesignApp| {
            let lang = app.ui.language.clone();

            let input =
                egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };

            let mut output = ctx.run_ui(input, |ui| content(app, ui, &lang, false));

            output.textures_delta.clear();

            collect_texts(&output.shapes)
        };

        assert!(content(&mut app).iter().any(|t| t.contains("✓") && t.contains("Dual Language Mode")), "bilingual by default, checked");

        crate::dock::pubgen::with_state(&ctx, |st| {
            st.bilingual_output = false;
        });

        assert!(content(&mut app).iter().any(|t| t.contains("Dual Language Mode") && !t.contains("✓")), "single after the toggle, unchecked");
    }

    /// Picking the BPS BRS profile turns dual-language off, so the menu row loses its checkmark.

    #[test]

    fn applying_the_brs_profile_unchecks_the_dual_language_row() {
        let ctx = egui::Context::default();

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        app.egui_ctx = Some(ctx.clone());

        crate::dock::pubgen::with_state(&ctx, |st| st.bilingual_output = true);

        let content = |app: &mut DesignApp| {
            let lang = app.ui.language.clone();

            let input =
                egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };

            let mut output = ctx.run_ui(input, |ui| content(app, ui, &lang, false));

            output.textures_delta.clear();

            collect_texts(&output.shapes)
        };

        assert!(content(&mut app).iter().any(|t| t.contains("✓") && t.contains("Dual Language Mode")), "seeded bilingual: checked");

        let _ = crate::menus::run::run_ui(&mut app, "pubgen.selectProfile", &serde_json::json!({"id": "bps_brs"}));

        let is_bilingual = crate::dock::pubgen::with_state(&ctx, |st| st.bilingual_output);

        assert!(!is_bilingual, "BRS profile is single-language");

        assert!(
            content(&mut app).iter().any(|t| t.contains("Dual Language Mode") && !t.contains("✓")),
            "dual-language row is unchecked after the BRS profile"
        );
    }

    /// A row whose background job runs carries a ● marker (the audit's P1-6).

    #[test]

    fn a_running_job_marks_its_publication_row() {
        let ctx = egui::Context::default();

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        app.egui_ctx = Some(ctx.clone());

        assert!(!publication_rows(&mut app, &ctx).iter().any(|t| t.starts_with('●')), "idle: no dot");

        crate::dock::pubgen::with_state(&ctx, |st| {
            st.compile_job = Some(Box::new(crate::dock::pubgen::compile::CompileJob::fake_running()));
        });

        let rows = publication_rows(&mut app, &ctx);

        let marked: Vec<_> = rows.iter().filter(|t| t.starts_with('●')).collect();

        assert_eq!(marked.len(), 1, "exactly the compiling row: {rows:?}");

        assert!(marked[0].contains("Compile Publication"), "{marked:?}");
    }

    /// Pillar 7: Publication Profile is accessible in the pipeline and via UI commands.

    #[test]

    fn publication_profile_accessible_from_menu_and_pipeline() {
        let ctx = egui::Context::default();

        crate::theme::install_fonts(&ctx, "en");

        let mut app = app();

        app.egui_ctx = Some(ctx.clone());

        let rows = publication_rows(&mut app, &ctx);

        assert!(rows.iter().any(|t| t.contains("Publication Profile")), "Publication Profile row in pipeline: {rows:?}");

        // Test running pubgen.profileCustomize UI command

        let _ = crate::menus::run::run_ui(&mut app, "pubgen.profileCustomize", &serde_json::json!({}));

        let is_open = crate::dock::pubgen::with_state(&ctx, |st| st.dialog_profile_customize_open);

        assert!(is_open, "dialog_profile_customize_open set by UI command");

        // Test running pubgen.selectProfile UI command

        let _ = crate::menus::run::run_ui(&mut app, "pubgen.selectProfile", &serde_json::json!({"id": "bps_brs"}));

        let active_id = crate::dock::pubgen::with_state(&ctx, |st| st.active_profile_id.clone());

        assert_eq!(active_id, "bps_brs", "active profile changed to bps_brs");

        // Test profiles_submenu renders

        let lang = app.ui.language.clone();

        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))), ..Default::default() };

        let mut output = ctx.run_ui(input, |ui| profiles_submenu(&mut app, ui, &lang, false));

        output.textures_delta.clear();

        let sub_texts = collect_texts(&output.shapes);

        assert!(sub_texts.iter().any(|t| t.contains("Berita Resmi Statistik")), "profiles submenu lists BRS: {sub_texts:?}");

        assert!(sub_texts.iter().any(|t| t.contains("Laporan Tahunan Korporat")), "profiles submenu lists Corporate: {sub_texts:?}");
    }
}
