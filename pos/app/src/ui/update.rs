use crate::main_ui::{App, AppTheme, AppView, CenterTab, DcStatus};
use iced::{window, Task};

use super::{message::*, UiApp};

/// Every enabled menu-contract id that has a real route in this module.
/// `header_footer_info` is permanently disabled and therefore exempt.
#[cfg(test)]
pub const ROUTED_MENU_IDS: &[&str] = &[
    // File
    "file_new",
    "file_open",
    "file_save",
    "file_import_sources",
    "file_import_excel",
    "file_load_template",
    "file_open_pdf",
    "app_exit",
    // Publication
    "pub_generate",
    "pub_batch_fill",
    "profile_bps_brs",
    "profile_bps_dda",
    "profile_generic_report",
    "profile_book",
    "profile_infographic",
    "profile_magazine",
    "profile_newsletter",
    "profile_slide",
    "tool_vector_chart",
    "tool_toc",
    "tool_cover_svg",
    "pub_qa_audit",
    // Publication Tools
    "open_document_settings",
    "pub_translate",
    "tool_glossary_import",
    "tool_glossary_export",
    // Automation & Content
    "auto_cover_studio",
    "auto_vector_chart",
    "auto_toc",
    "tool_ai_translate",
    "tool_glossary",
    "tool_diff",
    "auto_translate",
    // DesignCraft
    "dc_launch",
    "dc_kill",
    "dc_open_doc",
    "dc_reconnect",
    "dc_auto_extract",
    // View
    "view_home",
    "view_studio",
    "tab_content",
    "tab_preview",
    "tab_log",
    "theme_system",
    "theme_light",
    "theme_dark",
    "lang_id",
    "lang_en",
    // Shortcut
    "pub_batch_x4",
    // Help
    "help_bps",
    "help_ipc",
    "help_about",
];

pub fn update(ui: &mut UiApp, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            if ui.busy {
                ui.busy_tick = ui.busy_tick.wrapping_add(1);
            }
            tick(ui)
        }
        Message::DcProbe(probe) => {
            let probe = match std::sync::Arc::try_unwrap(probe.0) {
                Ok(res) => res,
                Err(_) => return Task::none(), // extra refs impossible; ignore defensively
            };
            dc_probe(ui, probe)
        }
        Message::JobDone(done) => {
            let temp = done
                .0
                .lock()
                .expect("job payload mutex poisoned")
                .take()
                .expect("job payload already consumed");
            ui.busy = false;
            ui.busy_action = None;
            ui.busy_tick = 0;
            merge(ui, temp);
            Task::none()
        }
        Message::WindowResized(size) => {
            ui.window_size = size;
            // Keep maximize state in sync (no direct maximized-changed event).
            window::latest()
                .and_then(|id| window::is_maximized(id))
                .map(|max| Message::Chrome(ChromeMsg::Maximized(max)))
        }
        Message::SystemTheme { dark } => {
            ui.system_dark = dark;
            Task::none()
        }
        Message::Chrome(msg) => chrome(ui, msg),
        Message::Menu(msg) => menu(ui, msg),
        Message::MenuKey(key) => menu_key_press(ui, key),
        Message::DcPillLaunched => {
            crate::actions::launch(&mut ui.app);
            Task::none()
        }
        Message::DcPillReprobe => {
            ui.app.client = None;
            ui.app.last_health = std::time::Instant::now() - std::time::Duration::from_secs(10);
            ui.app.log("reprobing IPC connection...");
            Task::none()
        }
        Message::PanelToggled(id) => {
            ui.touched_panels.insert(id);
            if ui.open_panels.contains(&id) {
                ui.open_panels.remove(&id);
            } else {
                ui.open_panels.insert(id);
            }
            Task::none()
        }
        Message::PanelsCollapseAll => {
            ui.open_panels.clear();
            ui.touched_panels
                .extend(super::PANEL_IDS.iter().copied());
            Task::none()
        }
        Message::PanelsExpandBasics => {
            ui.open_panels.clear();
            ui.open_panels.insert(super::BASIC_PANEL);
            ui.touched_panels
                .extend(super::PANEL_IDS.iter().copied());
            Task::none()
        }
        Message::OpenDialog(kind) => {
            ui.close_menu();
            ui.compile_menu_open = false;
            ui.active_dialog = Some(kind);
            Task::none()
        }
        Message::CloseDialog => {
            if ui.active_dialog == Some(DialogKind::OnboardingWizard) {
                let dur = ui.dialog_state.wizard_started_at.map(|t| t.elapsed().as_millis() as u64).unwrap_or(0);
                ui.telemetry.track_setup_cancelled(ui.dialog_state.wizard_step, dur);
            }
            ui.active_dialog = None;
            Task::none()
        }
        Message::DialogMsg(msg) => handle_dialog_msg(ui, msg),
        Message::HomeToggled => {
            ui.close_menu();
            ui.compile_menu_open = false;
            ui.active_dialog = None;
            ui.app.view = if ui.app.view == AppView::Home {
                AppView::Studio
            } else {
                AppView::Home
            };
            ui.editing_page = None;
            ui.confirm_delete = false;
            Task::none()
        }
        Message::CompilePressed => {
            ui.compile_menu_open = false;
            detach_action(ui, "pub_batch_fill")
        }
        Message::CompileMenuToggled => {
            ui.compile_menu_open = !ui.compile_menu_open;
            ui.close_menu();
            Task::none()
        }
        Message::LangSelected(lang) => {
            ui.app.ui_lang = lang;
            Task::none()
        }
        Message::HomeNewProject => {
            ui.app.project = crate::project::Project::new("Untitled");
            ui.app.view = AppView::Studio;
            ui.app.center_tab = CenterTab::Content;
            ui.editing_page = None;
            Task::none()
        }
        Message::HomeOpen => detach_action(ui, "file_open"),
        Message::HomeImportXlsx => detach_action(ui, "file_import_excel"),
        Message::HomeSampleBrs(path) => {
            let path = path.clone();
            detach(ui, move |app| {
                crate::actions::load_sample_brs(app, &path);
            })
        }
        Message::HomeSampleDda(path) => {
            let path = path.clone();
            detach(ui, move |app| {
                crate::actions::load_sample_dda(app, &path);
            })
        }
        Message::HomeLaunchDc => {
            crate::actions::launch(&mut ui.app);
            Task::none()
        }
        Message::ProfileCardSelected(profile_id, project_name) => {
            match ui.telemetry.variant() {
                crate::telemetry::ExperimentVariant::Control => {
                    ui.telemetry.track_setup_started("home_card");
                    ui.app.project = crate::project::Project::new(project_name);
                    ui.app.set_publication_profile(profile_id);
                    ui.app.view = AppView::Studio;
                    ui.app.center_tab = CenterTab::Content;
                    ui.telemetry.track_setup_completed(profile_id, 0);
                    Task::none()
                }
                crate::telemetry::ExperimentVariant::Wizard => {
                    ui.telemetry.track_setup_started("home_card");
                    ui.dialog_state.wizard_step = 1;
                    ui.dialog_state.wizard_profile_id = profile_id.to_string();
                    ui.dialog_state.wizard_title = project_name.to_string();
                    ui.dialog_state.wizard_year = "2026".to_string();
                    ui.dialog_state.wizard_started_at = Some(std::time::Instant::now());
                    ui.active_dialog = Some(DialogKind::OnboardingWizard);
                    Task::none()
                }
            }
        }
        Message::RecentOpened(path) => {
            detach(ui, move |app| {
                let _ = crate::actions::open_project_path(app, &path);
            })
        }
        Message::Action(id) => {
            ui.compile_menu_open = false;
            run_menu_action(ui, id)
        }
        Message::GoHome => {
            ui.close_menu();
            ui.compile_menu_open = false;
            ui.active_dialog = None;
            ui.app.view = AppView::Home;
            ui.editing_page = None;
            ui.confirm_delete = false;
            Task::none()
        }
        Message::EscapePressed => {
            // Topmost transient surface wins: modal → dialog → compile popover → menu bar.
            if ui.active_dialog.is_some() {
                ui.active_dialog = None;
                return Task::none();
            }
            // Inside the modal, Esc disarms the confirmation first (G10).
            if ui.editing_page.is_some() {
                if ui.confirm_delete {
                    ui.confirm_delete = false;
                } else {
                    ui.editing_page = None;
                }
                return Task::none();
            }
            if ui.compile_menu_open {
                ui.compile_menu_open = false;
                return Task::none();
            }
            ui.close_menu();
            Task::none()
        }
        Message::TabSelected(tab) => {
            ui.close_menu();
            ui.compile_menu_open = false;
            ui.active_dialog = None;
            ui.app.center_tab = tab;
            ui.app.view = AppView::Studio;
            ui.confirm_delete = false;
            if tab != CenterTab::Content {
                ui.editing_page = None;
            }
            Task::none()
        }
        Message::Input(input) => {
            let app = &mut ui.app;
            match input {
                InputMsg::LlmBase(s) => app.llm_base = s,
                InputMsg::LlmModel(s) => app.llm_model = s,
                InputMsg::LlmKey(s) => app.llm_key = s,
                InputMsg::GlossaryTerm(s) => app.glossary_term = s,
                InputMsg::GlossaryReplacement(s) => app.glossary_replacement = s,
                InputMsg::AiCoverTheme(s) => app.ai_cover_theme = s,
                InputMsg::ChartTitle(s) => app.chart_title = s,
            }
            Task::none()
        }
        Message::Combo(combo) => {
            match combo {
                ComboMsg::CoverStyle(style) => ui.app.cover_style = style,
                ComboMsg::ChartType(chart_type) => ui.app.chart_type = chart_type,
            }
            Task::none()
        }
        Message::GlossaryAdd => {
            let app = &mut ui.app;
            if !app.glossary_term.trim().is_empty() {
                app.project.glossary.push(crate::glossary::Entry {
                    term: app.glossary_term.trim().to_owned(),
                    replacement: app.glossary_replacement.trim().to_owned(),
                    do_not_translate: app.glossary_dnt,
                    case_preserve: true,
                });
                app.glossary_term.clear();
                app.glossary_replacement.clear();
                app.glossary_dnt = false;
            }
            Task::none()
        }
        Message::GlossaryRemove(index) => {
            if index < ui.app.project.glossary.len() {
                ui.app.project.glossary.remove(index);
            }
            Task::none()
        }
        Message::GlossaryDnt(v) => {
            ui.app.glossary_dnt = v;
            Task::none()
        }
        Message::CoverPromptCopy => {
            let prompt = crate::covers::generate_ai_cover_prompt(
                &ui.app.project.metadata,
                &ui.app.ai_cover_theme,
            );
            ui.app
                .log(format!("AI prompt copied to clipboard: {prompt}"));
            iced::clipboard::write(prompt)
        }
        Message::TocExtract(kind) => detach_toc(ui, kind),
        Message::TocCopy => {
            let app = &mut ui.app;
            if !app.toc_entries.is_empty() {
                let text_out = crate::toc::format_toc_story_text(
                    &app.toc_entries,
                    app.project.metadata.lang == crate::metadata_form::Lang::IdEn,
                    app.toc_entries[0].kind,
                );
                app.log("TOC text copied to clipboard");
                iced::clipboard::write(text_out)
            } else {
                Task::none()
            }
        }
        Message::SlotRemove(index) => {
            if index < ui.app.project.slots.len() {
                ui.app.project.slots.remove(index);
            }
            Task::none()
        }
        Message::SlotToggled(index) => {
            if ui.open_slots.contains(&index) {
                ui.open_slots.remove(&index);
            } else {
                ui.open_slots.insert(index);
            }
            Task::none()
        }
        Message::SlotUnname(index) => {
            let frame = ui.app.project.slots.get(index).map(|s| s.frame);
            if let Some(frame) = frame {
                ui.app.project.slots.remove(index);
                crate::actions::run_with_client(&mut ui.app, move |client, _app| {
                    crate::slots::assign_name(client, frame, "")
                });
            }
            Task::none()
        }
        Message::SlotBudget(index, text_in) => {
            if let Some(slot) = ui.app.project.slots.get_mut(index) {
                slot.budget = text_in.trim().parse::<u64>().ok().filter(|b| *b > 0);
            }
            Task::none()
        }
        Message::SlotSource(index, s) => {
            if let Some(slot) = ui.app.project.slots.get_mut(index) {
                slot.source = s;
            }
            Task::none()
        }
        Message::SlotText(index, s) => {
            if let Some(slot) = ui.app.project.slots.get_mut(index) {
                slot.text = s;
            }
            Task::none()
        }
        Message::ProfilePicked(profile) => {
            ui.app.set_publication_profile(&profile.id);
            Task::none()
        }
        Message::PreviewModeSelected(mode) => {
            ui.app.preview_mode = mode;
            Task::none()
        }
        Message::CoverSvgCopy => {
            let svg = crate::covers::render_cover_svg(
                &ui.app.project.metadata,
                ui.app.cover_style,
                595.0,
                842.0,
            );
            ui.app.log("Cover SVG markup copied to clipboard");
            iced::clipboard::write(svg)
        }
        Message::Metadata(msg) => metadata(ui, msg),
    }
}

