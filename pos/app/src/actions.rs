//! Pure application actions extracted from the legacy UI panels.
//!
//! These functions mutate `App` and talk to DesignCraft; they contain no UI code.

use crate::control_client::ControlClient;
use crate::ingest;
use crate::main_ui::{App, DcStatus};
use crate::slots::{self, Slot};
use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

pub fn run_with_client<T>(app: &mut App, f: impl FnOnce(&mut ControlClient, &mut App) -> Result<T>) -> Option<T> {
    match app.client.take() {
        Some(mut client) => {
            let result = f(&mut client, app);
            match result {
                Ok(value) => {
                    app.client = Some(client);
                    Some(value)
                }
                Err(e) => {
                    app.log(format!("error: {e}"));
                    let err_msg = e.to_string().to_lowercase();
                    if err_msg.contains("broken pipe")
                        || err_msg.contains("connection reset")
                        || err_msg.contains("eof")
                        || err_msg.contains("closed")
                    {
                        app.client = None;
                        app.dc_status = if app.dc.is_some() { DcStatus::Booting } else { DcStatus::Off };
                        app.log("control connection lost, will attempt reconnect");
                    } else {
                        app.client = Some(client);
                    }
                    None
                }
            }
        }
        None => {
            app.log("not connected — Launch first");
            None
        }
    }
}

pub fn launch(app: &mut App) {
    if app.dc.is_some() {
        app.log("designcraft already running");
        return;
    }
    let Some(exe) = crate::process::resolve_exe() else {
        app.log("no designcraft executable found — set PUBGEN_DESIGNCRAFT_EXE");
        return;
    };
    match crate::process::spawn(&exe, crate::main_ui::DC_PORT) {
        Ok(proc) => {
            app.log(format!("launched {}", exe.display()));
            app.dc = Some(proc);
            app.dc_status = DcStatus::Booting;
            app.client = None;
            app.last_health = std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(5))
                .unwrap_or_else(std::time::Instant::now);
        }
        Err(e) => app.log(format!("launch failed: {e}")),
    }
}

pub fn kill_dc(app: &mut App) {
    if let Some(mut dc) = app.dc.take() {
        crate::process::kill(&mut dc);
        app.log("designcraft killed");
    }
    app.client = None;
    app.dc_status = DcStatus::Off;
}

pub fn build_translator(app: &mut App) -> Box<dyn crate::translate::Translator> {
    if app.llm_key.trim().is_empty() {
        app.log("no API key — using offline mock translator (expands text ~20%)");
        Box::new(crate::translate::MockTranslator)
    } else {
        app.log(format!("LLM: {} @ {}", app.llm_model, app.llm_base));
        Box::new(crate::translate::LlmTranslator {
            base_url: app.llm_base.clone(),
            api_key: app.llm_key.clone(),
            model: app.llm_model.clone(),
        })
    }
}

pub fn lang_code(lang: crate::metadata_form::Lang) -> &'static str {
    match lang {
        crate::metadata_form::Lang::Id => "id",
        crate::metadata_form::Lang::En => "en",
        crate::metadata_form::Lang::IdEn => "iden",
    }
}

pub fn import_glossary(app: &mut App) {
    let Some(path) = pick_file("Glossary CSV", &["csv"]) else { return };
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let imported = crate::glossary::from_csv(&text);
            let count = imported.len();
            app.project.glossary.extend(imported);
            app.log(format!("imported {count} glossary entries from {}", path.display()));
        }
        Err(e) => app.log(format!("glossary import failed: {e}")),
    }
}

pub fn export_glossary(app: &mut App) {
    let file = rfd::FileDialog::new()
        .add_filter("glossary CSV", &["csv"])
        .set_file_name("glossary.csv")
        .save_file();
    let Some(path) = file else { return };
    match std::fs::write(&path, crate::glossary::to_csv(&app.project.glossary)) {
        Ok(()) => app.log(format!("glossary exported to {}", path.display())),
        Err(e) => app.log(format!("glossary export failed: {e}")),
    }
}

struct OwnedInput {
    name: String,
    source: String,
    budget: Option<u64>,
}

pub fn translate(app: &mut App) {
    let translator = build_translator(app);
    let lang = app.project.metadata.lang;
    let glossary = app.project.glossary.clone();

    let slot_inputs: Vec<OwnedInput> = app
        .project
        .slots
        .iter()
        .filter(|s| !s.source.is_empty())
        .map(|s| OwnedInput { name: s.name.clone(), source: s.source.clone(), budget: s.budget })
        .collect();
    let table_pages: Vec<(String, String)> = app
        .project
        .metadata
        .pages
        .iter()
        .filter(|p| p.kind == crate::metadata_form::PageKind::Table)
        .filter_map(|p| {
            app.project
                .page_source(p)
                .map(|(name, bytes)| (name.clone(), String::from_utf8_lossy(bytes).into_owned()))
        })
        .collect();

    if slot_inputs.is_empty() && table_pages.is_empty() {
        app.log("nothing to translate — add slot source text or table pages with sources");
        return;
    }
    app.log(format!(
        "translating {} slots and {} table pages → {}",
        slot_inputs.len(),
        table_pages.len(),
        lang.label()
    ));

    let refs: Vec<crate::translate::SlotInput> = slot_inputs
        .iter()
        .map(|i| crate::translate::SlotInput { name: &i.name, source: &i.source, budget: i.budget })
        .collect();
    let outcomes = crate::translate::translate_slots(translator.as_ref(), lang, &glossary, &refs);
    for outcome in outcomes {
        match &outcome.status {
            crate::translate::SlotStatus::Ok => {
                app.log(format!("{}: ok ({} chars)", outcome.name, outcome.text.chars().count()))
            }
            crate::translate::SlotStatus::OverBudget => app.log(format!(
                "{}: OVER BUDGET ({} chars) — flagged for human edit",
                outcome.name,
                outcome.text.chars().count()
            )),
            crate::translate::SlotStatus::Flagged(reason) => {
                app.log(format!("{}: FLAGGED — {reason}", outcome.name))
            }
        }
        if let Some(slot) = app.project.slots.iter_mut().find(|s| s.name == outcome.name) {
            slot.text = outcome.text;
        }
    }

    for (name, csv_text) in &table_pages {
        match crate::translate::translate_table(translator.as_ref(), lang, &glossary, csv_text) {
            Ok(csv) => {
                let stem = name.rsplit_once('.').map(|(a, _)| a).unwrap_or(name);
                let ext = name
                    .rsplit_once('.')
                    .and_then(|(_, e)| if e.is_empty() { None } else { Some(e.to_owned()) })
                    .unwrap_or_else(|| "csv".into());
                let entry = format!("{stem}.{code}.{ext}", code = lang_code(lang));
                app.project.add_content(entry, csv.into_bytes());
                app.log(format!("table translated: {name}"));
            }
            Err(e) => app.log(format!("table {name} failed: {e}")),
        }
    }

    if app.client.is_some() {
        overset_pass(app, translator.as_ref(), lang, &glossary);
    }
}

pub fn overset_pass(
    app: &mut App,
    translator: &dyn crate::translate::Translator,
    lang: crate::metadata_form::Lang,
    glossary: &crate::glossary::Glossary,
) {
    let texts: Vec<Slot> = app.project.slots.iter().filter(|s| !s.text.is_empty()).cloned().collect();
    if texts.is_empty() {
        return;
    }
    let _ = run_with_client(app, |client, app| {
        let filled = slots::fill_live(client, &texts)?;
        app.log(format!("filled {filled} slots"));
        Ok(())
    });

    let overset_names: Vec<String> = run_with_client::<Vec<String>>(app, |client, app| {
        let doc = client.inspect()?;
        let mut overset_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
        if let Some(stories) = doc.get("stories").and_then(Value::as_array) {
            for story in stories {
                if story.get("overset").and_then(Value::as_bool).unwrap_or(false)
                    && let Some(id) = story.get("id").and_then(Value::as_u64) {
                        overset_ids.insert(id);
                    }
            }
        }
        Ok(app
            .project
            .slots
            .iter()
            .filter(|s| overset_ids.contains(&s.story) && !s.text.is_empty())
            .map(|s| s.name.clone())
            .collect::<Vec<String>>())
    })
    .unwrap_or_default();

    if overset_names.is_empty() {
        app.log("engine: 0 oversets");
        return;
    }
    app.log(format!("engine oversets: {} — retrying once, tighter", overset_names.join(", ")));

    let retries: Vec<OwnedInput> = app
        .project
        .slots
        .iter()
        .filter(|s| overset_names.contains(&s.name))
        .map(|s| {
            let current = s.text.chars().count() as u64;
            let budget = Some(s.budget.unwrap_or(current).min(current * 4 / 5).max(10));
            OwnedInput { name: s.name.clone(), source: s.source.clone(), budget }
        })
        .collect();
    let refs: Vec<crate::translate::SlotInput> = retries
        .iter()
        .map(|i| crate::translate::SlotInput { name: &i.name, source: &i.source, budget: i.budget })
        .collect();
    let outcomes = crate::translate::translate_slots(translator, lang, glossary, &refs);
    for outcome in outcomes {
        if let Some(slot) = app.project.slots.iter_mut().find(|s| s.name == outcome.name) {
            slot.text = outcome.text;
        }
    }
    let texts: Vec<Slot> = app.project.slots.iter().filter(|s| !s.text.is_empty()).cloned().collect();
    let still: Vec<String> = run_with_client::<Vec<String>>(app, |client, app| {
        let _ = slots::fill_live(client, &texts)?;
        let doc = client.inspect()?;
        let mut overset_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
        if let Some(stories) = doc.get("stories").and_then(Value::as_array) {
            for story in stories {
                if story.get("overset").and_then(Value::as_bool).unwrap_or(false)
                    && let Some(id) = story.get("id").and_then(Value::as_u64) {
                        overset_ids.insert(id);
                    }
            }
        }
        Ok(app
            .project
            .slots
            .iter()
            .filter(|s| overset_ids.contains(&s.story) && !s.text.is_empty())
            .map(|s| s.name.clone())
            .collect::<Vec<String>>())
    })
    .unwrap_or_default();

    if still.is_empty() {
        app.log("engine: 0 oversets after retry");
    } else {
        app.log(format!("still overset after retry — FLAG for human edit: {}", still.join(", ")));
    }
}

/// Writes a converted content file into the source folder, preserving the `md/`/`csv/` layout.
pub fn write_converted(app: &mut App, base: &std::path::Path, name: &str, bytes: &[u8]) {
    let rel = PathBuf::from(name);
    if !(rel.starts_with("md") || rel.starts_with("csv")) {
        return;
    }
    let target = base.join(&rel);
    if let Some(dir) = target.parent()
        && let Err(e) = std::fs::create_dir_all(dir) {
        app.log(format!("cannot create {}: {e}", dir.display()));
        return;
    }
    match std::fs::write(&target, bytes) {
        Ok(()) => app.log(format!("saved {}", target.display())),
        Err(e) => app.log(format!("cannot write {}: {e}", target.display())),
    }
}