fn metadata(ui: &mut UiApp, msg: MetadataMsg) -> Task<Message> {
    // Card lifecycle and the delete confirmation touch `UiApp` itself (modal
    // state, logging), so they run before the project borrow below.
    match &msg {
        MetadataMsg::CardOpen(idx) => {
            ui.editing_page =
                (*idx < ui.app.project.metadata.pages.len()).then_some(*idx);
            ui.confirm_delete = false;
            return Task::none();
        }
        MetadataMsg::CardClose => {
            ui.editing_page = None;
            ui.confirm_delete = false;
            return Task::none();
        }
        MetadataMsg::CardDeleteRequest => {
            // Arm only when a card is actually open; never mutate the project.
            ui.confirm_delete = ui.editing_page.is_some();
            return Task::none();
        }
        MetadataMsg::CardDeleteCancel => {
            ui.confirm_delete = false;
            return Task::none();
        }
        MetadataMsg::CardRemove(idx) => {
            remove_card(ui, *idx);
            return Task::none();
        }
        MetadataMsg::CardCoverTab(tab) => {
            ui.card_cover_tab = *tab;
            return Task::none();
        }
        _ => {}
    }

    let m = &mut ui.app.project.metadata;
    match msg {
        MetadataMsg::Title(v) => m.title = v,
        MetadataMsg::TitleEn(v) => m.title_en = v,
        MetadataMsg::Subtitle(v) => m.subtitle = v,
        MetadataMsg::Date(v) => m.date = v,
        MetadataMsg::Volume(v) => m.volume = v,
        MetadataMsg::Catalogue(v) => m.catalogue = v,
        MetadataMsg::Code(v) => m.code = v,
        MetadataMsg::ReleaseNo(v) => m.release_no = v,
        MetadataMsg::ReleaseDate(v) => m.release_date = v,
        MetadataMsg::ReleaseMonth(v) => m.release_month = v,
        MetadataMsg::Agency(v) => m.agency = v,
        MetadataMsg::Address(v) => m.address = v,
        MetadataMsg::Layout(v) => m.layout = v,
        MetadataMsg::Lang(v) => m.lang = v,
        MetadataMsg::Paper(v) => m.paper = v,
        MetadataMsg::MarginStep(side, delta) => {
            let field = match side {
                crate::metadata_form::MarginSide::Top => &mut m.margins.top,
                crate::metadata_form::MarginSide::Right => &mut m.margins.right,
                crate::metadata_form::MarginSide::Bottom => &mut m.margins.bottom,
                crate::metadata_form::MarginSide::Left => &mut m.margins.left,
            };
            *field = (*field + delta).clamp(
                crate::metadata_form::MARGINS_MIN,
                crate::metadata_form::MARGINS_MAX,
            );
        }
        MetadataMsg::CardScope(idx, scope) => {
            if let Some(card) = m.pages.get_mut(idx) {
                card.scope = scope;
            }
        }
        MetadataMsg::CardKind(idx, kind) => {
            if let Some(card) = m.pages.get_mut(idx) {
                card.kind = kind;
                card.sync_kind_options();
            }
        }
        MetadataMsg::CardTitle(idx, title) => {
            if let Some(card) = m.pages.get_mut(idx) {
                card.title = title;
            }
        }
        MetadataMsg::CardTitleEn(idx, title_en) => {
            if let Some(card) = m.pages.get_mut(idx) {
                card.title_en = title_en;
            }
        }
        MetadataMsg::CardChapterId(idx, chapter_id) => {
            if let Some(card) = m.pages.get_mut(idx) {
                card.chapter_id = chapter_id;
            }
        }
        MetadataMsg::CardSource(idx, src_opt) => {
            if let Some(card) = m.pages.get_mut(idx) {
                card.source = src_opt.clone();
                if let Some(ref s) = src_opt {
                    if let Some((_, bytes)) =
                        ui.app.project.content.iter().find(|(name, _)| name == s)
                    {
                        card.has_title =
                            crate::ingest::content_starts_with_title_style(s, bytes);
                    }
                }
            }
        }
        MetadataMsg::CardChartType(idx, ct) => {
            if let Some(card) = m.pages.get_mut(idx) {
                card.chart_type = Some(ct);
            }
        }
        MetadataMsg::CardImageArea(idx, ia) => {
            if let Some(card) = m.pages.get_mut(idx) {
                card.image_area = Some(ia);
            }
        }
        MetadataMsg::CardCover(idx, field, val) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut c = card.cover.take().unwrap_or_default();
                match field {
                    crate::metadata_form::CoverField::ReleaseNo => {
                        c.release_no = val.clone();
                        m.release_no = val;
                    }
                    crate::metadata_form::CoverField::Date => c.release_date = val,
                    crate::metadata_form::CoverField::Title => c.title1 = val,
                    crate::metadata_form::CoverField::Month => c.release_month = val,
                    crate::metadata_form::CoverField::PubOffice => c.pub_office = val,
                    crate::metadata_form::CoverField::Hero => c.image_hero = val,
                    crate::metadata_form::CoverField::Logo => c.logo = val,
                    crate::metadata_form::CoverField::Point => c.point = val,
                    crate::metadata_form::CoverField::Qr => c.qr = val,
                    crate::metadata_form::CoverField::PicName => c.pic_name = val,
                    crate::metadata_form::CoverField::PicPhoto => c.pic_photo = val,
                    crate::metadata_form::CoverField::Position => c.position = val,
                    crate::metadata_form::CoverField::Office => c.office = val,
                    crate::metadata_form::CoverField::Phone => c.phone = val,
                    crate::metadata_form::CoverField::Email => c.email = val,
                    crate::metadata_form::CoverField::Title2 => c.title2 = val,
                    crate::metadata_form::CoverField::TitleEn => c.title_en = val,
                    crate::metadata_form::CoverField::Year => c.year = val,
                    crate::metadata_form::CoverField::Volume => c.volume = val,
                    crate::metadata_form::CoverField::Catalogue => c.catalogue = val,
                    crate::metadata_form::CoverField::Code => c.code = val,
                    crate::metadata_form::CoverField::Agency => c.agency = val,
                    crate::metadata_form::CoverField::Palette => c.palette = val,
                }
                card.cover = Some(c);
            }
        }
        MetadataMsg::CardCoverTemplate(idx, tpl) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut c = card.cover.take().unwrap_or_default();
                c.template_path = tpl;
                card.cover = Some(c);
            }
        }
        MetadataMsg::CardCoverPalettePreset(idx, p_idx) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut c = card.cover.take().unwrap_or_default();
                c.palette = format!("{p_idx}");
                card.cover = Some(c);
            }
        }
        MetadataMsg::CardCoverAiTheme(idx, s) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut c = card.cover.take().unwrap_or_default();
                c.table_theme = s;
                card.cover = Some(c);
            }
        }
        MetadataMsg::CardCoverAiPreset(idx, s) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut c = card.cover.take().unwrap_or_default();
                c.event_theme = s;
                card.cover = Some(c);
            }
        }
        MetadataMsg::CardCoverAiPalette(idx, s) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut c = card.cover.take().unwrap_or_default();
                c.palette = s;
                card.cover = Some(c);
            }
        }
        MetadataMsg::CardCoverIncludeInsideBack(_idx, _b) => {}
        MetadataMsg::CardCoverAddMap(_idx, _b) => {}
        MetadataMsg::CardCoverMapStyle(_idx, _s) => {}

        MetadataMsg::CardSectionStart(idx, b) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.start_section = b;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionAutomatic(idx, b) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.automatic_numbering = b;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionStartPage(idx, p) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.start_page_number = p;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionPrefix(idx, pfx) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.section_prefix = pfx;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionStyle(idx, sty) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.numbering_style = sty;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionMarker(idx, mrk) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.section_marker = mrk;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionIncludePrefix(idx, b) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.include_prefix = b;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionChapterStyle(idx, sty) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.chapter_style = sty;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionChapterSource(idx, src) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.chapter_source = src;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionChapterStart(idx, p) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut s = card.section.take().unwrap_or_default();
                s.chapter_start = p;
                card.section = Some(s);
            }
        }
        MetadataMsg::CardSectionApplyForward(idx) => {
            if let Some(sec) = m.pages.get(idx).and_then(|c| c.section.clone()) {
                for i in (idx + 1)..m.pages.len() {
                    if m.pages[i].scope == crate::metadata_form::PageItemScope::NewPage {
                        m.pages[i].section = Some(sec.clone());
                    }
                }
            }
        }

        MetadataMsg::CardTableStyle(idx, s) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.table_options.take().unwrap_or_default();
                t.style = s;
                card.table_options = Some(t);
            }
        }
        MetadataMsg::CardTableTheme(idx, s) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.table_options.take().unwrap_or_default();
                t.theme = s;
                card.table_options = Some(t);
            }
        }
        MetadataMsg::CardTableLayoutPreset(idx, s) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.table_options.take().unwrap_or_default();
                t.layout_preset = s;
                card.table_options = Some(t);
            }
        }
        MetadataMsg::CardTableFirstRowHeader(idx, b) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.table_options.take().unwrap_or_default();
                t.first_row_header = b;
                card.table_options = Some(t);
            }
        }
        MetadataMsg::CardTableColNumbers(idx, b) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.table_options.take().unwrap_or_default();
                t.col_numbers = b;
                card.table_options = Some(t);
            }
        }
        MetadataMsg::CardTableFooter(idx, b) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.table_options.take().unwrap_or_default();
                t.table_footer = b;
                card.table_options = Some(t);
            }
        }
        MetadataMsg::CardTableRowConditions(idx, b) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.table_options.take().unwrap_or_default();
                t.row_conditions = b;
                card.table_options = Some(t);
            }
        }
        MetadataMsg::CardTableApplyToAll(idx) => {
            if let Some(t) = m.pages.get(idx).and_then(|c| c.table_options.clone()) {
                m.default_table_options = Some(t.clone());
                for card in &mut m.pages {
                    if card.kind == crate::metadata_form::PageKind::Table {
                        card.table_options = Some(t.clone());
                    }
                }
            }
        }

        MetadataMsg::CardTocTitle(idx, title) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.toc.take().unwrap_or_default();
                t.title = title;
                card.toc = Some(t);
            }
        }
        MetadataMsg::CardTocStyle(idx, style) => {
            let bilingual = m.is_bilingual();
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.toc.take().unwrap_or_default();
                // Picking a table/chart/paragraph list fills in its title per single or dual
                // lang mode (reference pubgen suppresses the overwrite when no default exists).
                if let Some(title) = crate::metadata_form::TocData::auto_title(style, bilingual) {
                    t.title = title.to_string();
                }
                t.style = style;
                card.toc = Some(t);
            }
        }
        MetadataMsg::CardTocLayout(idx, layout) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.toc.take().unwrap_or_default();
                t.layout = layout;
                card.toc = Some(t);
            }
        }
        MetadataMsg::CardTocFlag(idx, flag, on) => {
            if let Some(card) = m.pages.get_mut(idx) {
                let mut t = card.toc.take().unwrap_or_default();
                match flag {
                    super::message::TocFlag::HangingIndent => t.hanging_indent = on,
                    super::message::TocFlag::IncludeSelf => t.include_self = on,
                    super::message::TocFlag::PairBilingual => t.pair_bilingual = on,
                }
                card.toc = Some(t);
            }
        }
        MetadataMsg::CardAdd(scope) => {
            let mut pe = crate::metadata_form::PageEntry {
                kind: ui.add_page_kind,
                scope,
                title: String::new(),
                source: None,
                ..Default::default()
            };
            pe.sync_kind_options();
            m.pages.push(pe);
            ui.editing_page = Some(m.pages.len() - 1);
            ui.confirm_delete = false;
        }
        MetadataMsg::CardAddKind(kind) => {
            ui.add_page_kind = kind;
        }
        MetadataMsg::CardInsertBefore(idx) | MetadataMsg::CardInsertAfter(idx) => {
            // Insert relative to the edited card: before it, or right after it.
            let after = matches!(msg, MetadataMsg::CardInsertAfter(_));
            let at = if after { (idx + 1).min(m.pages.len()) } else { idx.min(m.pages.len()) };
            let mut pe = crate::metadata_form::PageEntry {
                kind: ui.add_page_kind,
                scope: crate::metadata_form::PageItemScope::NewPage,
                title: String::new(),
                source: None,
                ..Default::default()
            };
            pe.sync_kind_options();
            m.pages.insert(at, pe);
            ui.editing_page = Some(at);
            ui.confirm_delete = false;
        }
        // Card lifecycle variants are handled before the project borrow.
        _ => {}
    }
    Task::none()
}

/// Removes one publication card, keeps the editor index valid, and logs a
/// single line. This is the only mutation path used by the card UI (G10).
fn remove_card(ui: &mut UiApp, idx: usize) {
    let Some(title) = ui.app.project.metadata.pages.get(idx).map(|card| {
        let title = card.title.trim();
        if title.is_empty() {
            crate::i18n::tr(ui.app.ui_lang, "card_untitled").to_string()
        } else {
            title.to_string()
        }
    }) else {
        ui.confirm_delete = false;
        return;
    };

    ui.app.project.metadata.pages.remove(idx);
    if let Some(editing) = ui.editing_page {
        if editing == idx {
            ui.editing_page = None;
        } else if editing > idx {
            ui.editing_page = Some(editing - 1);
        }
    }
    ui.confirm_delete = false;
    ui.app
        .log(format!("card removed: {title} (page {})", idx + 1));
}

fn chrome(ui: &mut UiApp, msg: ChromeMsg) -> Task<Message> {
    match msg {
        ChromeMsg::DragTitlebar => {
            ui.close_menu();
            ui.maximized = false;
            window::latest().and_then(window::drag)
        }
        ChromeMsg::ToggleMaximize => {
            // Sync via resize events afterwards; optimistically flip for UI.
            ui.maximized = !ui.maximized;
            window::latest().and_then(window::toggle_maximize)
        }
        ChromeMsg::Minimize => window::latest().and_then(|id| window::minimize(id, true)),
        ChromeMsg::Close => window::latest().and_then(window::close),
        ChromeMsg::BeginResize(direction) => window::latest()
            .and_then(move |id| window::drag_resize(id, direction)),
        ChromeMsg::Maximized(max) => {
            ui.maximized = max;
            Task::none()
        }
    }
}