/// Resolves a narrative link (an explicit `.rtf`/`.docx` source, or a virtual
/// `md/{stem}.md` source) to a real file: the exact relative path next to the
/// workbook first, then the bare filename and its stem in the workbook's folder
/// and the common narrative subfolders (`rtf/`, `docx/`).
fn resolve_linked(base: &std::path::Path, src: &str) -> Option<std::path::PathBuf> {
    let s = src.trim().trim_start_matches("./").trim();
    if s.is_empty() {
        return None;
    }
    if let Some(p) = base.join(s).canonicalize().ok() {
        return Some(p);
    }
    let file_name = std::path::Path::new(s).file_name()?.to_str()?.to_string();
    let stem = std::path::Path::new(&file_name).file_stem()?.to_str()?.to_string();
    for dir in ["", "rtf", "docx"] {
        let at = |name: &str| {
            if dir.is_empty() {
                base.join(name)
            } else {
                base.join(dir).join(name)
            }
        };
        if let Some(p) = at(&file_name).canonicalize().ok() {
            return Some(p);
        }
        for ext in ["rtf", "docx"] {
            if let Some(p) = at(&format!("{stem}.{ext}")).canonicalize().ok() {
                return Some(p);
            }
        }
    }
    None
}

pub fn ingest_linked_narratives(app: &mut App, base: &std::path::Path, entries: &mut [crate::metadata_form::PageEntry]) {
    for entry in entries {
        let Some(src) = entry.source.clone() else { continue };
        let low = src.to_lowercase();
        if !(low.ends_with(".docx") || low.ends_with(".rtf") || low.ends_with(".md")) {
            continue;
        }

        let Some(linked) = resolve_linked(base, &src) else { continue };

        match ingest::ingest(&linked) {
            Ok(files) => {
                for f in files {
                    app.log(format!("linked {} -> {} ({} B)", linked.display(), f.name, f.bytes.len()));
                    write_converted(app, base, &f.name, &f.bytes);
                    app.project.add_content(f.name.clone(), f.bytes);
                    entry.source = Some(f.name);
                }
            }
            Err(e) => app.log(format!("linked import failed for {}: {e}", linked.display())),
        }
    }
}

/// Converts one source workbook/document with the active profile and stores the
/// result twice: written next to the source (`csv/…`, `md/…`) and embedded as
/// project content. Shared by every import entry point so the card sources
/// always resolve to real files.
pub fn convert_and_store(app: &mut App, path: &std::path::Path, base: &std::path::Path) {
    match ingest::ingest_with_profile(path, &app.selected_profile) {
        Ok(files) => {
            for f in files {
                app.log(format!("converted {} -> {} ({} B)", path.display(), f.name, f.bytes.len()));
                write_converted(app, base, &f.name, &f.bytes);
                app.project.add_content(f.name, f.bytes);
            }
        }
        Err(e) => app.log(format!("import failed: {e}")),
    }
}

pub fn import_sources(app: &mut App) {
    let Some(path) = pick_file("Sources", &["docx", "rtf", "xlsx", "xlsm"]) else { return };
    app.center_tab = crate::main_ui::CenterTab::Content;
    let base = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let mut parsed_cover: Option<crate::metadata_form::ExcelCoverData> = None;
    if ext == "xlsx" || ext == "xlsm" {
        if let Ok(Some(cov)) = ingest::parse_excel_cover(&path) {
            let title = cov.title1.clone();
            app.project.metadata.apply_excel_cover(&cov);
            app.log(format!("auto-extracted metadata from {} ('{}')", path.display(), title));
            parsed_cover = Some(cov);
        }
        if let Ok(profile_id) = ingest::detect_workbook_profile(&path) {
            app.set_publication_profile(profile_id);
            app.log(format!("auto-detected profile: {profile_id}"));
        }
        if let Ok(mut entries) = ingest::extract_workbook_content_entries(&path, &app.selected_profile)
            && !entries.is_empty()
        {
            // Auto-ingest linked DOCX/RTF narratives referenced by sheets (file/berkas cells),
            // resolved next to the workbook, and rebind the entry to the converted .md key.
            ingest_linked_narratives(app, &base, &mut entries);
            let count = entries.len();
            // Fill Cover cards with the parsed COV sheet key/values.
            if let Some(cov) = &parsed_cover {
                for entry in &mut entries {
                    if entry.kind == crate::metadata_form::PageKind::Cover && entry.cover.is_none() {
                        entry.cover = Some(cov.clone());
                    }
                }
            }
            app.project.metadata.pages = entries;
            app.log(format!("extracted {count} publication content items from workbook sheets"));
        }
    }
    // Full workbook conversion: sheet CSVs (and any linked narratives) are written
    // next to the workbook and embedded as project content, so the extracted card
    // sources (`csv/…`, `md/…`) resolve to real files.
    convert_and_store(app, &path, &base);
}

pub fn import_excel_metadata_from_path(app: &mut App, path: &std::path::Path) {
    app.source_excel_path = Some(path.to_path_buf());
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        app.project.name = stem.to_string();
    }
    let base = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let mut parsed_cover: Option<crate::metadata_form::ExcelCoverData> = None;
    match ingest::parse_excel_cover(path) {
        Ok(Some(cov)) => {
            let title = cov.title1.clone();
            app.project.metadata.apply_excel_cover(&cov);
            app.log(format!("metadata imported from {} ('{}')", path.display(), title));
            parsed_cover = Some(cov);
        }
        Ok(None) => {
            app.log(format!("no COVER sheet found in {}", path.display()));
        }
        Err(e) => {
            app.log(format!("failed to read Excel metadata: {e}"));
        }
    }
    if let Ok(profile_id) = ingest::detect_workbook_profile(path) {
        app.set_publication_profile(profile_id);
        app.log(format!("auto-detected profile: {profile_id}"));
    }
    if let Ok(mut entries) = ingest::extract_workbook_content_entries(path, &app.selected_profile)
        && !entries.is_empty()
    {
        ingest_linked_narratives(app, &base, &mut entries);
        if let Some(cov) = &parsed_cover {
            for entry in &mut entries {
                if entry.kind == crate::metadata_form::PageKind::Cover && entry.cover.is_none() {
                    entry.cover = Some(cov.clone());
                }
            }
        }
        let count = entries.len();
        app.project.metadata.pages = entries;
        app.log(format!("extracted {count} publication content items from workbook sheets"));
    }
    // Full workbook conversion — same as "Import Sources": the sheet CSVs (and any
    // linked DOCX/RTF narratives) are written next to the workbook (`csv/`, `md/`)
    // and embedded as project content, so every card's `csv/…`/`md/…` source
    // resolves to a real file on disk.
    convert_and_store(app, path, &base);
    app.view = crate::main_ui::AppView::Studio;
    app.center_tab = crate::main_ui::CenterTab::Content;
}

pub fn import_excel_metadata(app: &mut App) {
    let Some(path) = pick_file("Excel Workbook", &["xlsx", "xlsm"]) else { return };
    import_excel_metadata_from_path(app, &path);
}

pub fn load_template(app: &mut App) {
    let Some(path) = pick_file("DesignCraft template", &["designcraft"]) else { return };
    match std::fs::read(&path) {
        Ok(bytes) => {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("template")
                .to_owned();
            app.project.template = Some(bytes);
            app.project.name = name;
            app.log(format!("template loaded from {}", path.display()));
        }
        Err(e) => app.log(format!("template read failed: {e}")),
    }
}

pub fn open_project_path(app: &mut App, path: &std::path::Path) -> Result<()> {
    let p = crate::project::Project::open(path)?;
    app.log(format!(
        "project opened: {} ({} content files, {} slots)",
        p.name,
        p.content.len(),
        p.slots.len()
    ));
    // Materialize converted md/ + csv/ content next to the .pubgen container.
    let base = path.parent().map(|d| d.to_path_buf()).unwrap_or_default();
    for (name, bytes) in &p.content {
        write_converted(app, &base, name, bytes);
    }
    app.selected_profile = p.metadata.profile_id.clone();
    app.project_path = Some(path.to_path_buf());
    app.project = p;
    sync_project_csv_content(app);
    app.add_recent_project(path.to_path_buf());
    app.view = crate::main_ui::AppView::Studio;
    app.center_tab = crate::main_ui::CenterTab::Content;
    Ok(())
}

pub fn open_project(app: &mut App) {
    let Some(path) = pick_file("pubgen project", &["pubgen"]) else { return };
    if let Err(e) = open_project_path(app, &path) {
        app.log(format!("open failed: {e}"));
    }
}

pub fn save_project(app: &mut App) {
    let file = rfd::FileDialog::new()
        .add_filter("pubgen project", &["pubgen"])
        .set_file_name(format!("{}.pubgen", app.project.name))
        .save_file();
    let Some(path) = file else { return };
    match app.project.save(&path) {
        Ok(()) => {
            app.project_path = Some(path.clone());
            app.log(format!("saved {}", path.display()));
            app.add_recent_project(path);
        }
        Err(e) => app.log(format!("save failed: {e}")),
    }
}

pub fn pick_file(filter_name: &str, exts: &[&str]) -> Option<PathBuf> {
    rfd::FileDialog::new().add_filter(filter_name, exts).pick_file()
}

pub fn discover(app: &mut App) {
    if app.project.template.is_none() {
        app.log("note: no template in project — discover works against the document open in DesignCraft");
    }
    run_with_client(app, |client, app| {
        let mut found = slots::discover(client)?;
        let known = &mut app.project.slots;
        for slot in found.drain(..) {
            match known.iter_mut().find(|s| s.name == slot.name) {
                Some(existing) => {
                    existing.frame = slot.frame;
                    existing.story = slot.story;
                }
                None => known.push(slot),
            }
        }
        known.sort_by(|a, b| a.name.cmp(&b.name));
        app.log(format!("discovered {} slot:* frames", app.project.slots.len()));
        Ok(())
    });
}

pub fn fill_live(app: &mut App) {
    let texts: Vec<Slot> = app
        .project
        .slots
        .iter()
        .filter(|s| !s.text.is_empty())
        .cloned()
        .collect();
    if texts.is_empty() {
        app.log("no slot text to fill");
        return;
    }
    run_with_client(app, |client, app| {
        let filled = slots::fill_live(client, &texts)?;
        app.log(format!("filled {filled} slots live"));
        Ok(())
    });
}