fn menu(ui: &mut UiApp, msg: MenuMsg) -> Task<Message> {
    match msg {
        MenuMsg::Toggled(i) => {
            ui.compile_menu_open = false;
            if ui.open_menu == Some(i) {
                ui.close_menu();
                return Task::none();
            }
            let defs = super::menu::runtime_menus(ui);
            ui.open_menu = Some(i);
            ui.menu_focus = Some(i);
            ui.menu_row = first_navigable(&defs, i);
            Task::none()
        }
        MenuMsg::Hovered(i) => {
            // Hover switches menus only while a popover is already open.
            if ui.open_menu.is_some() && ui.open_menu != Some(i) {
                let defs = super::menu::runtime_menus(ui);
                ui.open_menu = Some(i);
                ui.menu_focus = Some(i);
                ui.menu_row = first_navigable(&defs, i);
            }
            Task::none()
        }
        MenuMsg::Chosen(id) => {
            ui.close_menu();
            ui.compile_menu_open = false;
            run_menu_action(ui, id)
        }
    }
}

/// Indices of rows the keyboard may highlight (enabled actions only).
fn navigable_rows(def: Option<&super::menu::MenuDef>) -> Vec<usize> {
    def.map(|def| {
        def.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.is_actionable() && item.enabled)
            .map(|(index, _)| index)
            .collect()
    })
    .unwrap_or_default()
}

fn first_navigable(defs: &[super::menu::MenuDef], index: usize) -> Option<usize> {
    navigable_rows(defs.get(index)).first().copied()
}

fn step_row(rows: &[usize], current: Option<usize>, forward: bool) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    let pos = current.and_then(|c| rows.iter().position(|row| *row == c));
    match (pos, forward) {
        (Some(pos), true) => Some(rows[(pos + 1) % rows.len()]),
        (Some(pos), false) => Some(rows[(pos + rows.len() - 1) % rows.len()]),
        (None, true) => Some(rows[0]),
        (None, false) => Some(rows[rows.len() - 1]),
    }
}

/// Keyboard navigation across the bar and the open popover (G11 / F10).
/// Navigation keys are ignored until the bar or a popover is active.
fn menu_key_press(ui: &mut UiApp, key: MenuKey) -> Task<Message> {
    if key != MenuKey::FocusBar && ui.menu_focus.is_none() && ui.open_menu.is_none() {
        return Task::none();
    }

    let defs = super::menu::runtime_menus(ui);
    if defs.is_empty() {
        return Task::none();
    }
    let count = defs.len();

    match key {
        MenuKey::FocusBar => {
            if ui.menu_focus.is_some() || ui.open_menu.is_some() {
                ui.close_menu();
            } else {
                ui.menu_focus = Some(0);
                ui.open_menu = None;
                ui.menu_row = None;
            }
            Task::none()
        }
        MenuKey::Escape => {
            ui.close_menu();
            Task::none()
        }
        MenuKey::Left | MenuKey::Right => {
            let step = if key == MenuKey::Right {
                1
            } else {
                count - 1
            };
            match ui.open_menu {
                Some(open) => {
                    let next = (open + step) % count;
                    ui.open_menu = Some(next);
                    ui.menu_focus = Some(next);
                    ui.menu_row = first_navigable(&defs, next);
                }
                None => {
                    let current = ui.menu_focus.unwrap_or(0);
                    ui.menu_focus = Some((current + step) % count);
                }
            }
            Task::none()
        }
        MenuKey::Down | MenuKey::Up => {
            let forward = key == MenuKey::Down;
            match ui.open_menu {
                Some(open) => {
                    let rows = navigable_rows(defs.get(open));
                    ui.menu_row = step_row(&rows, ui.menu_row, forward);
                }
                None => {
                    if let Some(focus) = ui.menu_focus {
                        let rows = navigable_rows(defs.get(focus));
                        ui.open_menu = Some(focus);
                        ui.menu_row = if forward {
                            rows.first().copied()
                        } else {
                            rows.last().copied()
                        };
                    }
                }
            }
            Task::none()
        }
        MenuKey::Activate => {
            if let Some(open) = ui.open_menu {
                let Some(row) = ui.menu_row else {
                    ui.menu_row = first_navigable(&defs, open);
                    return Task::none();
                };
                let chosen = defs
                    .get(open)
                    .and_then(|def| def.items.get(row))
                    .filter(|item| item.is_actionable() && item.enabled)
                    .map(|item| item.id);
                if let Some(id) = chosen {
                    ui.close_menu();
                    ui.compile_menu_open = false;
                    return run_menu_action(ui, id);
                }
                Task::none()
            } else if let Some(focus) = ui.menu_focus {
                ui.open_menu = Some(focus);
                ui.menu_row = first_navigable(&defs, focus);
                Task::none()
            } else {
                Task::none()
            }
        }
    }
}

/// Routes stable menu item ids. UI-instant ids run sync on the real `App`;
/// everything that can block (dialogs, HTTP, file IO, zips) is detached.
fn run_menu_action(ui: &mut UiApp, id: &'static str) -> Task<Message> {
    // Engine-dependent actions fail fast with one log line when the engine is off.
    if super::ENGINE_ACTIONS.contains(&id)
        && matches!(ui.app.dc_status, DcStatus::Off)
    {
        ui.app.log("action skipped — engine offline");
        return Task::none();
    }

    // ---- instant, UI-thread ids (touch view state or the process handle) ----
    match id {
        "file_new" | "view_home" | "view_studio" | "tab_content" | "tab_preview" | "tab_log"
        | "theme_system" | "theme_light" | "theme_dark" | "lang_id" | "lang_en"
        | "app_exit" | "dc_launch" | "dc_kill" | "dc_reconnect"
        | "open_document_settings"
        | "tool_cover_svg" | "tool_vector_chart" | "tool_toc"
        | "auto_cover_studio" | "auto_vector_chart" | "auto_toc"
        | "tool_ai_translate" | "tool_glossary" | "tool_diff"
        | "auto_translate" => {
            run_ui_action(ui, id)
        }
        id if id.starts_with("profile_") => run_ui_action(ui, id),
        _ => detach_action(ui, id),
    }
}

/// Instant ids only — operate on the real `App` without leaving the UI thread.
fn run_ui_action(ui: &mut UiApp, id: &str) -> Task<Message> {
    match id {
        "file_new" => {
            match ui.telemetry.variant() {
                crate::telemetry::ExperimentVariant::Control => {
                    ui.telemetry.track_setup_started("menu_file_new");
                    let app = &mut ui.app;
                    app.project = crate::project::Project::new("Untitled");
                    app.last_output = None;
                    app.last_output_doc = None;
                    app.audit_report = None;
                    app.view = AppView::Studio;
                    app.center_tab = CenterTab::Content;
                    ui.editing_page = None;
                    app.log("new project created (Control Variant A)");
                    ui.telemetry.track_setup_completed("generic_report", 0);
                }
                crate::telemetry::ExperimentVariant::Wizard => {
                    ui.telemetry.track_setup_started("menu_file_new");
                    ui.dialog_state.wizard_step = 1;
                    ui.dialog_state.wizard_profile_id = "bps_dda".to_string();
                    ui.dialog_state.wizard_title = "Publikasi Baru".to_string();
                    ui.dialog_state.wizard_year = "2026".to_string();
                    ui.dialog_state.wizard_started_at = Some(std::time::Instant::now());
                    ui.active_dialog = Some(DialogKind::OnboardingWizard);
                    ui.app.log("opening onboarding wizard (Challenger Variant B)");
                }
            }
        }
        id if id.starts_with("profile_") => {
            let profile_id = id.strip_prefix("profile_").expect("prefix checked");
            ui.app.set_publication_profile(profile_id);
            ui.profile_picker = iced::widget::combo_box::State::new(
                crate::profiles::ProfileManager::builtin_profiles(),
            );
        }
        "view_home" => ui.app.view = AppView::Home,
        "view_studio" => ui.app.view = AppView::Studio,
        "open_document_settings" => {
            ui.app.view = AppView::Studio;
            ui.app.center_tab = CenterTab::Content;
            ui.open_panels.insert(super::BASIC_PANEL);
            ui.touched_panels.insert(super::BASIC_PANEL);
        }
        "tool_cover_svg" | "auto_cover_studio" => {
            ui.active_dialog = Some(DialogKind::CoverStudio);
        }
        "tool_vector_chart" | "auto_vector_chart" => {
            ui.active_dialog = Some(DialogKind::VectorChart);
        }
        "tool_toc" | "auto_toc" => {
            ui.active_dialog = Some(DialogKind::TocGenerator);
        }
        "auto_translate" => {
            return detach_action(ui, "pub_translate");
        }
        "tool_ai_translate" => {
            ui.active_dialog = Some(DialogKind::AiTranslation);
        }
        "tool_glossary" => {
            ui.active_dialog = Some(DialogKind::Glossary);
        }
        "tool_diff" => {
            ui.active_dialog = Some(DialogKind::Diff);
        }
        "tab_content" => {
            ui.app.center_tab = CenterTab::Content;
            ui.app.view = AppView::Studio;
        }
        "tab_preview" => {
            ui.app.center_tab = CenterTab::Preview;
            ui.app.view = AppView::Studio;
        }
        "tab_log" => {
            ui.app.center_tab = CenterTab::Log;
            ui.app.view = AppView::Studio;
        }
        "theme_system" => ui.app.theme = AppTheme::System,
        "theme_light" => ui.app.theme = AppTheme::Light,
        "theme_dark" => ui.app.theme = AppTheme::Dark,
        "lang_id" => ui.app.ui_lang = crate::i18n::UiLang::Id,
        "lang_en" => ui.app.ui_lang = crate::i18n::UiLang::En,
        "app_exit" => return window::latest().and_then(window::close),
        "dc_launch" => crate::actions::launch(&mut ui.app),
        "dc_kill" => crate::actions::kill_dc(&mut ui.app),
        "dc_reconnect" => {
            let app = &mut ui.app;
            app.client = None;
            app.last_health = std::time::Instant::now() - std::time::Duration::from_secs(10);
            app.log("attempting IPC reconnect...");
        }
        other => {
            ui.app.log(format!("unhandled ui action: {other}"));
        }
    }
    Task::none()
}

/// Everything that can touch `app`-owned state runs here, detached on a
/// blocking thread against a snapshot `App`; results come back via JobDone.
fn run_app_action(app: &mut App, id: &str) {
    match id {
        // ---- File ----
        "file_new" => {
            app.project = crate::project::Project::new("Untitled");
            app.last_output = None;
            app.last_output_doc = None;
            app.audit_report = None;
            app.view = AppView::Studio;
            app.log("new project created");
        }
        "file_open" => crate::actions::open_project(app),
        "file_save" => crate::actions::save_project(app),
        "file_import_sources" => crate::actions::import_sources(app),
        "file_import_excel" => crate::actions::import_excel_metadata(app),
        "file_load_template" => crate::actions::load_template(app),
        "file_open_dc" => crate::actions::open_in_designcraft(app),
        "file_open_pdf" => crate::actions::open_pdf_viewer(app),

        // ---- Publication ----
        id if id.starts_with("profile_") => {
            let profile_id = id.strip_prefix("profile_").expect("prefix checked");
            app.set_publication_profile(profile_id);
        }
        "pub_generate" => crate::actions::batch_generate(app, false, None),
        "pub_generate_publish" => crate::actions::batch_generate(app, true, None),
        "pub_batch_fill" => crate::actions::batch_fill(app, None),
        "pub_batch_x4" => crate::actions::batch_fill(app, Some("x4")),
        "pub_discover" => crate::actions::discover(app),
        "pub_fill_live" => crate::actions::fill_live(app),
        "pub_translate" => crate::actions::translate(app),
        "pub_qa_audit" => crate::actions::run_qa_audit(app),

        // ---- Publication Tools ----
        "tool_vector_chart" | "tool_vector_chart_exec" => crate::actions::generate_vector_chart(app),
        "tool_choropleth" => crate::actions::generate_choropleth_map(app),
        "tool_toc" | "tool_toc_exec" => crate::actions::extract_toc(app, crate::toc::TocKind::Contents),
        "tool_cover_svg" | "tool_cover_svg_exec" => crate::actions::export_cover_svg(app),
        "tool_glossary_import" => crate::actions::import_glossary(app),
        "tool_glossary_export" => crate::actions::export_glossary(app),

        // ---- DesignCraft ----
        "dc_open_doc" => crate::actions::open_in_designcraft(app),
        "dc_auto_extract" => match crate::process::auto_extract_portable() {
            Some(path) => app.log(format!("Portable DesignCraft ready: {}", path.display())),
            None => app.log("No portable archive found or extraction failed"),
        },

        // ---- Help ----
        "help_bps" => app.log("BPS guidelines: standard sizes A4 / B5, cover style Horizon/Minimal, bilingual ID/EN."),
        "help_ipc" => app.log(format!("IPC: 127.0.0.1:{} JSON-RPC over TCP (document.get, slot.fill, page.goto, export.pdf)", crate::main_ui::DC_PORT)),
        "help_about" => app.log("Publication Orchestrator Studio (POS) v0.1.0 — companion layout compiler for DesignCraft"),

        other => app.log(format!("unhandled menu action: {other}")),
    }
}