pub fn resolve_asset_path(src: &str, base_dirs: &[&std::path::Path]) -> String {
    let raw = src.trim();
    if raw.is_empty() {
        return String::new();
    }
    let p = std::path::Path::new(raw);
    if p.is_absolute() && p.exists() {
        return p.to_string_lossy().replace('\\', "/");
    }

    for base in base_dirs {
        let cand = base.join(p);
        if cand.exists() {
            if let Ok(canon) = std::fs::canonicalize(&cand) {
                let s = canon.to_string_lossy();
                let clean = s.strip_prefix(r"\\?\").unwrap_or(&s);
                return clean.replace('\\', "/");
            }
            return cand.to_string_lossy().replace('\\', "/");
        }
    }

    // Also check relative to working directory and standard asset search paths
    for fallback in &["", "assets", "../assets", ".."] {
        let cand = if fallback.is_empty() {
            std::path::PathBuf::from(p)
        } else {
            std::path::Path::new(fallback).join(p)
        };
        if cand.exists() {
            if let Ok(canon) = std::fs::canonicalize(&cand) {
                let s = canon.to_string_lossy();
                let clean = s.strip_prefix(r"\\?\").unwrap_or(&s);
                return clean.replace('\\', "/");
            }
            return cand.to_string_lossy().replace('\\', "/");
        }
    }

    // Fallback: check embedded assets (e.g. templates or maps compiled into pos.exe)
    if let Some(extracted) = crate::embedded_assets::ensure_asset_on_disk(raw) {
        return extracted.to_string_lossy().replace('\\', "/");
    }

    raw.replace('\\', "/")
}

pub struct BatchScriptBuilder {
    script: String,
    step_count: usize,
}

impl BatchScriptBuilder {
    pub fn new() -> Self {
        Self {
            script: String::new(),
            step_count: 0,
        }
    }

    pub fn push(&mut self, command: &str, params: serde_json::Value) -> usize {
        let step_idx = self.step_count;
        let cmd = serde_json::json!({
            "command": command,
            "params": params
        });
        self.script.push_str(&cmd.to_string());
        self.script.push('\n');
        self.step_count += 1;
        step_idx
    }

    pub fn finish(self) -> String {
        self.script
    }
}

pub fn format_csv_to_tab_table(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let first_line = trimmed.lines().next().unwrap_or("");
    let comma_count = first_line.matches(',').count();
    let semi_count = first_line.matches(';').count();
    let tab_count = first_line.matches('\t').count();
    let delim = if tab_count > comma_count && tab_count > semi_count {
        b'\t'
    } else if semi_count > comma_count {
        b';'
    } else {
        b','
    };

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .flexible(true)
        .has_headers(false)
        .from_reader(trimmed.as_bytes());

    let mut rows = Vec::new();
    for rec in rdr.records().flatten() {
        let cells: Vec<String> = rec.iter().map(|s| s.trim().to_string()).collect();
        rows.push(cells.join("\t"));
    }
    if rows.is_empty() {
        trimmed.replace(delim as char, "\t")
    } else {
        rows.join("\n")
    }
}

fn get_entry_content(
    entry: &crate::metadata_form::PageEntry,
    project: Option<&crate::project::Project>,
    base_dirs: &[&std::path::Path],
) -> Option<Vec<u8>> {
    if let Some((_, bytes)) = project.and_then(|p| p.page_source(entry)) {
        return Some(bytes.clone());
    }
    if let Some(ref src) = entry.source {
        let p = resolve_asset_path(src.trim(), base_dirs);
        if let Ok(bytes) = std::fs::read(&p) {
            return Some(bytes);
        }
    }
    None
}

/// Picks the canonical cover artboard for a DDA cover by the stored style name;
/// unknown styles fall back to `007_modern_horizon` (the reference default).
fn cover_template_by_style(style: Option<&str>) -> &'static str {
    let s = style.unwrap_or("").to_lowercase();
    let idx = if s.contains("wave") || s.contains("kurva") {
        0
    } else if s.contains("chevron") {
        3
    } else if s.contains("medallion") {
        1
    } else if s.contains("origami") {
        2
    } else if s.contains("honeycomb") || s.contains("hexagon") {
        4
    } else if s.contains("headline") || s.contains("korporat") {
        5
    } else if s.contains("diagonal") || s.contains("arrow") {
        6
    } else if s.contains("report") || s.contains("bisnis") {
        8
    } else if s.contains("angle") || s.contains("urban") {
        9
    } else if s.contains("tri") || s.contains("band") {
        10
    } else if s.contains("prism") || s.contains("diamond") {
        11
    } else if s.contains("quarter") || s.contains("stack") {
        12
    } else {
        7
    };
    [
        "assets/templates/covers/000_wave_curves.designcraft",
        "assets/templates/covers/001_geometric_medallion.designcraft",
        "assets/templates/covers/002_folded_origami.designcraft",
        "assets/templates/covers/003_chevron_medallion.designcraft",
        "assets/templates/covers/004_honeycomb_hexagon.designcraft",
        "assets/templates/covers/005_corporate_headline.designcraft",
        "assets/templates/covers/006_diagonal_arrow.designcraft",
        "assets/templates/covers/007_modern_horizon.designcraft",
        "assets/templates/covers/008_business_report.designcraft",
        "assets/templates/covers/009_urban_angle.designcraft",
        "assets/templates/covers/010_tri_color_bands.designcraft",
        "assets/templates/covers/011_prism_diamonds.designcraft",
        "assets/templates/covers/012_quarter_stack.designcraft",
    ][idx]
}

/// Resolves the cover palette (Primary / Accent / Dark) from a palette/theme name.
fn cover_palette(name: &str) -> (&'static str, &'static str, &'static str) {
    let n = name.to_lowercase();
    if n.contains("orange") || n.contains("brs") {
        ("#f68839", "#122647", "#7a3e00")
    } else if n.contains("navy") || n.contains("dark") {
        ("#0a2540", "#f5871f", "#050f1f")
    } else if n.contains("green") {
        ("#00a859", "#f5871f", "#05432a")
    } else {
        // BPS teal (reference default): Primary #0072bc, Accent #f5ab19, Dark #122647.
        ("#0072bc", "#f5ab19", "#122647")
    }
}

fn first_non_empty<'a>(a: &'a str, b: &'a str, fallback: &'a str) -> &'a str {
    if !a.trim().is_empty() {
        a
    } else if !b.trim().is_empty() {
        b
    } else {
        fallback
    }
}