// ---- snapshot / detach / merge machinery (Phase 8) ----

/// Clones the sendable state of `app` into a temp `App` for a blocking run.
/// The live DC process handle stays on the UI side; the ControlClient moves
/// into the snapshot (same take() semantics as the legacy client runner).
fn snapshot(app: &mut App) -> App {
    let mut temp = App::new();
    temp.project = app.project.clone();
    temp.client = app.client.take();
    temp.dc_status = app.dc_status;
    temp.last_output = app.last_output.clone();
    temp.last_output_doc = app.last_output_doc.clone();
    temp.audit_report = app.audit_report.clone();
    temp.toc_entries = app.toc_entries.clone();
    temp.recent_projects = app.recent_projects.clone();
    temp.project_path = app.project_path.clone();
    temp.source_excel_path = app.source_excel_path.clone();
    temp.llm_base = app.llm_base.clone();
    temp.llm_model = app.llm_model.clone();
    temp.llm_key = app.llm_key.clone();
    temp.glossary_term = app.glossary_term.clone();
    temp.glossary_replacement = app.glossary_replacement.clone();
    temp.glossary_dnt = app.glossary_dnt;
    temp.selected_profile = app.selected_profile.clone();
    temp.view = app.view;
    temp.center_tab = app.center_tab;
    temp.ui_lang = app.ui_lang;
    temp
}

/// Copies the mutable results of a detached run back onto the real `App`.
fn merge(ui: &mut UiApp, temp: App) {
    ui.editing_page = None;
    let app = &mut ui.app;
    app.project = temp.project;
    app.client = temp.client;
    app.dc_status = temp.dc_status;
    app.last_output = temp.last_output;
    app.last_output_doc = temp.last_output_doc;
    app.audit_report = temp.audit_report;
    app.toc_entries = temp.toc_entries;
    app.recent_projects = temp.recent_projects;
    app.project_path = temp.project_path;
    app.source_excel_path = temp.source_excel_path;
    app.log = temp.log;
    app.glossary_term = temp.glossary_term;
    app.glossary_replacement = temp.glossary_replacement;
    app.glossary_dnt = temp.glossary_dnt;
    app.selected_profile = temp.selected_profile;
    app.view = temp.view;
    app.center_tab = temp.center_tab;
    ui.profile_picker = iced::widget::combo_box::State::new(
        crate::profiles::ProfileManager::builtin_profiles(),
    );
}

/// Runs `run_app_action(id)` on a blocking thread against a snapshot.
/// Single-flight: while a job runs, further detach requests are dropped.
fn detach_action(ui: &mut UiApp, id: &'static str) -> Task<Message> {
    if ui.busy {
        ui.app.log("action skipped — busy");
        return Task::none();
    }
    ui.busy = true;
    ui.busy_action = Some(id.to_string());
    ui.busy_tick = 0;
    detach(ui, move |app| run_app_action(app, id))
}

/// Generic snapshot-detach runner. `f` runs on a blocking thread.
fn detach(ui: &mut UiApp, f: impl FnOnce(&mut App) + Send + 'static) -> Task<Message> {
    let mut temp = snapshot(&mut ui.app);
    let payload = super::message::JobPayload::default();
    let slot = std::sync::Arc::clone(&payload.0);
    Task::perform(
        async move {
            let handle = tokio::task::spawn_blocking(move || {
                f(&mut temp);
                temp
            });
            let done = match handle.await {
                Ok(done) => done,
                Err(e) => {
                    let mut failed = App::new();
                    failed.log(format!("background task failed: {e}"));
                    failed
                }
            };
            if let Ok(mut guard) = slot.lock() {
                *guard = Some(done);
            }
            payload
        },
        Message::JobDone,
    )
}

/// Detached TOC extraction needs the target kind moved into the closure.
fn detach_toc(ui: &mut UiApp, kind: crate::toc::TocKind) -> Task<Message> {
    if ui.busy {
        ui.app.log("action skipped — busy");
        return Task::none();
    }
    ui.busy = true;
    ui.busy_action = Some(format!("tool_toc_{kind:?}"));
    ui.busy_tick = 0;
    detach(ui, move |app| {
        crate::actions::extract_toc(app, kind);
    })
}

/// Port of `poll_designcraft`: process-exit check runs sync (cheap try_wait);
/// the TCP health probe runs detached so the 250ms timeout never stalls paint.
fn tick(ui: &mut UiApp) -> Task<Message> {
    // 1. Process exit poll (sync — try_wait is free).
    if let Some(proc) = ui.app.dc.as_mut() {
        if let Some(status) = crate::process::poll(proc) {
            ui.app.log(format!("designcraft exited: {status}"));
            ui.app.dc = None;
            ui.app.client = None;
            ui.app.dc_status = DcStatus::Off;
            return Task::none();
        }
    }

    // 2. Health probe gating.
    let interval = if ui.app.client.is_none() {
        std::time::Duration::from_millis(500)
    } else {
        std::time::Duration::from_secs(2)
    };
    if ui.app.last_health.elapsed() < interval {
        return Task::none();
    }
    ui.app.last_health = std::time::Instant::now();

    if ui.app.client.is_some() {
        ui.app.dc_status = DcStatus::Ok;
        return Task::none();
    }

    // 3. Async connect probe (250ms timeout off the UI thread).
    Task::perform(
        async move {
            let handle = tokio::task::spawn_blocking(|| {
                crate::control_client::ControlClient::connect_timeout(
                    ("127.0.0.1", crate::main_ui::DC_PORT),
                    std::time::Duration::from_millis(250),
                )
            });
            let result = match handle.await {
                Ok(result) => result,
                Err(e) => Err(anyhow::anyhow!("{e}")),
            };
            super::message::ProbePayload(std::sync::Arc::new(result))
        },
        Message::DcProbe,
    )
}