pub fn build_batch_script(
    metadata: &crate::metadata_form::Metadata,
    profile_id: &str,
    slots: &[&Slot],
    project: Option<&crate::project::Project>,
    base_dirs: &[&std::path::Path],
) -> String {
    let mut builder = BatchScriptBuilder::new();

    let profile = crate::profiles::ProfileManager::get_profile(profile_id);
    let is_brs = profile_id.trim().to_lowercase().contains("brs");
    let facing_pages = metadata.layout == crate::metadata_form::PageLayout::Double || profile.geometry.facing_pages || is_brs;

    let brs_tpl = resolve_asset_path("assets/templates/covers/brs/brs.designcraft", base_dirs);
    let use_brs_template = is_brs && std::path::Path::new(&brs_tpl).exists();

    // If this is a BRS profile with BRS narratives and template is available, use dedicated BRS generator
    let has_brs_narratives = metadata.pages.iter().any(|p| {
        let s = p.source.as_deref().unwrap_or("");
        let t = &p.title;
        s.contains("U1") || s.contains("U2") || t.contains("U1") || t.contains("U2")
    });

    if is_brs && use_brs_template && has_brs_narratives {
        return crate::brs::build_batch_script_brs(metadata, project, base_dirs);
    }

    // 0. Initialize blank document or open existing BRS template
    if use_brs_template {
        builder.push("file.open", serde_json::json!({
            "path": brs_tpl
        }));
        builder.push("layout.documentSetup", serde_json::json!({
            "facingPages": true,
            "margins": { "top": 57.26, "bottom": 56.69, "inside": 53.86, "outside": 42.52 }
        }));
    } else {
        builder.push("file.new", serde_json::json!({
            "preset": "A4",
            "facingPages": facing_pages,
            "pages": 1
        }));
        if facing_pages {
            builder.push("layout.documentSetup", serde_json::json!({
                "facingPages": true,
                "margins": { "top": 18.0, "bottom": 18.0, "inside": 20.0, "outside": 20.0 }
            }));
        }
    }

    // Geometry margins for content frames (A4 portrait: 595.28 x 841.89 pt)
    let margin_y0 = 57.3;
    let margin_y1 = 785.2;
    let gap = 12.0;

    // 1. Text variables for document title and release info
    if !metadata.title.trim().is_empty() {
        builder.push("variables.define", serde_json::json!({
            "name": "Title",
            "type": "custom",
            "text": metadata.title.trim()
        }));
    }
    if !metadata.release_no.trim().is_empty() {
        builder.push("variables.define", serde_json::json!({
            "name": "Release No",
            "type": "custom",
            "text": metadata.release_no.trim()
        }));
    }

    let cover_entry = metadata.pages.iter().find(|p| p.kind == crate::metadata_form::PageKind::Cover);
    let cov = cover_entry.and_then(|p| p.cover.as_ref());

    // If using the BRS template, populate Front Cover, Point Page, and Back Cover in the template
    if use_brs_template {
        let agency = if let Some(c) = cov && !c.pub_office.trim().is_empty() {
            c.pub_office.trim().to_string()
        } else if let Some(c) = cov && !c.agency.trim().is_empty() {
            c.agency.trim().to_string()
        } else if !metadata.publisher.trim().is_empty() {
            metadata.publisher.trim().to_string()
        } else {
            "BADAN PUSAT STATISTIK\nPROVINSI JAWA BARAT".to_string()
        };
        builder.push("story.setText", serde_json::json!({ "story": 321, "text": agency }));

        let rel_no = cov
            .map(|c| c.release_no.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(metadata.release_no.as_str())
            .trim();
        let rel_date = cov
            .map(|c| c.release_date.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(metadata.release_date.as_str())
            .trim();

        let clean_no = if let Some(stripped) = rel_no.strip_prefix("No.") {
            stripped.trim()
        } else if let Some(stripped) = rel_no.strip_prefix("No ") {
            stripped.trim()
        } else {
            rel_no
        };

        let release_info = if !clean_no.is_empty() {
            if !rel_date.is_empty() {
                format!("No. {clean_no}, {rel_date}")
            } else {
                format!("No. {clean_no}")
            }
        } else {
            "No. 61/10/32/Th. XXVIII, 1 Oktober 2026".to_string()
        };
        builder.push("story.setText", serde_json::json!({ "story": 323, "text": release_info }));

        let point = if let Some(c) = cov && !c.point.trim().is_empty() {
            c.point.trim().to_string()
        } else if !metadata.description.trim().is_empty() {
            metadata.description.trim().to_string()
        } else {
            "Nilai Tukar Petani Jawa Barat September 2026 sebesar 120,54 atau naik 0,99 persen dibandingkan Agustus 2026 (2018 = 100).".to_string()
        };
        builder.push("story.setText", serde_json::json!({ "story": 868, "text": point }));

        let raw_title = if let Some(c) = cov && !c.title1.trim().is_empty() {
            c.title1.trim()
        } else if !metadata.title.trim().is_empty() {
            metadata.title.trim()
        } else {
            "Perkembangan Nilai Tukar Petani"
        };
        let month = cov
            .map(|c| c.release_month.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(metadata.release_month.as_str())
            .trim();
        let full_title = if !month.is_empty() && !raw_title.to_lowercase().contains(&month.to_lowercase()) {
            format!("{raw_title} {month}")
        } else {
            raw_title.to_string()
        };
        builder.push("story.setText", serde_json::json!({ "story": 870, "text": full_title }));

        if let Some(c) = cov && !c.image_hero.trim().is_empty() {
            let hero_path = resolve_asset_path(c.image_hero.trim(), base_dirs);
            if std::path::Path::new(&hero_path).exists() {
                builder.push("file.place", serde_json::json!({ "frame": 10, "path": hero_path }));
            }
        }

        // Point page text on Spread 1 (story 884)
        let point_items = crate::brs::extract_brs_point_items(metadata, project, base_dirs);
        let points_body = if !point_items.is_empty() {
            point_items.join("\n")
        } else if let Some(c) = cov && !c.point.trim().is_empty() {
            c.point.trim().to_string()
        } else {
            "Nilai Tukar Petani (NTP) adalah perbandingan indeks harga yang diterima petani (It) terhadap indeks harga yang dibayar petani (Ib).\n\nNTP merupakan salah satu indikator untuk melihat tingkat kemampuan/daya beli petani di perdesaan.".to_string()
        };
        builder.push("story.setText", serde_json::json!({ "story": 884, "text": points_body }));

        // Back cover text (stories: 893, 895, 901, 903)
        let pic_name = cov.map(|c| c.pic_name.as_str()).filter(|s| !s.trim().is_empty()).unwrap_or("Yana Hendriana, S.ST");
        builder.push("story.setText", serde_json::json!({ "story": 893, "text": pic_name }));

        let contact_details = if let Some(c) = cov {
            let mut lines = Vec::new();
            if !c.position.trim().is_empty() { lines.push(c.position.trim().to_string()); }
            if !c.office.trim().is_empty() { lines.push(c.office.trim().to_string()); }
            if !c.phone.trim().is_empty() { lines.push(format!("Telp: {}", c.phone.trim())); }
            if !c.email.trim().is_empty() { lines.push(format!("E-mail: {}", c.email.trim())); }
            if lines.is_empty() {
                "Ketua Tim Statistik Harga\nBPS Provinsi Jawa Barat\nTelp: (022) 7272595\nE-mail: yanah@bps.go.id".to_string()
            } else {
                lines.join("\n")
            }
        } else {
            "Ketua Tim Statistik Harga\nBPS Provinsi Jawa Barat\nTelp: (022) 7272595\nE-mail: yanah@bps.go.id".to_string()
        };
        builder.push("story.setText", serde_json::json!({ "story": 895, "text": contact_details }));

        let agency_back = "BADAN PUSAT STATISTIK";
        builder.push("story.setText", serde_json::json!({ "story": 901, "text": agency_back }));

        let office_address = cov.map(|c| c.office_address.as_str()).filter(|s| !s.trim().is_empty())
            .unwrap_or("Jl. dr. Sutomo No. 6-8 Jakarta 10710\nTelp : (021) 3841195, 3842508, 3810291-4, Fax : (021) 3857046\nHomepage : http://www.bps.go.id   E-mail : bpshq@bps.go.id");
        builder.push("story.setText", serde_json::json!({ "story": 903, "text": office_address }));
    }

    // Coordinates in facing spreads:
    // Page 0 (Page 1) is Spread 0 (Cover, right page): x in [42.5, 541.4]
    // For page p >= 1:
    // spread = (p + 1) / 2
    // if p % 2 == 1 -> Left page of spread: x in [42.5, 541.4]
    // if p % 2 == 0 -> Right page of spread: x in [649.1, 1148.0]
    let page_spread_coords = |p: usize| -> (usize, f64, f64) {
        if facing_pages {
            if p == 0 {
                (0, 42.5, 541.4)
            } else {
                let sp = (p + 1) / 2;
                let is_right = p % 2 == 0;
                if is_right {
                    (sp, 649.1, 1148.0)
                } else {
                    (sp, 42.5, 541.4)
                }
            }
        } else {
            (p, 42.5, 541.4)
        }
    };

    // Body entries: exclude Cover and exclude Point summary (which is already on Inside Cover Page 2)
    let body_entries: Vec<(usize, &crate::metadata_form::PageEntry)> = metadata
        .pages
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            if p.kind == crate::metadata_form::PageKind::Cover {
                return false;
            }
            if is_brs {
                let t = p.title.trim().to_lowercase();
                if t.contains("ringkasan poin") || t.starts_with("point") || t.starts_with("poin") {
                    return false;
                }
            }
            true
        })
        .collect();

    // In BRS template, pages 1 and 2 are Front Cover and Point page, so content begins at page 3 (index 2).
        let cover_pages = if profile.geometry.facing_pages || metadata.layout == crate::metadata_form::PageLayout::Double { 2 } else { 1 };
    let page_plan = crate::toc::simulate_pagination(&metadata.pages, cover_pages);
    let bilingual = metadata.is_bilingual();
    let mut toc_cards: Vec<(usize, String, serde_json::Value, crate::metadata_form::TocData)> = Vec::new();

    let mut cur_page_idx: usize = if use_brs_template {
        // Insert first body page before the back cover (which is initially at index 2)
        builder.push("layout.pages.insert", serde_json::json!({
            "count": 1,
            "before": 2
        }));
        2
    } else {
        0
    };
    let mut cur_y: f64 = margin_y0;
    let mut items_on_current_page: usize = 0;

    for &(idx, entry) in &body_entries {
        let item_num = idx + 1;

        let est_h = match entry.kind {
            crate::metadata_form::PageKind::Table => {
                let row_count = if let Some((_, bytes)) = project.and_then(|p| p.page_source(entry)) {
                    bytes.split(|&b| b == b'\n').filter(|l| !l.is_empty()).count()
                } else {
                    12
                };
                (row_count as f64 * 18.0 + 50.0).clamp(100.0, 720.0)
            }
            crate::metadata_form::PageKind::Chart => 260.0,
            crate::metadata_form::PageKind::Paragraph => {
                if let Some((_, bytes)) = project.and_then(|p| p.page_source(entry)) {
                    let char_count = bytes.len();
                    let lines = (char_count as f64 / 80.0).ceil();
                    (lines * 14.0 + 30.0).clamp(80.0, 720.0)
                } else {
                    140.0
                }
            }
            crate::metadata_form::PageKind::Image => 700.0,
            _ => 150.0,
        };

        let is_g_sheet = match entry.kind {
            crate::metadata_form::PageKind::Image => true,
            _ => {
                let title_low = entry.title.to_lowercase();
                title_low.contains("g1")
                    || title_low.contains("infografis")
                    || entry.source.as_ref().is_some_and(|s| s.to_lowercase().ends_with(".png") || s.to_lowercase().ends_with(".jpg"))
            }
        };

        let advance_page = if items_on_current_page == 0 {
            false
        } else if entry.scope == crate::metadata_form::PageItemScope::NewPage {
            true
        } else if is_g_sheet {
            true
        } else if is_brs && entry.kind == crate::metadata_form::PageKind::Table {
            // In BRS publications, tables start on a fresh page
            true
        } else {
            cur_y + est_h > margin_y1 || (margin_y1 - cur_y < 120.0)
        };

        if advance_page {
            if use_brs_template {
                builder.push("layout.pages.insert", serde_json::json!({
                    "count": 1,
                    "after": cur_page_idx
                }));
            } else {
                builder.push("layout.pages.insert", serde_json::json!({
                    "count": 1
                }));
            }
            cur_page_idx += 1;
            cur_y = margin_y0;
            items_on_current_page = 0;
        }

        let (cur_spread, px0, px1) = page_spread_coords(cur_page_idx);

        match entry.kind {
            crate::metadata_form::PageKind::Table => {
                let title_text = entry.title.trim();
                let title_step = builder.push("frame.create", serde_json::json!({
                    "spread": cur_spread,
                    "rect": [px0, cur_y, px1, cur_y + 20.0],
                    "content": "text",
                    "text": title_text
                }));
                builder.push("object.label", serde_json::json!({
                    "ids": [format!("${title_step}.id")],
                    "label": format!("#slot:text:table_title_{item_num}")
                }));
                if !title_text.is_empty() {
                    builder.push("text.select", serde_json::json!({
                        "story": format!("${title_step}.story"),
                        "anchor": 0,
                        "focus": title_text.len()
                    }));
                    builder.push("type.char", serde_json::json!({
                        "fontFamily": "Lato",
                        "fontStyle": "Bold",
                        "size": 10.5
                    }));
                }

                let (table_text, row_count) = if let Some(bytes) = get_entry_content(entry, project, base_dirs) {
                    let raw = String::from_utf8_lossy(&bytes);
                    let tab = format_csv_to_tab_table(&raw);
                    let cnt = tab.lines().count().max(1);
                    (tab, cnt)
                } else {
                    (String::new(), 12)
                };

                let table_h = (row_count as f64 * 18.0 + 30.0).min(margin_y1 - cur_y - 30.0).max(100.0);

                let table_step = builder.push("frame.create", serde_json::json!({
                    "spread": cur_spread,
                    "rect": [px0, cur_y + 24.0, px1, cur_y + 24.0 + table_h],
                    "content": "text",
                    "text": table_text
                }));
                builder.push("object.label", serde_json::json!({
                    "ids": [format!("${table_step}.id")],
                    "label": format!("#slot:table:{item_num}")
                }));

                if !table_text.trim().is_empty() {
                    let text_len = table_text.len();
                    builder.push("text.select", serde_json::json!({
                        "story": format!("${table_step}.story"),
                        "anchor": 0,
                        "focus": text_len
                    }));
                    builder.push("table.convertFromText", serde_json::json!({
                        "columnSeparator": "tab"
                    }));
                    builder.push("table.options", serde_json::json!({
                        "headerRows": 1
                    }));
                }

                let mut used_h = 24.0 + table_h;

                if let Some(ref src) = entry.source && !src.trim().is_empty() {
                    let src_text = format!("Sumber: {}", src.trim());
                    let src_step = builder.push("frame.create", serde_json::json!({
                        "spread": cur_spread,
                        "rect": [px0, cur_y + used_h + 4.0, px1, cur_y + used_h + 22.0],
                        "content": "text",
                        "text": &src_text
                    }));
                    builder.push("object.label", serde_json::json!({
                        "ids": [format!("${src_step}.id")],
                        "label": format!("#slot:text:table_source_{item_num}")
                    }));
                    builder.push("text.select", serde_json::json!({
                        "story": format!("${src_step}.story"),
                        "anchor": 0,
                        "focus": src_text.len()
                    }));
                    builder.push("type.char", serde_json::json!({
                        "fontFamily": "Lato",
                        "size": 8.0
                    }));
                    used_h += 26.0;
                }

                cur_y += used_h + gap;
                items_on_current_page += 1;
            }
            crate::metadata_form::PageKind::Chart => {
                let title_text = entry.title.trim();
                let title_step = builder.push("frame.create", serde_json::json!({
                    "spread": cur_spread,
                    "rect": [px0, cur_y, px1, cur_y + 20.0],
                    "content": "text",
                    "text": title_text
                }));
                builder.push("object.label", serde_json::json!({
                    "ids": [format!("${title_step}.id")],
                    "label": format!("#slot:text:chart_title_{item_num}")
                }));
                if !title_text.is_empty() {
                    builder.push("text.select", serde_json::json!({
                        "story": format!("${title_step}.story"),
                        "anchor": 0,
                        "focus": title_text.len()
                    }));
                    builder.push("type.char", serde_json::json!({
                        "fontFamily": "Lato",
                        "fontStyle": "Bold",
                        "size": 9.5
                    }));
                }

                let chart_h = if is_brs { 360.0 } else { 240.0 };
                let chart_step = builder.push("frame.create", serde_json::json!({
                    "spread": cur_spread,
                    "rect": [px0, cur_y + 24.0, px1, (cur_y + 24.0 + chart_h).min(margin_y1)],
                    "content": "graphic"
                }));
                builder.push("object.label", serde_json::json!({
                    "ids": [format!("${chart_step}.id")],
                    "label": format!("#slot:chart:{item_num}")
                }));

                if let Some(ref src) = entry.source && !src.trim().is_empty() {
                    let resolved = resolve_asset_path(src.trim(), base_dirs);
                    if std::path::Path::new(&resolved).exists() && (resolved.ends_with(".png") || resolved.ends_with(".jpg")) {
                        builder.push("file.place", serde_json::json!({
                            "frame": format!("${chart_step}.id"),
                            "path": resolved
                        }));
                        builder.push("object.fit", serde_json::json!({
                            "mode": "fitProportionally",
                            "ids": [format!("${chart_step}.id")]
                        }));
                        builder.push("object.fit", serde_json::json!({
                            "mode": "centerContent",
                            "ids": [format!("${chart_step}.id")]
                        }));
                    }
                }

                let mut used_h = 24.0 + chart_h;

                if let Some(ref src) = entry.source && !src.trim().is_empty() {
                    let src_text = format!("Sumber: {}", src.trim());
                    let src_step = builder.push("frame.create", serde_json::json!({
                        "spread": cur_spread,
                        "rect": [px0, cur_y + used_h + 4.0, px1, cur_y + used_h + 22.0],
                        "content": "text",
                        "text": &src_text
                    }));
                    builder.push("object.label", serde_json::json!({
                        "ids": [format!("${src_step}.id")],
                        "label": format!("#slot:text:chart_source_{item_num}")
                    }));
                    builder.push("text.select", serde_json::json!({
                        "story": format!("${src_step}.story"),
                        "anchor": 0,
                        "focus": src_text.len()
                    }));
                    builder.push("type.char", serde_json::json!({
                        "fontFamily": "Lato",
                        "size": 8.0
                    }));
                    used_h += 26.0;
                }

                cur_y += used_h + gap;
                items_on_current_page += 1;
            }
            crate::metadata_form::PageKind::Paragraph => {
                let has_title = !is_brs && (entry.has_title || !entry.title.trim().is_empty());
                let title_h = if has_title {
                    let title_step = builder.push("frame.create", serde_json::json!({
                        "spread": cur_spread,
                        "rect": [px0, cur_y, px1, cur_y + 20.0],
                        "content": "text",
                        "text": entry.title.trim()
                    }));
                    builder.push("object.label", serde_json::json!({
                        "ids": [format!("${title_step}.id")],
                        "label": format!("#slot:text:narrative_title_{item_num}")
                    }));
                    24.0
                } else {
                    0.0
                };

                let body_text = if let Some(bytes) = get_entry_content(entry, project, base_dirs) {
                    String::from_utf8_lossy(&bytes).into_owned()
                } else if !entry.title.trim().is_empty() {
                    entry.title.trim().to_string()
                } else {
                    "Ulasan...".to_string()
                };

                let char_count = body_text.len();
                let estimated_lines = (char_count as f64 / 82.0).ceil().max(1.0);
                let est_total_h = estimated_lines * 14.0 + 20.0;
                let avail_h = margin_y1 - cur_y - title_h;

                if is_brs && est_total_h > avail_h && avail_h >= 80.0 {
                    // Frame 1 fills the remaining height on this page
                    let f1_step = builder.push("frame.create", serde_json::json!({
                        "spread": cur_spread,
                        "rect": [px0, cur_y + title_h, px1, margin_y1],
                        "content": "text",
                        "text": &body_text
                    }));
                    builder.push("object.label", serde_json::json!({
                        "ids": [format!("${f1_step}.id")],
                        "label": format!("#slot:text:narrative_body_{item_num}")
                    }));
                    builder.push("text.select", serde_json::json!({
                        "story": format!("${f1_step}.story"),
                        "anchor": 0,
                        "focus": body_text.len()
                    }));
                    builder.push("type.char", serde_json::json!({
                        "fontFamily": "Lato",
                        "size": 12.0,
                        "leading": { "kind": "points", "value": 15.0 }
                    }));

                    // Insert next page and thread
                    if use_brs_template {
                        builder.push("layout.pages.insert", serde_json::json!({
                            "count": 1,
                            "after": cur_page_idx
                        }));
                    } else {
                        builder.push("layout.pages.insert", serde_json::json!({
                            "count": 1
                        }));
                    }
                    cur_page_idx += 1;
                    let (next_spread, npx0, npx1) = page_spread_coords(cur_page_idx);
                    let rem_h = (est_total_h - avail_h).clamp(60.0, margin_y1 - margin_y0);

                    let f2_step = builder.push("frame.create", serde_json::json!({
                        "spread": next_spread,
                        "rect": [npx0, margin_y0, npx1, margin_y0 + rem_h],
                        "content": "text"
                    }));
                    builder.push("object.label", serde_json::json!({
                        "ids": [format!("${f2_step}.id")],
                        "label": format!("#slot:text:narrative_body_{item_num}_cont")
                    }));
                    builder.push("text.thread", serde_json::json!({
                        "from": format!("${f1_step}.id"),
                        "to": format!("${f2_step}.id")
                    }));

                    cur_y = margin_y0 + rem_h + gap;
                    items_on_current_page = 1;
                } else {
                    let narrative_h = est_total_h.min(avail_h);
                    let body_step = builder.push("frame.create", serde_json::json!({
                        "spread": cur_spread,
                        "rect": [px0, cur_y + title_h, px1, cur_y + title_h + narrative_h],
                        "content": "text",
                        "text": &body_text
                    }));
                    builder.push("object.label", serde_json::json!({
                        "ids": [format!("${body_step}.id")],
                        "label": format!("#slot:text:narrative_body_{item_num}")
                    }));
                    builder.push("text.select", serde_json::json!({
                        "story": format!("${body_step}.story"),
                        "anchor": 0,
                        "focus": body_text.len()
                    }));
                    builder.push("type.char", serde_json::json!({
                        "fontFamily": "Lato",
                        "size": 12.0,
                        "leading": { "kind": "points", "value": 15.0 }
                    }));
                    builder.push("object.textFrameOptions", serde_json::json!({
                        "ids": [format!("${body_step}.id")],
                        "autoSize": "heightOnly",
                        "autoSizeReference": "topLeft"
                    }));
                    builder.push("object.fit", serde_json::json!({
                        "mode": "fitFrameToContent",
                        "axis": "height",
                        "ids": [format!("${body_step}.id")]
                    }));

                    cur_y += title_h + narrative_h + gap;
                    items_on_current_page += 1;
                }
            }
            crate::metadata_form::PageKind::Image => {
                let img_h = if is_brs { margin_y1 - margin_y0 } else { 320.0 };
                let image_step = builder.push("frame.create", serde_json::json!({
                    "spread": cur_spread,
                    "rect": [px0, cur_y, px1, (cur_y + img_h).min(margin_y1)],
                    "content": "graphic"
                }));
                builder.push("object.label", serde_json::json!({
                    "ids": [format!("${image_step}.id")],
                    "label": format!("#slot:image:{item_num}")
                }));

                if let Some(ref src) = entry.source && !src.trim().is_empty() {
                    let resolved = resolve_asset_path(src.trim(), base_dirs);
                    builder.push("file.place", serde_json::json!({
                        "frame": format!("${image_step}.id"),
                        "path": resolved
                    }));
                    builder.push("object.fit", serde_json::json!({
                        "mode": "fitProportionally",
                        "ids": [format!("${image_step}.id")]
                    }));
                    builder.push("object.fit", serde_json::json!({
                        "mode": "centerContent",
                        "ids": [format!("${image_step}.id")]
                    }));
                }

                let mut used_h = img_h;

                if !is_brs && !entry.title.trim().is_empty() {
                    let cap_step = builder.push("frame.create", serde_json::json!({
                        "spread": cur_spread,
                        "rect": [px0, cur_y + used_h + 4.0, px1, cur_y + used_h + 22.0],
                        "content": "text",
                        "text": entry.title.trim()
                    }));
                    builder.push("object.label", serde_json::json!({
                        "ids": [format!("${cap_step}.id")],
                        "label": format!("#slot:text:image_caption_{item_num}")
                    }));
                    used_h += 26.0;
                }

                cur_y += used_h + gap;
                items_on_current_page += 1;
            }
            crate::metadata_form::PageKind::Cover => {
                if !is_brs {
                    let tpl_path = cover_template_by_style(entry.cover.as_ref().map(|c| c.style.as_str()));
                    let cov = entry.cover.clone().unwrap_or_default();
                    let mut text = serde_json::Map::new();
                    text.insert("title".into(), serde_json::json!(first_non_empty(&cov.title1, &metadata.title, "KABUPATEN DALAM ANGKA")));
                    let title2_val = first_non_empty(&cov.title2, &metadata.subtitle, "");
                    text.insert("title#2".into(), serde_json::json!(title2_val));
                    text.insert("year".into(), serde_json::json!(first_non_empty(&cov.year, &metadata.date, "2026")));
                    text.insert("volume".into(), serde_json::json!(first_non_empty(&cov.volume, &metadata.volume, "")));
                    text.insert("catalogue".into(), serde_json::json!(first_non_empty(&cov.catalogue, &metadata.catalogue, "")));
                    text.insert("code".into(), serde_json::json!(first_non_empty(&cov.code, &metadata.code, "")));
                    text.insert("agency".into(), serde_json::json!(first_non_empty(&cov.agency, &metadata.agency, "BADAN PUSAT STATISTIK")));
                    if bilingual {
                        let title_en = first_non_empty(&cov.title_en, &metadata.title_en, "");
                        let already = !title_en.is_empty()
                            && (title2_val.eq_ignore_ascii_case(title_en)
                                || title2_val.to_lowercase().contains(&title_en.to_lowercase()));
                        if !title_en.is_empty() && !already {
                            text.insert("title_en".into(), serde_json::json!(title_en));
                        }
                    }
                    let mut image = serde_json::Map::new();
                    if !cov.image_hero.trim().is_empty() {
                        image.insert("hero".into(), serde_json::json!(cov.image_hero.trim()));
                    }
                    if !cov.image_landscape.trim().is_empty() {
                        image.insert("landscape".into(), serde_json::json!(cov.image_landscape.trim()));
                    }
                    if !cov.logo.trim().is_empty() {
                        image.insert("logo".into(), serde_json::json!(cov.logo.trim()));
                    }
                    let (primary, accent, dark) = cover_palette(first_non_empty(&cov.palette, &cov.table_theme, "teal"));
                    let mut params = serde_json::Map::new();
                    params.insert("path".into(), serde_json::json!(tpl_path));
                    params.insert("before".into(), serde_json::json!(0));
                    params.insert("slots".into(), serde_json::json!({"text": text, "image": image}));
                    params.insert("palette".into(), serde_json::json!({"Primary": primary, "Accent": accent, "Dark": dark}));
                    if !metadata.language.trim().is_empty() {
                        params.insert("lang".into(), serde_json::json!(metadata.language.trim()));
                    }
                    builder.push("template.page.apply", serde_json::json!(params));
                }
            }
            crate::metadata_form::PageKind::Toc => {
                let t = entry.toc.clone().unwrap_or_default();
                let page = page_plan.page_of.get(idx).copied().unwrap_or(1);
                let span = crate::toc::toc_span_pages(&metadata.pages, entry).max(1);
                builder.push("layout.pages.insert", serde_json::json!({"count": span}));
                let title = match t.layout {
                    crate::metadata_form::TocLayout::TablePage => crate::toc::toc_header_line(t.style, bilingual),
                    _ => t.title_or_default().to_string(),
                };
                let entries = crate::toc::toc_entries_json(t.style, bilingual && t.pair_bilingual);
                builder.push("toc.generate", serde_json::json!({
                    "page": page,
                    "title": title,
                    "entries": entries,
                    "replace": false,
                    "autoflow": true
                }));
                toc_cards.push((page, title, entries, t));
            }
        }
    }

    // In facing pages BRS, ensure back cover is alone on the left of the final spread
    if use_brs_template && cur_page_idx % 2 == 1 {
        builder.push("layout.pages.insert", serde_json::json!({
            "count": 1,
            "after": cur_page_idx
        }));
        cur_page_idx += 1;
    }

    // Header / Footer application
    if is_brs {
        let (h1, h2) = crate::header_footer::resolve_brs_header_texts(
            &metadata.title,
            &metadata.release_no,
            &metadata.release_date,
            &metadata.release_month,
        );
        let footer_text = if h2.is_empty() { h1 } else { format!("{h1}\n{h2}") };
        let total_pages = cur_page_idx + 2;
        builder.push("layout.headerFooter.apply", serde_json::json!({
            "preset": "hf-brs",
            "sided": "double",
            "scope": {
                "range": [2, total_pages - 1]
            },
            "footer": {
                "single": footer_text,
                "left": footer_text,
                "right": footer_text
            }
        }));
    } else {
        let (f1, f2) = crate::header_footer::resolve_dda_footer_lines(
            &metadata.title,
            &metadata.subtitle,
            &metadata.title_en,
            &metadata.date,
            &metadata.release_date,
            &metadata.release_month,
        );
        let footer_text = if f2.is_empty() { f1 } else { format!("{f1}\n{f2}") };
        let total_pages = cur_page_idx + 1;
        builder.push("layout.headerFooter.apply", serde_json::json!({
            "preset": "hf-dda",
            "sided": if facing_pages { "double" } else { "single" },
            "scope": {
                "range": [2, total_pages]
            },
            "footer": {
                "single": footer_text,
                "left": footer_text,
                "right": footer_text
            }
        }));
    }

    // 3b. TOC cards: regenerate after all content exists so entry page numbers are final
    for (page, title, entries, t) in &toc_cards {
        let gen_index = builder.push("toc.generate", serde_json::json!({
            "page": *page,
            "title": title,
            "entries": entries,
            "replace": true,
            "autoflow": true
        }));
        if t.include_self {
            let doc_entries = crate::toc::estimate_toc_entries(&metadata.pages, t.style);
            let (at, self_line) = if doc_entries > 0 {
                (title.len() + 1, format!("{title}\t{page}\n"))
            } else {
                (title.len(), format!("\n{title}\t{page}"))
            };
            builder.push("story.replaceRange", serde_json::json!({
                "story": format!("${gen_index}.story"),
                "start": at,
                "end": at,
                "text": self_line
            }));
            builder.push("text.select", serde_json::json!({
                "story": format!("${gen_index}.story"),
                "anchor": at,
                "focus": at
            }));
            builder.push("style.paragraph.apply", serde_json::json!({
                "name": "TOC Level 1"
            }));
        }
    }

    // 5. Slot text replacements
    for slot in slots {
        builder.push("story.setText", serde_json::json!({
            "story": slot.story,
            "text": slot.text
        }));
    }

    builder.finish()
}

pub fn sync_project_csv_content(app: &mut App) {
    let mut base_dirs = Vec::new();
    if let Some(ref p) = app.project_path {
        if let Some(parent) = p.parent() {
            base_dirs.push(parent.to_path_buf());
        }
    }
    if let Some(ref e) = app.source_excel_path {
        if let Some(parent) = e.parent() {
            if !base_dirs.contains(&parent.to_path_buf()) {
                base_dirs.push(parent.to_path_buf());
            }
        }
    }
    // 1. Sync from disk csv/ directory if present
    for base in &base_dirs {
        let csv_dir = base.join("csv");
        if csv_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&csv_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
                    {
                        if let Some(file_name) = path.file_name() {
                            let key = format!("csv/{}", file_name.to_string_lossy());
                            if !app.project.content.iter().any(|(k, _)| k == &key) {
                                if let Ok(bytes) = std::fs::read(&path) {
                                    app.project.add_content(key, bytes);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Fallback: if project content has missing CSVs or no disk csv/ folder,
    // extract sheets from source excel path or any .xlsx in base_dirs!
    let has_tables = app.project.content.iter().any(|(k, _)| k.contains("T1") || k.contains("T2"));
    if !has_tables {
        let mut excel_candidates = Vec::new();
        if let Some(ref e) = app.source_excel_path {
            if e.exists() {
                excel_candidates.push(e.clone());
            }
        }
        for base in &base_dirs {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file()
                        && p.extension().and_then(|s| s.to_str()).is_some_and(|ext| {
                            ext.eq_ignore_ascii_case("xlsx") || ext.eq_ignore_ascii_case("xlsm")
                        })
                    {
                        if !excel_candidates.contains(&p) {
                            excel_candidates.push(p);
                        }
                    }
                }
            }
        }
        for excel_path in excel_candidates {
            if let Ok(files) = crate::ingest::ingest_with_profile(&excel_path, &app.selected_profile) {
                for f in files {
                    if f.name.starts_with("csv/") && !app.project.content.iter().any(|(k, _)| k == &f.name) {
                        app.project.add_content(f.name, f.bytes);
                    }
                }
            }
        }
    }
}

pub fn ensure_project_saved(app: &mut App) -> bool {
    sync_project_csv_content(app);
    if let Some(ref path) = app.project_path.clone() {
        if let Err(e) = app.project.save(path) {
            app.log(format!("warning: could not re-save workspace to {}: {e}", path.display()));
        }
        return true;
    }
    if let Some(ref excel_path) = app.source_excel_path.clone() {
        let pubgen_path = excel_path.with_extension("pubgen");
        match app.project.save(&pubgen_path) {
            Ok(()) => {
                app.project_path = Some(pubgen_path.clone());
                app.add_recent_project(pubgen_path.clone());
                app.log(format!("Auto-saved workspace to {}", pubgen_path.display()));
                return true;
            }
            Err(e) => {
                app.log(format!("Auto-save workspace failed: {e}"));
                return false;
            }
        }
    }
    app.log("Simpan workspace (*.pubgen) terlebih dahulu sebelum generate");
    false
}

pub fn batch_generate(app: &mut App, export_pdf: bool, standard: Option<&str>) {
    sync_project_csv_content(app);
    if !ensure_project_saved(app) {
        return;
    }

    let template = if let Some(tpl) = app.project.template.clone() {
        tpl
    } else {
        let tpl_candidate = if app.selected_profile.to_lowercase().contains("brs") {
            "assets/templates/covers/brs/brs.designcraft"
        } else {
            "assets/templates/covers/007_modern_horizon.designcraft"
        };
        if let Ok(bytes) = std::fs::read(tpl_candidate) {
            app.project.template = Some(bytes.clone());
            app.log(format!("auto-loaded template: {tpl_candidate}"));
            bytes
        } else {
            app.log("load a template into the project first");
            return;
        }
    };
    let Some(cli) = crate::process::resolve_cli() else {
        app.log("designcraft-cli not found — set PUBGEN_DESIGNCRAFT_CLI");
        return;
    };
    let slots_with_text: Vec<&Slot> = app.project.slots.iter().filter(|s| !s.text.is_empty()).collect();
    if slots_with_text.is_empty() && app.project.metadata.pages.is_empty() {
        app.log("nothing to generate — add pages or slots to project");
        return;
    }

    let target_dir = app.project_path.as_ref()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .or_else(|| app.source_excel_path.as_ref().and_then(|p| p.parent().map(|d| d.to_path_buf())))
        .unwrap_or_else(|| std::env::temp_dir().join("pubgen"));

    let _ = std::fs::create_dir_all(&target_dir);

    let doc_name = if !app.project.name.is_empty() && app.project.name != "Untitled" {
        app.project.name.clone()
    } else if let Some(ref p) = app.project_path {
        p.file_stem().and_then(|s| s.to_str()).unwrap_or("document").to_string()
    } else {
        "document".to_string()
    };

    let run_dir = std::env::temp_dir().join("pubgen");
    if let Err(e) = std::fs::create_dir_all(&run_dir) {
        app.log(format!("cannot create {}: {e}", run_dir.display()));
        return;
    }
    let template_path = run_dir.join("template.designcraft");
    let out_doc = target_dir.join(format!("{doc_name}.designcraft"));
    let out_pdf = target_dir.join(format!("{doc_name}.pdf"));
    if let Err(e) = std::fs::write(&template_path, &template) {
        app.log(format!("cannot write {}: {e}", template_path.display()));
        return;
    }

    let started = std::time::Instant::now();

    let project_dir = app.project_path.as_ref()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .or_else(|| app.source_excel_path.as_ref().and_then(|p| p.parent().map(|d| d.to_path_buf())));
    let cur_dir = std::env::current_dir().ok();

    let mut base_dirs: Vec<&std::path::Path> = Vec::new();
    if let Some(ref pd) = project_dir {
        base_dirs.push(pd.as_path());
    }
    if let Some(ref cd) = cur_dir {
        base_dirs.push(cd.as_path());
    }

    let script = build_batch_script(&app.project.metadata, &app.selected_profile, &slots_with_text, Some(&app.project), &base_dirs);

    let fill_path = run_dir.join("fill.jsonl");
    if let Err(e) = std::fs::write(&fill_path, script) {
        app.log(format!("cannot write {}: {e}", fill_path.display()));
        return;
    }

    let mut args: Vec<String> = vec![
        "script".into(),
        fill_path.display().to_string(),
        "--save".into(),
        out_doc.display().to_string(),
    ];
    if export_pdf {
        args.push("--export".into());
        args.push(out_pdf.display().to_string());
    }

    let profile = crate::profiles::ProfileManager::get_profile(&app.selected_profile);
    let std_label = standard.map(|s| format!(" ({s})")).unwrap_or_default();
    let action_desc = if export_pdf { "generate & publish" } else { "generate" };
    app.log(format!("batch {action_desc}: designcraft-cli ({} slots, profile: {}){}…", slots_with_text.len(), profile.name, std_label));
    match Command::new(&cli).args(&args).output() {
        Ok(out) if out.status.success() => {
            let is_brs = app.selected_profile.trim().to_lowercase().contains("brs");
            if is_brs {
                if let Err(e) = crate::brs::post_process_brs_document(&out_doc, &base_dirs) {
                    app.log(format!("post-process BRS document warning: {e}"));
                } else if export_pdf {
                    let pdf_args = vec![
                        "run".to_string(),
                        "--in".to_string(),
                        out_doc.display().to_string(),
                        "--export".to_string(),
                        out_pdf.display().to_string(),
                    ];
                    let _ = Command::new(&cli).args(&pdf_args).output();
                }
            }

            app.last_output_doc = Some(out_doc.clone());
            if export_pdf {
                app.last_output = Some(out_pdf.clone());
                app.log(format!(
                    "batch wrote {} and {} ({} ms)",
                    out_doc.display(),
                    out_pdf.display(),
                    started.elapsed().as_millis()
                ));
            } else {
                app.log(format!(
                    "batch wrote {} ({} ms)",
                    out_doc.display(),
                    started.elapsed().as_millis()
                ));
                open_in_designcraft(app);
            }

            if app.client.is_some() {
                run_qa_audit(app);
            }
        }
        Ok(out) => app.log(format!(
            "batch failed ({}): {} {}",
            out.status,
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        )),
        Err(e) => app.log(format!("batch spawn failed: {e}")),
    }
}

pub fn batch_fill(app: &mut App, standard: Option<&str>) {
    batch_generate(app, true, standard);
}

pub fn open_in_designcraft(app: &mut App) {
    let Some(exe) = crate::process::resolve_exe() else {
        app.log("no designcraft executable found — set PUBGEN_DESIGNCRAFT_EXE");
        return;
    };

    if let Some(doc) = app.last_output_doc.as_ref()
        && doc.exists() {
        match crate::process::launch_gui_with_doc(&exe, doc) {
            Ok(_) => app.log(format!("opened {} in DesignCraft GUI", doc.display())),
            Err(e) => app.log(format!("failed to open in DesignCraft: {e}")),
        }
        return;
    }

    if let Some(tpl) = &app.project.template {
        let run_dir = std::env::temp_dir().join("pubgen");
        let _ = std::fs::create_dir_all(&run_dir);
        let tpl_path = run_dir.join(format!("{}.designcraft", app.project.name));
        if std::fs::write(&tpl_path, tpl).is_ok() {
            match crate::process::launch_gui_with_doc(&exe, &tpl_path) {
                Ok(_) => app.log(format!("opened template {} in DesignCraft GUI", tpl_path.display())),
                Err(e) => app.log(format!("failed to open in DesignCraft: {e}")),
            }
            return;
        }
    }

    match Command::new(&exe).spawn() {
        Ok(_) => app.log(format!("launched DesignCraft GUI ({})", exe.display())),
        Err(e) => app.log(format!("failed to launch DesignCraft GUI: {e}")),
    }
}

pub fn open_pdf_viewer(app: &mut App) {
    if let Some(out) = &app.last_output {
        if out.exists() {
            match crate::process::open_in_system_viewer(out) {
                Ok(_) => app.log(format!("opened {} in system viewer", out.display())),
                Err(e) => app.log(format!("failed to open PDF: {e}")),
            }
        } else {
            app.log(format!("PDF does not exist: {}", out.display()));
        }
    } else {
        app.log("no PDF exported yet — run Batch Fill or Export PDF first");
    }
}

pub fn run_qa_audit(app: &mut App) {
    let profile = app.selected_profile.clone();
    let report = run_with_client(app, |client, app| {
        let rep = crate::audit::run_pubgen_audit(client, &profile)?;
        app.log(format!(
            "preflight QA audit: {} errors, {} warnings (profile: {})",
            rep.errors, rep.warnings, rep.profile
        ));
        Ok(rep)
    });
    if let Some(rep) = report {
        app.audit_report = Some(rep);
    }
}

pub fn export_cover_svg(app: &mut App) {
    let file = rfd::FileDialog::new()
        .add_filter("SVG Image", &["svg"])
        .set_file_name("cover.svg")
        .save_file();
    let Some(path) = file else { return };
    let svg = crate::covers::render_cover_svg(&app.project.metadata, app.cover_style, 595.0, 842.0);
    match std::fs::write(&path, svg.as_bytes()) {
        Ok(()) => {
            app.project.add_content("cover.svg".into(), svg.into_bytes());
            app.log(format!("cover exported to {}", path.display()));
        }
        Err(e) => app.log(format!("failed to export cover: {e}")),
    }
}

pub fn generate_vector_chart(app: &mut App) {
    let cats = vec!["2022".into(), "2023".into(), "2024".into(), "2025".into(), "2026".into()];
    let series = vec!["Realisasi".into(), "Target".into()];
    let data = vec![
        vec![72.4, 78.1, 84.5, 91.2, 95.0],
        vec![70.0, 75.0, 80.0, 85.0, 90.0],
    ];

    let svg = match app.chart_type {
        crate::vector::ChartType::Line => crate::vector::render_line_chart_svg(
            &app.chart_title,
            &cats,
            &series,
            &data,
            550.0,
            320.0,
        ),
        crate::vector::ChartType::Donut => {
            let donut_cats = vec!["Sektor A".into(), "Sektor B".into(), "Sektor C".into()];
            let donut_vals = vec![45.0, 35.0, 20.0];
            crate::vector::render_donut_chart_svg(&app.chart_title, &donut_cats, &donut_vals, 450.0, 300.0)
        }
        _ => crate::vector::render_bar_chart_svg(
            &app.chart_title,
            &cats,
            &series,
            &data,
            550.0,
            320.0,
        ),
    };

    let filename = format!("chart_{}.svg", app.chart_title.trim().replace(' ', "_"));
    app.project.add_content(filename.clone(), svg.into_bytes());
    app.log(format!("vector chart generated and added to project assets: {filename}"));
}

pub fn generate_choropleth_map(app: &mut App) {
    let geojson = r#"{
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": { "name": "Kecamatan Utara" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[107.2, -6.1], [107.4, -6.1], [107.4, -6.2], [107.2, -6.2], [107.2, -6.1]]]
                }
            },
            {
                "type": "Feature",
                "properties": { "name": "Kecamatan Selatan" },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[107.2, -6.2], [107.4, -6.2], [107.4, -6.3], [107.2, -6.3], [107.2, -6.2]]]
                }
            }
        ]
    }"#;

    match crate::vector::GeoMap::from_json_str(geojson) {
        Ok(geomap) => {
            let mut data = std::collections::HashMap::new();
            data.insert("Kecamatan Utara".into(), 85.0);
            data.insert("Kecamatan Selatan".into(), 62.0);

            let svg = crate::vector::render_choropleth_map_svg(&app.chart_title, &geomap, &data, 0, 500.0, 350.0);
            let filename = format!("map_{}.svg", app.chart_title.trim().replace(' ', "_"));
            app.project.add_content(filename.clone(), svg.into_bytes());
            app.log(format!("thematic choropleth map generated and added to project: {filename}"));
        }
        Err(e) => app.log(format!("failed to parse map geojson: {e}")),
    }
}