/// Applies a health probe result to the pill state (legacy poll logic).
fn dc_probe(ui: &mut UiApp, probe: anyhow::Result<crate::control_client::ControlClient>) -> Task<Message> {
    match probe {
        Ok(client) => {
            ui.app.client = Some(client);
            ui.app.dc_status = DcStatus::Ok;
            ui.app.log(format!("control connection established (127.0.0.1:{})", crate::main_ui::DC_PORT));
        }
        Err(_) => {
            ui.app.dc_status = if ui.app.dc.is_some() {
                DcStatus::Booting
            } else {
                DcStatus::Off
            };
        }
    }
    Task::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> UiApp {
        let (ui, _task) = UiApp::new();
        ui
    }

    #[test]
    fn engine_actions_skip_once_when_engine_off() {
        let mut ui = fresh();
        ui.app.dc_status = DcStatus::Off;
        ui.app.log.clear();

        for id in super::super::ENGINE_ACTIONS {
            let _ = run_menu_action(&mut ui, id);
        }

        assert!(
            ui.app
                .log
                .iter()
                .all(|line| line == "action skipped — engine offline"),
            "each blocked engine action should log exactly the skip line: {:?}",
            ui.app.log
        );
        assert_eq!(ui.app.log.len(), super::super::ENGINE_ACTIONS.len());
        assert!(!ui.busy, "blocked actions must not start a job");
    }

    #[test]
    fn busy_actions_skip_with_busy_log() {
        let mut ui = fresh();
        ui.busy = true;
        ui.app.log.clear();

        let _ = detach_action(&mut ui, "file_save");

        assert_eq!(ui.app.log.last().map(String::as_str), Some("action skipped — busy"));
        assert!(ui.busy);
    }

    #[test]
    fn all_menu_ids_are_routed() {
        for lang in crate::i18n::UiLang::ALL {
            for def in super::super::menu::menu_contract(lang) {
                for item in def.items.iter().filter(|item| item.is_actionable()) {
                    if item.id == "header_footer_info" {
                        assert!(!item.enabled, "header_footer_info must stay disabled");
                        continue;
                    }
                    assert!(
                        ROUTED_MENU_IDS.contains(&item.id),
                        "enabled menu id has no route: {}",
                        item.id
                    );
                }
            }
        }
    }

    #[test]
    fn header_footer_info_is_exempt_but_disabled() {
        assert!(!ROUTED_MENU_IDS.contains(&"header_footer_info"));
        let defs = super::super::menu::menu_contract(crate::i18n::UiLang::Id);
        let row = defs
            .iter()
            .flat_map(|def| def.items.iter())
            .find(|item| item.id == "header_footer_info")
            .expect("header_footer_info row");
        assert!(!row.enabled);
        assert!(row.reason.is_some());
    }

    #[test]
    fn open_document_settings_opens_basic_panel_in_studio() {
        let mut ui = fresh();
        ui.app.view = AppView::Home;
        ui.app.center_tab = CenterTab::Log;
        ui.open_panels.clear();

        let _ = run_menu_action(&mut ui, "open_document_settings");

        assert_eq!(ui.app.view, AppView::Studio);
        assert_eq!(ui.app.center_tab, CenterTab::Content);
        assert!(ui.open_panels.contains(super::super::BASIC_PANEL));
        assert!(ui.touched_panels.contains(super::super::BASIC_PANEL));
    }

    #[test]
    fn busy_compile_row_is_disabled_and_marked() {
        let mut ui = fresh();
        ui.busy = true;
        let defs = super::super::menu::runtime_menus(&ui);
        let row = defs
            .iter()
            .flat_map(|def| def.items.iter())
            .find(|item| item.id == "pub_batch_fill")
            .expect("compile row");
        assert!(!row.enabled);
        assert!(row.label.starts_with('●'), "busy row should be marked: {}", row.label);
        assert!(row.reason.is_some());
    }

    #[test]
    fn open_pdf_row_follows_last_output() {
        let mut ui = fresh();
        ui.app.last_output = None;
        let disabled = super::super::menu::runtime_menus(&ui)
            .into_iter()
            .flat_map(|def| def.items)
            .find(|item| item.id == "file_open_pdf")
            .expect("open pdf row");
        assert!(!disabled.enabled);
        assert!(disabled.reason.is_some());

        ui.app.last_output = Some(std::path::PathBuf::from("out.pdf"));
        let enabled = super::super::menu::runtime_menus(&ui)
            .into_iter()
            .flat_map(|def| def.items)
            .find(|item| item.id == "file_open_pdf")
            .expect("open pdf row");
        assert!(enabled.enabled);
        assert!(enabled.reason.is_none());
    }
}


fn handle_dialog_msg(ui: &mut UiApp, msg: DialogMsg) -> Task<Message> {
    match msg {
        // Cover Studio
        DialogMsg::CoverStyle(style) => {
            ui.app.cover_style = style;
            Task::none()
        }
        DialogMsg::CoverIncludeBack(b) => {
            ui.dialog_state.cover_include_inside_back = b;
            Task::none()
        }
        DialogMsg::CoverAddMap(b) => {
            ui.dialog_state.cover_add_map = b;
            Task::none()
        }
        DialogMsg::CoverMapStyle(s) => {
            ui.dialog_state.cover_map_style = s;
            Task::none()
        }
        DialogMsg::CoverThemePreset(idx) => {
            ui.dialog_state.cover_theme_preset = idx;
            match idx {
                0 => {
                    ui.dialog_state.cover_primary_color = "#0072BC".into();
                    ui.dialog_state.cover_accent_color = "#F5871F".into();
                    ui.dialog_state.cover_dark_color = "#122647".into();
                }
                1 => {
                    ui.dialog_state.cover_primary_color = "#1E3A8A".into();
                    ui.dialog_state.cover_accent_color = "#D97706".into();
                    ui.dialog_state.cover_dark_color = "#0F172A".into();
                }
                2 => {
                    ui.dialog_state.cover_primary_color = "#047857".into();
                    ui.dialog_state.cover_accent_color = "#EAB308".into();
                    ui.dialog_state.cover_dark_color = "#064E3B".into();
                }
                3 => {
                    ui.dialog_state.cover_primary_color = "#BE123C".into();
                    ui.dialog_state.cover_accent_color = "#EA580C".into();
                    ui.dialog_state.cover_dark_color = "#1E293B".into();
                }
                4 => {
                    ui.dialog_state.cover_primary_color = "#0891B2".into();
                    ui.dialog_state.cover_accent_color = "#F59E0B".into();
                    ui.dialog_state.cover_dark_color = "#1E293B".into();
                }
                5 => {
                    ui.dialog_state.cover_primary_color = "#6C5CE7".into();
                    ui.dialog_state.cover_accent_color = "#A29BFE".into();
                    ui.dialog_state.cover_dark_color = "#1E293B".into();
                }
                _ => {}
            }
            Task::none()
        }
        DialogMsg::CoverTitle(s) => {
            ui.dialog_state.cover_title = s;
            Task::none()
        }
        DialogMsg::CoverTitle2(s) => {
            ui.dialog_state.cover_title2 = s;
            Task::none()
        }
        DialogMsg::CoverTitleEn(s) => {
            ui.dialog_state.cover_title_en = s;
            Task::none()
        }
        DialogMsg::CoverYear(s) => {
            ui.dialog_state.cover_year = s;
            Task::none()
        }
        DialogMsg::CoverVolume(s) => {
            ui.dialog_state.cover_volume = s;
            Task::none()
        }
        DialogMsg::CoverCatalogue(s) => {
            ui.dialog_state.cover_catalogue = s;
            Task::none()
        }
        DialogMsg::CoverPubNo(s) => {
            ui.dialog_state.cover_pub_no = s;
            Task::none()
        }
        DialogMsg::CoverAgency(s) => {
            ui.dialog_state.cover_agency = s;
            Task::none()
        }
        DialogMsg::CoverPrimaryColor(s) => {
            ui.dialog_state.cover_primary_color = s;
            Task::none()
        }
        DialogMsg::CoverAccentColor(s) => {
            ui.dialog_state.cover_accent_color = s;
            Task::none()
        }
        DialogMsg::CoverDarkColor(s) => {
            ui.dialog_state.cover_dark_color = s;
            Task::none()
        }
        DialogMsg::CoverAiTheme(s) => {
            ui.app.ai_cover_theme = s;
            Task::none()
        }
        DialogMsg::CoverExportSvg => detach_action(ui, "tool_cover_svg_exec"),

        // Vector Chart
        DialogMsg::ChartType(t) => {
            ui.app.chart_type = t;
            Task::none()
        }
        DialogMsg::ChartTitle(s) => {
            ui.app.chart_title = s;
            Task::none()
        }
        DialogMsg::ChartTitleEn(s) => {
            ui.dialog_state.chart_title_en = s;
            Task::none()
        }
        DialogMsg::ChartCategoryCol(s) => {
            ui.dialog_state.chart_category_col = s;
            Task::none()
        }
        DialogMsg::ChartSeries1Col(s) => {
            ui.dialog_state.chart_series_1_col = s;
            Task::none()
        }
        DialogMsg::ChartSeries1Label(s) => {
            ui.dialog_state.chart_series_1_label = s;
            Task::none()
        }
        DialogMsg::ChartSeries2Enabled(b) => {
            ui.dialog_state.chart_series_2_enabled = b;
            Task::none()
        }
        DialogMsg::ChartSeries2Col(s) => {
            ui.dialog_state.chart_series_2_col = s;
            Task::none()
        }
        DialogMsg::ChartSeries2Label(s) => {
            ui.dialog_state.chart_series_2_label = s;
            Task::none()
        }
        DialogMsg::ChartSeries3Enabled(b) => {
            ui.dialog_state.chart_series_3_enabled = b;
            Task::none()
        }
        DialogMsg::ChartSeries3Col(s) => {
            ui.dialog_state.chart_series_3_col = s;
            Task::none()
        }
        DialogMsg::ChartSeries3Label(s) => {
            ui.dialog_state.chart_series_3_label = s;
            Task::none()
        }
        DialogMsg::ChartDataStartRow(r) => {
            ui.dialog_state.chart_data_start_row = r;
            Task::none()
        }
        DialogMsg::ChartDataEndRow(r) => {
            ui.dialog_state.chart_data_end_row = r;
            Task::none()
        }
        DialogMsg::ChartValueFormat(fmt) => {
            ui.dialog_state.chart_value_format = fmt;
            Task::none()
        }
        DialogMsg::ChartShowLabels(b) => {
            ui.dialog_state.chart_show_labels = b;
            Task::none()
        }
        DialogMsg::ChartShowLegend(b) => {
            ui.dialog_state.chart_show_legend = b;
            Task::none()
        }
        DialogMsg::ChartShowGrid(b) => {
            ui.dialog_state.chart_show_grid = b;
            Task::none()
        }
        DialogMsg::ChartMaxVal(s) => {
            ui.dialog_state.chart_max_val = s;
            Task::none()
        }
        DialogMsg::ChartMapPalette(idx) => {
            ui.dialog_state.chart_map_palette = idx;
            Task::none()
        }
        DialogMsg::ChartRawCsv(s) => {
            ui.dialog_state.chart_raw_csv = s;
            Task::none()
        }
        DialogMsg::ChartLoadSample => {
            ui.dialog_state.chart_raw_csv = "Tahun,Pertanian,Industri,Jasa\n2021,3.8,4.2,5.1\n2022,4.1,4.5,5.6\n2023,3.9,4.8,5.9\n2024,4.3,5.1,6.2\n2025,4.5,5.4,6.5".to_string();
            Task::none()
        }
        DialogMsg::ChartGenerateSvg => detach_action(ui, "tool_vector_chart_exec"),
        DialogMsg::ChartGenerateChoropleth => detach_action(ui, "tool_choropleth"),

        // Table of Contents
        DialogMsg::TocTitle(s) => {
            ui.dialog_state.toc_title = s;
            Task::none()
        }
        DialogMsg::TocKind(k) => {
            ui.dialog_state.toc_kind = k;
            Task::none()
        }
        DialogMsg::TocLayout(l) => {
            ui.dialog_state.toc_layout = l;
            Task::none()
        }
        DialogMsg::TocHangingIndent(b) => {
            ui.dialog_state.toc_hanging_indent = b;
            Task::none()
        }
        DialogMsg::TocIncludeSelf(b) => {
            ui.dialog_state.toc_include_self = b;
            Task::none()
        }
        DialogMsg::TocPairBilingual(b) => {
            ui.dialog_state.toc_pair_bilingual = b;
            Task::none()
        }
        DialogMsg::TocExtract(kind) => detach_toc(ui, kind),
        DialogMsg::TocCopy => {
            let is_bilingual = ui.app.project.metadata.lang == crate::metadata_form::Lang::IdEn;
            let kind = ui.app.toc_entries.first().map(|e| e.kind).unwrap_or(crate::toc::TocKind::Contents);
            let text = crate::toc::format_toc_story_text(&ui.app.toc_entries, is_bilingual, kind);
            iced::clipboard::write(text)
        }

        // AI Translation
        DialogMsg::AiProvider(p) => {
            ui.dialog_state.ai_provider = p;
            match p {
                0 => {
                    ui.app.llm_base = "https://api.openai.com/v1".into();
                    ui.app.llm_model = "gpt-4o-mini".into();
                }
                1 => {
                    ui.app.llm_base = "https://api.groq.com/openai/v1".into();
                    ui.app.llm_model = "llama-3.3-70b-versatile".into();
                }
                2 => {
                    ui.app.llm_base = "https://api.deepseek.com/v1".into();
                    ui.app.llm_model = "deepseek-chat".into();
                }
                3 => {
                    ui.app.llm_key.clear();
                }
                _ => {}
            }
            Task::none()
        }
        DialogMsg::AiBaseUrl(s) => {
            ui.app.llm_base = s;
            Task::none()
        }
        DialogMsg::AiModel(s) => {
            ui.app.llm_model = s;
            Task::none()
        }
        DialogMsg::AiApiKey(s) => {
            ui.app.llm_key = s;
            Task::none()
        }
        DialogMsg::AiBilingual(b) => {
            ui.dialog_state.ai_bilingual = b;
            Task::none()
        }
        DialogMsg::AiTranslateProject => detach_action(ui, "pub_translate"),

        // Glossary
        DialogMsg::GlossaryFilter(s) => {
            ui.dialog_state.glossary_filter = s;
            Task::none()
        }
        DialogMsg::GlossaryTerm(s) => {
            ui.app.glossary_term = s;
            Task::none()
        }
        DialogMsg::GlossaryReplacement(s) => {
            ui.app.glossary_replacement = s;
            Task::none()
        }
        DialogMsg::GlossaryDnt(b) => {
            ui.app.glossary_dnt = b;
            Task::none()
        }
        DialogMsg::GlossaryAdd => {
            let app = &mut ui.app;
            if !app.glossary_term.trim().is_empty() {
                app.project.glossary.push(crate::glossary::Entry {
                    term: app.glossary_term.trim().to_owned(),
                    replacement: app.glossary_replacement.trim().to_owned(),
                    do_not_translate: app.glossary_dnt,
                    case_preserve: true,
                });
                app.glossary_term.clear();
                app.glossary_replacement.clear();
                app.glossary_dnt = false;
            }
            Task::none()
        }
        DialogMsg::GlossaryRemove(idx) => {
            if idx < ui.app.project.glossary.len() {
                ui.app.project.glossary.remove(idx);
            }
            Task::none()
        }
        DialogMsg::GlossaryImport => detach_action(ui, "tool_glossary_import"),
        DialogMsg::GlossaryExport => detach_action(ui, "tool_glossary_export"),

        // Translation Diff
        DialogMsg::DiffTranslate => detach_action(ui, "pub_translate"),

        // Onboarding Wizard (A/B Test Variant B)
        DialogMsg::WizardNext => {
            if ui.dialog_state.wizard_step < 3 {
                ui.dialog_state.wizard_step += 1;
            }
            Task::none()
        }
        DialogMsg::WizardPrev => {
            if ui.dialog_state.wizard_step > 1 {
                ui.dialog_state.wizard_step -= 1;
            }
            Task::none()
        }
        DialogMsg::WizardSelectProfile(p) => {
            ui.dialog_state.wizard_profile_id = p;
            Task::none()
        }
        DialogMsg::WizardTitle(t) => {
            ui.dialog_state.wizard_title = t;
            Task::none()
        }
        DialogMsg::WizardYear(y) => {
            ui.dialog_state.wizard_year = y;
            Task::none()
        }
        DialogMsg::WizardCancel => {
            let dur = ui.dialog_state.wizard_started_at.map(|t| t.elapsed().as_millis() as u64).unwrap_or(0);
            ui.telemetry.track_setup_cancelled(ui.dialog_state.wizard_step, dur);
            ui.active_dialog = None;
            Task::none()
        }
        DialogMsg::WizardFinish => {
            let profile_id = if ui.dialog_state.wizard_profile_id.is_empty() {
                "bps_dda".to_string()
            } else {
                ui.dialog_state.wizard_profile_id.clone()
            };
            let title = if ui.dialog_state.wizard_title.trim().is_empty() {
                match profile_id.as_str() {
                    "bps_dda" => "Daerah Dalam Angka",
                    "bps_brs" => "Berita Resmi Statistik",
                    _ => "Untitled Publication",
                }
                .to_string()
            } else {
                ui.dialog_state.wizard_title.trim().to_string()
            };
            let dur = ui.dialog_state.wizard_started_at.map(|t| t.elapsed().as_millis() as u64).unwrap_or(0);
            ui.telemetry.track_setup_completed(&profile_id, dur);

            ui.app.project = crate::project::Project::new(&title);
            ui.app.set_publication_profile(&profile_id);
            if !ui.dialog_state.wizard_year.trim().is_empty() {
                ui.app.project.metadata.date = ui.dialog_state.wizard_year.trim().to_string();
            }
            ui.app.view = AppView::Studio;
            ui.app.center_tab = CenterTab::Content;
            ui.active_dialog = None;
            ui.app.log(&format!("project initialized via wizard ({}) in {}ms", profile_id, dur));
            Task::none()
        }
    }
}