pub fn extract_toc(app: &mut App, kind: crate::toc::TocKind) {
    if app.client.is_some() {
        let entries = run_with_client(app, |client, app| {
            let doc = client.inspect()?;
            let found = crate::toc::extract_toc_from_inspect(&doc, kind);
            app.log(format!("extracted {} TOC entries from DesignCraft document", found.len()));
            Ok(found)
        });
        if let Some(list) = entries {
            app.toc_entries = list;
            return;
        }
    }

    // Fallback: build synthetic TOC from project metadata pages
    let mut fallback = Vec::new();
    for (idx, page) in app.project.metadata.pages.iter().enumerate() {
        let title = match page.kind {
            crate::metadata_form::PageKind::Paragraph => format!("BAB 0{} {}", idx + 1, page.title),
            crate::metadata_form::PageKind::Table => format!("Tabel 1.{} {}", idx + 1, page.title),
            crate::metadata_form::PageKind::Chart => format!("Grafik 1.{} {}", idx + 1, page.title),
            crate::metadata_form::PageKind::Image | crate::metadata_form::PageKind::Cover | crate::metadata_form::PageKind::Toc => continue,
        };
        if kind.matches_prefix(&title) {
            fallback.push(crate::toc::TocEntry {
                kind,
                title,
                page: idx + 1,
                level: 1,
                bilingual_title: None,
            });
        }
    }
    app.log(format!("generated {} TOC entries from project pages", fallback.len()));
    app.toc_entries = fallback;
}

pub fn find_sample_file(rel: &str) -> Option<std::path::PathBuf> {
    for prefix in &["", "../", "../../", "pubgen/"] {
        let p = std::path::Path::new(prefix).join(rel);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn find_brs_sample() -> Option<std::path::PathBuf> {
    find_sample_file("samples/brs/BRS_NTO_202610.pubgen")
        .or_else(|| find_sample_file("samples/brs/ntp202610.xlsx"))
}

pub fn find_dda_sample() -> Option<std::path::PathBuf> {
    find_sample_file("samples/DUAL_LANG/BAB_01_2025_test.xlsx")
        .or_else(|| find_sample_file("samples/DUAL_LANG/BAB_01_2025.xlsx"))
}

pub fn load_sample_brs(app: &mut App, sample_path: &std::path::Path) {
    if sample_path.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("pubgen")).unwrap_or(false) {
        if let Err(e) = open_project_path(app, sample_path) {
            app.log(format!("failed to open sample pubgen: {e}"));
        }
    } else {
        import_excel_metadata_from_path(app, sample_path);
    }
    app.log(format!("loaded sample BRS publication from {}", sample_path.display()));
}

pub fn load_sample_dda(app: &mut App, sample_path: &std::path::Path) {
    if sample_path.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("pubgen")).unwrap_or(false) {
        if let Err(e) = open_project_path(app, sample_path) {
            app.log(format!("failed to open sample pubgen: {e}"));
        }
    } else {
        import_excel_metadata_from_path(app, sample_path);
    }
    app.log(format!("loaded sample DDA publication from {}", sample_path.display()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_build_batch_script_dda() {
        let mut meta = crate::metadata_form::Metadata::default();
        meta.title = "Kecamatan Karawang Timur Dalam Angka".into();
        meta.title_en = "East Karawang District in Figures".into();
        meta.date = "2026".into();

        let slot1 = Slot {
            name: "slot:title".into(),
            frame: 10,
            story: 20,
            budget: Some(50),
            source: "".into(),
            text: "Kecamatan Karawang Timur Dalam Angka 2026".into(),
        };
        let slots = vec![&slot1];

        let script = build_batch_script(&meta, "bps_dda", &slots, None, &[]);
        assert!(script.contains("variables.define"));
        assert!(script.contains("layout.headerFooter.apply"));
        assert!(script.contains("story.setText"));
        assert!(script.contains("Kecamatan Karawang Timur"));
    }

    #[test]
    fn test_build_batch_script_brs() {
        let mut meta = crate::metadata_form::Metadata::default();
        meta.title = "Perkembangan Ekspor dan Impor".into();
        meta.release_no = "12/03/32/Th. XXVIII".into();
        meta.release_date = "1 Maret 2026".into();
        meta.release_month = "Maret 2026".into();

        let slot1 = Slot {
            name: "slot:lead".into(),
            frame: 11,
            story: 21,
            budget: Some(100),
            source: "".into(),
            text: "Nilai ekspor mencapai...".into(),
        };
        let slots = vec![&slot1];

        let script = build_batch_script(&meta, "bps_brs", &slots, None, &[]);
        assert!(script.contains("variables.define"));
        assert!(script.contains("layout.headerFooter.apply"));
        assert!(script.contains("story.setText"));
    }

    #[test]
    fn test_build_batch_script_pages_and_tables() {
        let mut meta = crate::metadata_form::Metadata::default();
        meta.pages.clear();
        meta.title = "Perkembangan Nilai Tukar Petani".into();
        meta.release_no = "61/10/32".into();

        // Add BRS Cover with hero and logo
        let mut cov = crate::metadata_form::ExcelCoverData::default();
        cov.image_hero = "assets/hero.png".into();
        cov.logo = "assets/logo.png".into();
        cov.point = "NTP naik 0.99%".into();

        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Cover,
            scope: crate::metadata_form::PageItemScope::NewPage,
            title: "Sampul Rilis".into(),
            cover: Some(cov),
            ..Default::default()
        });

        // Add a Table page
        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Table,
            scope: crate::metadata_form::PageItemScope::NewPage,
            title: "Tabel 1".into(),
            title_en: "Table 1".into(),
            chapter_id: "1".into(),
            source: Some("csv/T1.csv".into()),
            ..Default::default()
        });

        // Add a Chart object
        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Chart,
            scope: crate::metadata_form::PageItemScope::NewObject,
            title: "Grafik 1".into(),
            chart_type: Some(crate::vector::ChartType::Line),
            source: Some("csv/C1_line.csv".into()),
            ..Default::default()
        });

        // Add a second content page
        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Paragraph,
            scope: crate::metadata_form::PageItemScope::NewPage,
            title: "Ulasan Narasi".into(),
            ..Default::default()
        });

        let script = build_batch_script(&meta, "bps_brs", &[], None, &[Path::new("samples/brs")]);
        assert!(script.contains("file.open") || script.contains("file.new"));
        assert!(script.contains("layout.pages.insert"));
        assert!(script.contains("brs.designcraft"));
        assert!(script.contains("#slot:text:table_title_2"));
        assert!(script.contains("#slot:table:2"));
        assert!(script.contains("#slot:text:table_source_2"));
        assert!(script.contains("#slot:text:chart_title_3"));
        assert!(script.contains("#slot:chart:3"));
        assert!(script.contains("#slot:text:chart_source_3"));
    }

    #[test]
    fn test_build_batch_script_toc_card() {
        let mut meta = crate::metadata_form::Metadata::default();
        meta.pages.clear();
        meta.title = "Kecamatan Dalam Angka".into();
        meta.title_en = "District in Figures".into();
        meta.date = "2026".into();
        meta.layout = crate::metadata_form::PageLayout::Double;

        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Cover,
            scope: crate::metadata_form::PageItemScope::NewPage,
            ..Default::default()
        });
        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Toc,
            scope: crate::metadata_form::PageItemScope::NewPage,
            toc: Some(crate::metadata_form::TocData {
                title: "Daftar Isi / Table of Contents".into(),
                style: crate::metadata_form::TocStyle::Paragraph,
                layout: crate::metadata_form::TocLayout::Standard,
                hanging_indent: true,
                include_self: true,
                pair_bilingual: true,
            }),
            ..Default::default()
        });
        // Two narrative + one table page after the TOC, to verify page targeting.
        meta.pages.extend(vec![
            crate::metadata_form::PageEntry {
                kind: crate::metadata_form::PageKind::Paragraph,
                scope: crate::metadata_form::PageItemScope::NewPage,
                title: "BAB 1 Geografi".into(),
                chapter_id: "1".into(),
                ..Default::default()
            },
            crate::metadata_form::PageEntry {
                kind: crate::metadata_form::PageKind::Table,
                scope: crate::metadata_form::PageItemScope::NewPage,
                title: "Tabel 1.1".into(),
                source: Some("csv/T1.csv".into()),
                ..Default::default()
            },
        ]);

        let script = build_batch_script(&meta, "bps_dda", &[], None, &[]);
        // Facing cover = pages 1-2, so the TOC card lands on absolute page 3, and
        // the two content pages follow on 4 and 5.
        assert!(script.contains("\"page\":3"), "TOC must target absolute page 3:\n{script}");
        assert!(script.contains("\"command\":\"toc.generate\""));
        assert!(script.contains("\"style\":\"TOC Chapter\""));
        assert!(script.contains("\"chapterNumber\":true"));
        assert!(script.contains("\"pairColumns\":true"));
        // Dynamic TOC page count: engine autoflow is enabled on both passes.
        assert_eq!(script.matches("\"autoflow\":true").count(), 2);
        // include_self: reference the first-regenerate step's story id; the self row
        // is inserted mid-story with a trailing newline so it cannot merge into the
        // first entry's paragraph.
        assert!(script.contains("\"story\":\"$"));
        assert!(script.contains("Daftar Isi / Table of Contents\\t3\\n"), "self row must end the paragraph:\n{script}");
        assert!(script.contains("style.paragraph.apply"));
        assert!(script.contains("\"name\":\"TOC Level 1\""));
    }

    #[test]
    fn test_build_batch_script_multi_toc_page_targets() {
        let mut meta = crate::metadata_form::Metadata::default();
        meta.pages.clear();
        meta.layout = crate::metadata_form::PageLayout::Double;
        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Cover,
            scope: crate::metadata_form::PageItemScope::NewPage,
            ..Default::default()
        });
        // TOC 1 (Paragraph): 30 narrative pages estimated → span 2 pages (3-4).
        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Toc,
            scope: crate::metadata_form::PageItemScope::NewPage,
            toc: Some(crate::metadata_form::TocData {
                style: crate::metadata_form::TocStyle::Paragraph,
                ..Default::default()
            }),
            ..Default::default()
        });
        // TOC 2 (Table): no table pages → span 1 page (5).
        meta.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Toc,
            scope: crate::metadata_form::PageItemScope::NewPage,
            toc: Some(crate::metadata_form::TocData {
                style: crate::metadata_form::TocStyle::Table,
                ..Default::default()
            }),
            ..Default::default()
        });
        for i in 0..30 {
            meta.pages.push(crate::metadata_form::PageEntry {
                kind: crate::metadata_form::PageKind::Paragraph,
                scope: crate::metadata_form::PageItemScope::NewPage,
                title: format!("BAB {i}"),
                ..Default::default()
            });
        }

        let script = build_batch_script(&meta, "bps_dda", &[], None, &[]);
        // Reserved spans match the pagination model: TOC 1 gets 2 pages, TOC 2 gets 1.
        assert!(script.contains("\"count\":2"), "TOC 1 must reserve its estimated span:\n{script}");
        // Targets: TOC 1 on page 3 (cover spread = 1-2), TOC 2 on page 5.
        assert_eq!(script.matches("\"page\":3").count(), 2, "TOC 1 create+regen both target page 3");
        assert_eq!(script.matches("\"page\":5").count(), 2, "TOC 2 create+regen both target page 5");
    }

    #[test]
    fn test_ingest_linked_narratives_resolves_rtf_subfolder() {
        // Workbook layout like samples/DUAL_LANG: narratives live in `rtf/` while
        // the entries carry bare names or virtual `md/{stem}.md` sources.
        let dir = std::env::temp_dir().join(format!("pos-narrative-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("rtf")).unwrap();
        std::fs::write(dir.join("rtf/R01.rtf"), b"{\\rtf1\\ansi Ulasan bab satu.}").unwrap();

        let mut app = App::new();
        let mut entries = vec![crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Paragraph,
            scope: crate::metadata_form::PageItemScope::NewPage,
            title: "R01".into(),
            source: Some("md/R01.md".into()),
            ..Default::default()
        }];
        ingest_linked_narratives(&mut app, &dir, &mut entries);

        assert!(dir.join("md/R01.md").is_file(), "md/ must be written next to the workbook");
        assert!(app.project.content.iter().any(|(n, _)| n == "md/R01.md"), "converted md must enter project content");
        assert_eq!(entries[0].source.as_deref(), Some("md/R01.md"));

        // A bare explicit `.rtf` source resolves from the same subfolder.
        let mut entries = vec![crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Paragraph,
            scope: crate::metadata_form::PageItemScope::NewPage,
            title: "R01".into(),
            source: Some("R01.rtf".into()),
            ..Default::default()
        }];
        ingest_linked_narratives(&mut app, &dir, &mut entries);
        assert_eq!(entries[0].source.as_deref(), Some("md/R01.md"), "explicit rtf source rebinds to md");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_import_excel_metadata_from_path_sample() {
        let sample = std::path::Path::new("samples/brs/ntp202610.xlsx");
        if sample.exists() {
            // Work on a temp copy: the import must write the converted csv/md next
            // to the workbook, and the repo's samples must stay clean.
            let dir = std::env::temp_dir().join(format!("pos-import-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let copy = dir.join("ntp202610.xlsx");
            std::fs::copy(sample, &copy).unwrap();

            let mut app = App::new();
            assert_eq!(app.view, crate::main_ui::AppView::Home);
            import_excel_metadata_from_path(&mut app, &copy);
            assert_eq!(app.view, crate::main_ui::AppView::Studio);
            assert_eq!(app.selected_profile, "bps_brs");
            assert!(!app.project.metadata.pages.is_empty());
            assert_eq!(app.center_tab, crate::main_ui::CenterTab::Content);

            // Regression: the converted files land next to the workbook and inside
            // the project content.
            assert!(dir.join("csv").is_dir(), "csv/ must be created next to the workbook");
            assert!(
                app.project.content.iter().any(|(name, _)| name.starts_with("csv/")),
                "project content must embed the converted csv files"
            );
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn test_brs_generation_order_with_cli() {
        let sample_pubgen = if std::path::Path::new("samples/brs/BRS_NTO_202610.pubgen").exists() {
            Some(std::path::PathBuf::from("samples/brs/BRS_NTO_202610.pubgen"))
        } else if std::path::Path::new("../samples/brs/BRS_NTO_202610.pubgen").exists() {
            Some(std::path::PathBuf::from("../samples/brs/BRS_NTO_202610.pubgen"))
        } else {
            None
        };
        if let Some(sample_pubgen) = sample_pubgen {
            let sample_dir = sample_pubgen.parent().unwrap();
            let mut p = crate::project::Project::open(&sample_pubgen).unwrap();
            p.metadata.title = "Perkembangan Nilai Tukar Petani".into();
            p.metadata.release_no = "61/10/32".into();
            let script = build_batch_script(&p.metadata, "bps_brs", &[], Some(&p), &[sample_dir]);

            // Verify the script begins with opening the template
            assert!(script.contains("\"file.open\"") || script.contains("\"file.new\""));

            // Verify body content items were placed on spreads (item 2 was point summary, filtered out of body)
            assert!(script.contains("#slot:text:narrative_body_1"));
            assert!(script.contains("#slot:chart:1"));
            assert!(script.contains("#slot:table:1"));

            // Verify G1.png image asset resolution
            assert!(script.contains("G1.png"));
            assert!(!script.contains("\"path\":\"G1.png\""), "G1.png must be resolved to an absolute/relative path");

            // If designcraft-cli is available, test executing the script!
            if let Some(cli) = crate::process::resolve_cli() {
                let temp_dir = std::env::temp_dir().join("pubgen_test_brs");
                let _ = std::fs::create_dir_all(&temp_dir);
                let script_path = temp_dir.join("test_brs.jsonl");
                let out_doc = temp_dir.join("test_brs.designcraft");
                std::fs::write(&script_path, &script).unwrap();

                let out = Command::new(&cli)
                    .args(&["script", script_path.to_str().unwrap(), "--save", out_doc.to_str().unwrap()])
                    .output()
                    .unwrap();
                assert!(out.status.success(), "cli script failed: {}", String::from_utf8_lossy(&out.stderr));
                assert!(out_doc.exists());

                // Test post-processing
                crate::brs::post_process_brs_document(&out_doc, &[sample_dir]).unwrap();

                // Verify chart group 756 and swatches exist in out_doc
                {
                    let file = std::fs::File::open(&out_doc).unwrap();
                    let mut zip = zip::ZipArchive::new(file).unwrap();
                    let mut e = zip.by_name("document.json").unwrap();
                    let mut bytes = Vec::new();
                    std::io::Read::read_to_end(&mut e, &mut bytes).unwrap();
                    let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

                    // Check spread 1 has chart group 756
                    let sp1_items = doc["spreads"][1]["items"].as_array().unwrap();
                    assert!(sp1_items.iter().any(|it| it["id"] == 756), "Spread 1 must contain chart group 756");

                    // Check swatches contain required chart colors
                    let swatches = doc["swatches"].as_array().unwrap();
                    for req in &["#418AB3", "#002060", "#E0E0E0", "#D97706"] {
                        assert!(swatches.iter().any(|s| s["name"] == *req), "swatches must contain {req}");
                    }
                }

                // Test PDF export from post-processed document
                let out_pdf = temp_dir.join("test_brs.pdf");
                let pdf_out = Command::new(&cli)
                    .args(&["run", "--in", out_doc.to_str().unwrap(), "--export", out_pdf.to_str().unwrap()])
                    .output()
                    .unwrap();
                assert!(pdf_out.status.success(), "pdf export failed: {}", String::from_utf8_lossy(&pdf_out.stderr));
                assert!(out_pdf.exists());

                let final_doc = sample_dir.join("ntp202610.designcraft");
                let final_pdf = sample_dir.join("ntp202610.pdf");
                let _ = std::fs::copy(&out_doc, &final_doc);
                let _ = std::fs::copy(&out_pdf, &final_pdf);
            }
        }
    }

    #[test]
    fn test_ensure_project_saved_auto_saves_from_excel_source() {
        let mut app = App::new();
        // Unsaved project without excel source: fails and prompts
        assert!(!ensure_project_saved(&mut app));
        assert!(app.project_path.is_none());

        // With source excel path: auto-saves to <excel_stem>.pubgen
        let temp_dir = std::env::temp_dir().join("pubgen_test_autosave");
        let _ = std::fs::create_dir_all(&temp_dir);
        let dummy_excel = temp_dir.join("test_report_2026.xlsx");
        let expected_pubgen = temp_dir.join("test_report_2026.pubgen");
        let _ = std::fs::remove_file(&expected_pubgen);

        app.source_excel_path = Some(dummy_excel.clone());
        app.project.name = "test_report_2026".to_string();

        assert!(ensure_project_saved(&mut app));
        assert_eq!(app.project_path, Some(expected_pubgen.clone()));
        assert!(expected_pubgen.exists());

        // Clean up
        let _ = std::fs::remove_file(&expected_pubgen);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
