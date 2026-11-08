//! The pre-compile translation stage: plan the worksheet's units, resolve the
//! local hits (translation memory, whole-string glossary), send only the misses
//! to the provider, validate the numbers survived, and write the English back
//! onto the cards and into the memory (plan docs/plans/2026-11-08, task 7).
//!
//! The stage operates on the **core** `WorkspaceDraft` (`pubgen-core`): that is
//! what `collect_units` consumes and what the Orchestrator's worksheet maps into
//! via `to_metadata`. The UI card's `title_en` is refreshed from the core card's
//! `settings["title_en"]` after the stage runs.
//!
//! T10 adds whole narrative bodies: `translate_narrative` translates an RTF body
//! paragraph by paragraph and rebuilds it with the source's paragraph styles and
//! character runs, so the English spread keeps the layout the headings and the
//! table of contents are derived from.

use pubgen_core::translation::{TmEntry, TmOrigin, TranslationMemory, now_stamp};
use pubgen_core::translation_units::{TransUnit, UnitRole, collect_units};
use pubgen_core::workspace::PublicationCard;
use pubgen_core::workspace_validation::WorkspaceDraft;

use designcraft_doc::{CharFormat, CharRun, Story};

use super::PubgenState;
use super::ai::BpsGlossary;
use designcraft_translate as translate;
use designcraft_translate::TranslateError;

/// What one stage run did: the counts the toolbar log line reports.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StageResult {
    pub translated: usize,
    pub reused: usize,
    pub skipped: usize,
    /// Glossary replacements `deterministic_fix` applied to accepted targets (T8).
    pub glossary_fixes: usize,
    /// (source key, why it was rejected).
    pub failed: Vec<(String, String)>,
}

/// Split the worksheet's units into local hits and provider misses. A unit with a
/// translation-memory target (non-empty) or a whole-string official glossary term
/// is resolved locally and counted as reused; only the misses must be requested.
pub fn plan_units(draft: &WorkspaceDraft, tm: &TranslationMemory, glossary: &BpsGlossary) -> (Vec<TransUnit>, StageResult) {
    let mut misses = Vec::new();
    let mut res = StageResult::default();
    for unit in collect_units(draft) {
        if tm.lookup(&unit.text).is_some_and(|e| !e.target.is_empty()) {
            res.reused += 1;
        } else if glossary.lookup(&unit.text).is_some() {
            res.reused += 1; // deterministic English; entered into the memory by `store_glossary_hits`
        } else {
            misses.push(unit);
        }
    }
    (misses, res)
}

/// Enter the plan's whole-string glossary hits into the memory as `TmOrigin::Imported`.
/// `plan_units` cannot do it (it only borrows the memory), and generation seeds its
/// cache from the memory alone, so without this step a glossary hit never renders.
pub fn store_glossary_hits(draft: &WorkspaceDraft, tm: &mut TranslationMemory, glossary: &BpsGlossary, misses: &[TransUnit]) {
    for unit in collect_units(draft) {
        if misses.iter().any(|m| m.key == unit.key) {
            continue;
        }
        if tm.lookup(&unit.text).is_some() {
            continue;
        }
        if let Some(en) = glossary.lookup(&unit.text) {
            tm.insert(TmEntry {
                source: unit.text.clone(),
                target: en.to_string(),
                origin: TmOrigin::Imported,
                role: unit.role.as_str().into(),
                updated: now_stamp(),
            });
        }
    }
}

/// The worker request list: each miss masked with the glossary pairs. `restore_outputs`
/// masks the source the same way, so the placeholders line up with the model's echo.
pub fn masked_requests(units: &[TransUnit], glossary_pairs: &[(String, String)]) -> Vec<(String, String)> {
    units.iter().map(|u| (u.key.clone(), translate::mask(&u.text, glossary_pairs).text)).collect()
}

/// Restore masked model outputs to plain English with the same glossary pairs that
/// masked the request. An output whose placeholders were mangled (missing, invented,
/// reordered) is returned as a failure instead of silently corrupting statistics.
pub fn restore_outputs(
    units: &[TransUnit],
    outputs: &[(String, String)],
    glossary_pairs: &[(String, String)],
) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let mut ok = Vec::new();
    let mut failed = Vec::new();
    for (key, output) in outputs {
        let Some(unit) = units.iter().find(|u| &u.key == key) else { continue };
        let masked = translate::mask(&unit.text, glossary_pairs);
        match translate::unmask(output, &masked) {
            Ok(restored) => ok.push((key.clone(), restored)),
            Err(e) => failed.push((key.clone(), e.to_string())),
        }
    }
    (ok, failed)
}

/// Apply validated translations: every accepted pair enters the memory as AI work and
/// is written onto its card. A pair whose number census differs from its source is the
/// guard that makes statistical tables safe — it is recorded in `failed` and nothing is
/// written, not even the memory. Official glossary terms are enforced on the model's output
/// before the census, so a model that ignored the prompt cannot ship a non-standard term.
pub fn apply_results(
    draft: &mut WorkspaceDraft,
    tm: &mut TranslationMemory,
    units: &[TransUnit],
    results: &[(String, String)],
    glossary: &[pubgen_core::glossary_fix::Entry],
) -> StageResult {
    let mut res = StageResult::default();
    for unit in units {
        let Some(target) = results.iter().find(|(k, _)| *k == unit.key).map(|(_, t)| t.trim()) else {
            res.skipped += 1;
            continue;
        };
        if target.is_empty() {
            res.skipped += 1;
            continue;
        }
        let (target, fixes) = deterministic_fix(target, glossary);
        res.glossary_fixes += fixes;
        if translate::mask::number_census(&unit.text) != translate::mask::number_census(&target) {
            res.failed.push((unit.key.clone(), format!("number changed in translation: {target}")));
            continue;
        }
        // A reviewed/imported entry gates the memory (AI never clobbers it); the card
        // then shows the memory's target so the editor and the pages agree.
        let target = if tm.insert(TmEntry {
            source: unit.text.clone(),
            target: target.clone(),
            origin: TmOrigin::Ai,
            role: unit.role.as_str().into(),
            updated: now_stamp(),
        }) {
            target
        } else {
            tm.lookup(&unit.text).map(|e| e.target.clone()).unwrap_or(target)
        };
        if let Some(card) = draft.metadata.cards.iter_mut().find(|c| c.id == unit.card_id) {
            write_card_english(card, unit.role, &unit.text, &target);
        }
        res.translated += 1;
    }
    res
}

/// Deterministic glossary enforcement: the shared `pubgen_core::glossary_fix::enforce`, applied
/// to every AI result so the official English wins even when the model ignored the prompt.
pub fn deterministic_fix(text: &str, entries: &[pubgen_core::glossary_fix::Entry]) -> (String, usize) {
    pubgen_core::glossary_fix::enforce(text, entries)
}

/// The review state shown next to a card's English draft: `Ai` is an unreviewed draft,
/// `Reviewed` is a human decision, and an `Imported` glossary hit is neither.
pub fn badge_label(tm: &TranslationMemory, source: &str) -> Option<&'static str> {
    match tm.lookup(source)?.origin {
        TmOrigin::Ai => Some("AI draft"),
        TmOrigin::Reviewed => Some("Reviewed"),
        TmOrigin::Imported => None,
    }
}

enum ReviewAction {
    Accept { source: String, target: String, buffer: egui::Id },
    Reject { source: String, buffer: egui::Id },
}

/// The Translation review pane: one row per unreviewed AI draft. The source is read-only, the
/// target is editable, Accept promotes (or replaces an edited target and promotes), Reject drops
/// the entry so the next run may translate it again. Any change sets `dirty`.
///
/// A run leaves dozens–hundreds of drafts and the right dock's body is clipped to the panel, so the
/// rows live in a vertical [`egui::ScrollArea`] like every sibling dock body.
pub fn review_pane(ui: &mut egui::Ui, tm: &mut TranslationMemory, dirty: &mut bool) {
    egui::ScrollArea::vertical().id_salt("pubgen_tm_review_scroll").auto_shrink([false, false]).show(ui, |ui| {
        review_rows(ui, tm, dirty);
    });
}

/// The pane's rows, laid out inside [`review_pane`]'s scroll area.
fn review_rows(ui: &mut egui::Ui, tm: &mut TranslationMemory, dirty: &mut bool) {
    let pending: Vec<(String, String)> = tm.pending_review().iter().map(|e| (e.source.clone(), e.target.clone())).collect();
    if pending.is_empty() {
        ui.label(egui::RichText::new("No AI drafts awaiting review.").color(ui.visuals().weak_text_color()));
        return;
    }
    ui.label(egui::RichText::new(format!("{} AI draft(s) awaiting review", pending.len())).strong());
    let mut action: Option<ReviewAction> = None;
    for (source, target) in pending {
        // Keyed by source *and* target: a later AI run that stores a new target gets a fresh
        // buffer instead of editing this run's draft with the previous edit.
        let buffer = ui.scope_id().with(("tm_review_target", &source, &target));
        let mut draft = ui.data(|d| d.get_temp::<String>(buffer)).unwrap_or_else(|| target.clone());
        ui.label(egui::RichText::new(&source).strong());
        if ui.add(egui::TextEdit::singleline(&mut draft).desired_width(f32::INFINITY)).changed() {
            ui.data_mut(|d| d.insert_temp(buffer, draft.clone()));
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(!draft.trim().is_empty(), egui::Button::new("Accept")).clicked() {
                action = Some(ReviewAction::Accept { source: source.clone(), target: draft.trim().to_string(), buffer });
            }
            if ui.button("Reject").clicked() {
                action = Some(ReviewAction::Reject { source, buffer });
            }
        });
        ui.separator();
    }
    match action {
        Some(ReviewAction::Accept { source, target, buffer }) => {
            let applied =
                if tm.lookup(&source).is_some_and(|e| e.target.trim() == target) { tm.promote(&source) } else { tm.update_target(&source, &target) };
            if applied {
                *dirty = true;
                ui.data_mut(|d| d.remove_temp::<String>(buffer));
            }
        }
        Some(ReviewAction::Reject { source, buffer }) => {
            if !tm.remove(&source) {
                return;
            }
            *dirty = true;
            ui.data_mut(|d| d.remove_temp::<String>(buffer));
        }
        None => {}
    }
}

/// Write one English string onto a core card. Title roles (and the image caption) go to
/// `settings["title_en"]` — the core card has no `title_en` field, and the UI card's
/// `title_en: String` is refreshed from this setting. Table headers, row labels, notes,
/// series labels and header/footer lines only live in the memory: generation reads them
/// through `st.translations` / `pair_cells`. An empty English never overwrites a
/// non-empty existing value (a human edit survives a re-run).
pub fn write_card_english(card: &mut PublicationCard, role: UnitRole, _source: &str, english: &str) {
    let writes_title = matches!(
        role,
        UnitRole::CardTitle
            | UnitRole::CoverTitle
            | UnitRole::TableTitle
            | UnitRole::ChartTitle
            | UnitRole::MapTitle
            | UnitRole::NarrativeTitle
            | UnitRole::DividerTitle
            | UnitRole::TocTitle
            | UnitRole::ImageCaption
    );
    if !writes_title || english.trim().is_empty() {
        return;
    }
    card.settings.insert("title_en".into(), serde_json::Value::String(english.trim().to_string()));
}

/// Seed the generation cache from the memory so a compile reuses every translation:
/// `st.translations` is the map the page generators consult (`en_text`, `pair_cells`,
/// the title fallbacks). Bilingual output turns on when the profile is bilingual and
/// there is anything to render, and `translate_with_ai` — the flag the page
/// generators gate the paired table cells and the English narrative body on — follows
/// it. Nothing else on the Generate path sets that flag, so without this line the
/// plan's English would never render.
pub fn seed_pubgen_state(st: &mut PubgenState, tm: &TranslationMemory) {
    st.translations.extend(tm.to_map());
    if !tm.entries.is_empty() && st.active_profile().structure.bilingual {
        st.bilingual_output = true;
    }
    st.translate_with_ai = st.bilingual_output && !tm.entries.is_empty();
}

// ---------------------------------------------------------------------------
// Narrative bodies (T10): translate a whole RTF body, paragraph by paragraph,
// keeping the paragraph styles and character runs the BPS heading/TOC generation
// reads. A paragraph the model drops or distorts keeps its Indonesian text.
// ---------------------------------------------------------------------------

/// Translate an RTF narrative body and return the English body as RTF, with no network:
/// `provider` receives the masked paragraph segments and returns one output per segment.
/// The testable seam [`translate_narrative`] wraps with a real backend.
pub fn translate_narrative_with(rtf: &[u8], provider: &dyn Fn(&[String]) -> Vec<String>) -> Result<Vec<u8>, TranslateError> {
    translate_body_with(rtf, &[], provider)
}

/// Translate an RTF narrative body with the configured provider (Groq, a custom endpoint or a
/// local server), masking the glossary terms and Indonesian numbers per segment and restoring
/// them strictly. A provider failure is an error; a paragraph that fails validation keeps its
/// Indonesian text and the body still publishes.
pub fn translate_narrative(rtf: &[u8], cfg: &super::ai::BackendConfig, glossary: &[(String, String)]) -> Result<Vec<u8>, TranslateError> {
    let translator = std::cell::RefCell::new(translate::OnlineTranslator::with_endpoint(&cfg.key, &cfg.model, &cfg.base_url));
    let failure: std::cell::RefCell<Option<TranslateError>> = std::cell::RefCell::new(None);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let body = translate_body_with(rtf, glossary, &|texts| {
        let result =
            translator.borrow_mut().translate(texts, translate::Lang::Indonesian, translate::Lang::English, Some(glossary), &cancel, &mut |_, _| {});
        match result {
            Ok(outputs) => outputs,
            Err(error) => {
                *failure.borrow_mut() = Some(error);
                Vec::new()
            }
        }
    })?;
    match failure.into_inner() {
        Some(error) => Err(error),
        None => Ok(body),
    }
}

/// The RTF body with every paragraph `lookup` resolves replaced by its English; `None` when
/// nothing was translated. Paragraph formats and character runs travel with the text, so the
/// generated English spread keeps the Indonesian body's layout instead of flat Arial.
pub fn translated_body_rtf_bytes(source: &[u8], lookup: &dyn Fn(&str) -> Option<String>) -> Option<Vec<u8>> {
    let imported = import_body(source).ok()?;
    let story = &imported.story;
    let mut translations: Vec<Option<String>> = Vec::new();
    let mut any = false;
    for range in story.para_ranges() {
        let english = usable_target(lookup(story.slice(range).trim()));
        any |= english.is_some();
        translations.push(english);
    }
    any.then(|| render_body(&rebuilt_story(story, &translations)).into_bytes())
}

/// A target usable as one paragraph: trimmed, non-empty and a single line. A target carrying
/// `\n` or `\r` would make `paragraphs.join("\n")` produce more paragraphs than `source.paras`,
/// shifting every later paragraph format (and `Story::check` rejects such a story), so it counts
/// as missing and the paragraph keeps its Indonesian text. The TM entry itself is never rewritten.
fn usable_target(target: Option<String>) -> Option<String> {
    target.map(|target| target.trim().to_string()).filter(|target| !target.is_empty() && !target.contains('\n') && !target.contains('\r'))
}

/// Mask and request every segment of every translatable paragraph, then rebuild the body by
/// paragraph index. A paragraph is accepted only when all its segments come back non-empty,
/// restore their placeholders, keep their numbers and are not suspiciously short; otherwise it
/// keeps its Indonesian text — one bad paragraph never costs the whole body.
fn translate_body_with(rtf: &[u8], glossary: &[(String, String)], provider: &dyn Fn(&[String]) -> Vec<String>) -> Result<Vec<u8>, TranslateError> {
    let story = import_body(rtf)?.story;
    let ranges = story.para_ranges();
    let mut requests: Vec<String> = Vec::new();
    let mut masks: Vec<translate::Masked> = Vec::new();
    // (paragraph index, first request index, request count) for every requested paragraph.
    let mut plan: Vec<(usize, usize, usize)> = Vec::new();
    for (index, range) in ranges.iter().enumerate() {
        let paragraph = story.slice(range.clone()).trim();
        if paragraph.is_empty() || paragraph.chars().any(is_story_marker) {
            continue;
        }
        let chunks = translate::segment(paragraph);
        if chunks.is_empty() {
            continue;
        }
        let first = requests.len();
        for chunk in chunks {
            let masked = translate::mask(&chunk, glossary);
            requests.push(masked.text.clone());
            masks.push(masked);
        }
        plan.push((index, first, requests.len() - first));
    }
    if requests.is_empty() {
        return Ok(rtf.to_vec());
    }
    let outputs = provider(&requests);
    let mut translations: Vec<Option<String>> = vec![None; ranges.len()];
    for (index, first, count) in plan {
        let mut joined = String::new();
        let mut accepted = true;
        for offset in 0..count {
            let (Some(output), Some(masked)) = (outputs.get(first + offset), masks.get(first + offset)) else {
                accepted = false;
                break;
            };
            match translate::unmask(output.trim(), masked) {
                Ok(restored) if !restored.trim().is_empty() => {
                    if !joined.is_empty() {
                        joined.push(' ');
                    }
                    joined.push_str(restored.trim());
                }
                _ => {
                    accepted = false;
                    break;
                }
            }
        }
        if !accepted {
            continue;
        }
        let source = story.slice(ranges[index].clone());
        let source = source.trim();
        if joined.split_whitespace().count() * 2 < source.split_whitespace().count() {
            continue; // the model answered with a fragment
        }
        if translate::mask::number_census(source) != translate::mask::number_census(&joined) {
            continue; // statistics must not move
        }
        // A provider answer spanning lines is as unusable as a missing one (see `usable_target`).
        translations[index] = usable_target(Some(joined));
    }
    Ok(render_body(&rebuilt_story(&story, &translations)).into_bytes())
}

/// Import an RTF body for the narrative rebuild. A file the importer cannot read is an error,
/// never a silently empty body: the caller falls back to the Indonesian text.
fn import_body(rtf: &[u8]) -> Result<designcraft_textimport::Imported, TranslateError> {
    designcraft_textimport::import("narrative.rtf", rtf).map_err(|error: designcraft_textimport::ImportError| {
        TranslateError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
    })
}

/// Story markers (table anchors, footnote references, …) in a paragraph: a model would mangle
/// them, so such a paragraph is left alone.
fn is_story_marker(c: char) -> bool {
    ('\u{E000}'..='\u{E1FF}').contains(&c)
}

/// The source story with `translations[i]` as paragraph `i` (`None` keeps the Indonesian text),
/// carrying the source paragraph formats and character runs over to the new text. The paragraph
/// count is always the source's: a paragraph that failed translation is the same paragraph.
fn rebuilt_story(source: &Story, translations: &[Option<String>]) -> Story {
    let ranges = source.para_ranges();
    let paragraphs: Vec<&str> = ranges
        .iter()
        .enumerate()
        .map(|(index, range)| {
            translations
                .get(index)
                .and_then(|target| target.as_deref())
                .map(str::trim)
                .filter(|target| !target.is_empty())
                .unwrap_or_else(|| source.slice(range.clone()))
        })
        .collect();
    let mut rebuilt = Story::with_text(source.id, &paragraphs.join("\n"), source.paras.first().cloned().unwrap_or_default());
    if source.paras.len() == ranges.len() {
        rebuilt.paras = source.paras.clone();
    }
    let mut chars: Vec<CharRun> = Vec::new();
    for (index, paragraph) in paragraphs.iter().enumerate() {
        push_paragraph_runs(source, &ranges[index], paragraph, &mut chars);
        if index + 1 < paragraphs.len() {
            push_paragraph_return(&mut chars, source, &ranges[index]);
        }
    }
    rebuilt.chars = chars;
    rebuilt
}

/// Push the runs of one rebuilt paragraph: the source paragraph's run pattern, scaled onto the
/// new text, so a bold lead-in stays bold and body text stays body. Boundaries are pulled back
/// to character boundaries (the RTF writer slices the text by run range).
fn push_paragraph_runs(source: &Story, range: &std::ops::Range<usize>, paragraph: &str, chars: &mut Vec<CharRun>) {
    let mut pieces: Vec<(std::ops::Range<usize>, CharFormat)> = Vec::new();
    for (run, format) in source.runs() {
        let start = run.start.max(range.start);
        let end = run.end.min(range.end);
        if start < end {
            pieces.push((start - range.start..end - range.start, format.clone()));
        }
    }
    if pieces.is_empty() {
        if !paragraph.is_empty() {
            chars.push(CharRun { len: paragraph.len(), format: source.char_format_at(range.start).clone() });
        }
        return;
    }
    let source_len: usize = pieces.iter().map(|(piece, _)| piece.len()).sum();
    let mut written = 0usize;
    for (index, (piece, format)) in pieces.iter().enumerate() {
        let end = if index + 1 == pieces.len() || source_len == 0 {
            paragraph.len()
        } else {
            // Proportional, rounded, then pulled back to a character boundary and kept in order.
            let scaled = piece.end.saturating_mul(paragraph.len()).saturating_add(source_len / 2) / source_len;
            char_boundary_floor(paragraph, scaled).clamp(written, paragraph.len())
        };
        if end > written {
            chars.push(CharRun { len: end - written, format: format.clone() });
            written = end;
        }
    }
}

/// The `\n` between two rebuilt paragraphs takes the preceding paragraph's last run format.
fn push_paragraph_return(chars: &mut Vec<CharRun>, source: &Story, range: &std::ops::Range<usize>) {
    match chars.last_mut() {
        Some(last) => last.len += 1,
        None => chars.push(CharRun { len: 1, format: source.char_format_at(range.start).clone() }),
    }
}

/// `index` clamped to `text` and pulled back to the start of the character containing it.
fn char_boundary_floor(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// RTF for a story: the rich exporter, which writes the resolved paragraph and character
/// formatting (never `write_plain_rtf`, which flattens every paragraph to Arial 11).
fn render_body(story: &Story) -> String {
    let doc = designcraft_doc::Document::new(&designcraft_doc::build::NewDocument::default());
    designcraft_textimport::export::rtf(&doc, story)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pubgen_core::glossary_fix::Entry;
    use pubgen_core::translation_units::UnitRole;
    use pubgen_core::workspace::{CardKind, CardScope, Glossary, Language, PageLayout, ProjectManifest, PublicationMetadata};

    /// Same explicit builder as Task 5: `WorkspaceDraft` has no `Default`.
    fn draft() -> WorkspaceDraft {
        WorkspaceDraft {
            manifest: ProjectManifest { format_version: 1, name: "test".into(), template: false, slots: Vec::new() },
            metadata: PublicationMetadata {
                profile_id: "bps_dda".into(),
                layout: PageLayout::Single,
                language: Language::Id,
                title: "test".into(),
                title_en: None,
                cards: Vec::new(),
                extra: Default::default(),
            },
            glossary: Glossary::default(),
            tm: TranslationMemory::default(),
            csv: Vec::new(),
            text: Vec::new(),
            assets: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn reviewed_tm_entries_are_reused_without_a_request() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Luas Wilayah menurut Kecamatan"));
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry {
            source: "Luas Wilayah menurut Kecamatan".into(),
            target: "Total Area by Subdistrict".into(),
            origin: TmOrigin::Reviewed,
            role: "table_title".into(),
            updated: now_stamp(),
        });
        let (misses, res) = plan_units(&draft, &tm, &BpsGlossary::default());
        assert!(misses.is_empty(), "a reviewed hit must not reach the provider");
        assert_eq!(res.reused, 1);
    }

    #[test]
    fn a_whole_string_glossary_hit_is_deterministic() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Kecamatan"));
        let mut g = BpsGlossary::default();
        // `GlossaryTerm::new` is private (ai.rs) — construct the struct directly.
        g.terms.push(super::super::ai::GlossaryTerm { id: "Kecamatan".into(), official: "Subdistrict".into(), ..Default::default() });
        let (misses, res) = plan_units(&draft, &TranslationMemory::default(), &g);
        assert!(misses.is_empty());
        assert_eq!(res.reused, 1);
        // The hit must also reach the memory, or generation never sees it.
        let mut tm = TranslationMemory::default();
        store_glossary_hits(&draft, &mut tm, &g, &misses);
        assert_eq!(tm.lookup("Kecamatan").map(|e| e.target.as_str()), Some("Subdistrict"));
        assert_eq!(tm.lookup("Kecamatan").map(|e| e.origin), Some(TmOrigin::Imported));
    }

    #[test]
    fn apply_results_writes_title_en_and_the_tm() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Luas Wilayah"));
        let units = vec![TransUnit {
            key: "Luas Wilayah".into(),
            text: "Luas Wilayah".into(),
            role: UnitRole::TableTitle,
            card_id: draft.metadata.cards[0].id.clone(),
        }];
        let mut tm = TranslationMemory::default();
        let res = apply_results(&mut draft, &mut tm, &units, &[("Luas Wilayah".into(), "Total Area".into())], &[]);
        assert_eq!(res.translated, 1);
        assert_eq!(draft.metadata.cards[0].settings.get("title_en").and_then(|v| v.as_str()), Some("Total Area"));
        assert_eq!(tm.lookup("Luas Wilayah").map(|e| e.origin), Some(TmOrigin::Ai));
    }

    #[test]
    fn a_number_lost_in_translation_is_a_failure_not_a_write() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Tabel 3.1 Luas Wilayah"));
        let units = vec![TransUnit {
            key: "Tabel 3.1 Luas Wilayah".into(),
            text: "Tabel 3.1 Luas Wilayah".into(),
            role: UnitRole::TableTitle,
            card_id: draft.metadata.cards[0].id.clone(),
        }];
        let mut tm = TranslationMemory::default();
        // The model dropped "3.1": the number census must reject this.
        let res = apply_results(&mut draft, &mut tm, &units, &[("Tabel 3.1 Luas Wilayah".into(), "Table Total Area".into())], &[]);
        assert_eq!(res.translated, 0);
        assert_eq!(res.failed.len(), 1);
        assert!(tm.entries.is_empty(), "a rejected unit must not enter the TM");
        assert!(draft.metadata.cards[0].settings.get("title_en").is_none(), "a rejected unit must not touch the card");
    }

    #[test]
    fn a_mangled_placeholder_is_reported_not_restored() {
        let units =
            vec![TransUnit { key: "Kecamatan 1.234".into(), text: "Kecamatan 1.234".into(), role: UnitRole::TableHeader, card_id: "t1".into() }];
        let pairs = vec![("Kecamatan".to_string(), "Subdistrict".to_string())];
        let masked = translate::mask("Kecamatan 1.234", &pairs);
        // Intact echo restores the number; an invented placeholder is a failure.
        let (ok, failed) = restore_outputs(&units, &[(units[0].key.clone(), masked.text.clone())], &pairs);
        assert_eq!(ok.len(), 1);
        assert!(ok[0].1.contains("1.234"), "the number must come back: {}", ok[0].1);
        assert!(failed.is_empty());
        let broken = format!("{} [[9]]", masked.text.replace("[[1]]", ""));
        let (ok, failed) = restore_outputs(&units, &[(units[0].key.clone(), broken)], &pairs);
        assert!(ok.is_empty());
        assert_eq!(failed.len(), 1);
    }

    #[test]
    fn seed_fills_the_generation_cache() {
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry {
            source: "Kecamatan".into(),
            target: "Subdistrict".into(),
            origin: TmOrigin::Ai,
            role: "table_header".into(),
            updated: now_stamp(),
        });
        let mut st = PubgenState::default();
        seed_pubgen_state(&mut st, &tm);
        assert_eq!(st.translations.get("Kecamatan").map(String::as_str), Some("Subdistrict"));
        // An empty memory must not touch the generation state.
        let mut st2 = PubgenState::default();
        seed_pubgen_state(&mut st2, &TranslationMemory::default());
        assert!(st2.translations.is_empty());
    }

    #[test]
    fn a_bilingual_profile_and_a_memory_arm_bilingual_rendering() {
        // The page generators gate the paired English on `translate_with_ai`; nothing on the
        // Generate path sets it, so the seed is the one place that must.
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry {
            source: "Produksi Padi".into(),
            target: "Rice Production".into(),
            origin: TmOrigin::Reviewed,
            role: "table_header".into(),
            updated: now_stamp(),
        });
        let mut st = PubgenState::default();
        assert!(!st.translate_with_ai, "a fresh state never renders bilingual English");
        seed_pubgen_state(&mut st, &tm);
        assert!(st.bilingual_output, "the default profile is bilingual");
        assert!(st.translate_with_ai, "a bilingual profile with a memory must render the English");
        assert_eq!(st.translations.get("Produksi Padi").map(String::as_str), Some("Rice Production"));
    }

    #[test]
    fn an_empty_memory_never_arms_bilingual_rendering() {
        let mut st = PubgenState::default();
        st.translate_with_ai = true; // stale flag from an earlier run
        seed_pubgen_state(&mut st, &TranslationMemory::default());
        assert!(!st.translate_with_ai, "nothing to render must not leave the gate armed");
    }

    #[test]
    fn a_single_language_profile_never_arms_bilingual_rendering() {
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry {
            source: "Produksi Padi".into(),
            target: "Rice Production".into(),
            origin: TmOrigin::Reviewed,
            role: "table_header".into(),
            updated: now_stamp(),
        });
        let mut st = PubgenState { active_profile_id: "bps_brs".into(), ..PubgenState::default() };
        seed_pubgen_state(&mut st, &tm);
        assert!(!st.bilingual_output, "BRS is single-language");
        assert!(!st.translate_with_ai, "a single-language profile must not pair cells");
        assert_eq!(st.translations.get("Produksi Padi").map(String::as_str), Some("Rice Production"), "the memory is still cached");
    }

    #[test]
    fn an_empty_translation_never_overwrites_an_existing_title_en() {
        let mut c = table_card("Luas Wilayah");
        c.settings.insert("title_en".into(), serde_json::json!("Human Edited"));
        write_card_english(&mut c, UnitRole::TableTitle, "Luas Wilayah", "");
        assert_eq!(c.settings.get("title_en").and_then(|v| v.as_str()), Some("Human Edited"));
        write_card_english(&mut c, UnitRole::TableTitle, "Luas Wilayah", "Total Area ");
        assert_eq!(c.settings.get("title_en").and_then(|v| v.as_str()), Some("Total Area"));
    }

    #[test]
    fn table_headers_go_to_the_memory_only() {
        let mut c = table_card("Kecamatan");
        write_card_english(&mut c, UnitRole::TableHeader, "Kecamatan", "Subdistrict");
        assert!(c.settings.get("title_en").is_none(), "headers render through st.translations, not the card");
    }

    #[test]
    fn glossary_enforce_is_applied_to_ai_output() {
        let entries = vec![Entry { term: "district".into(), replacement: "Subdistrict".into(), do_not_translate: false, case_preserve: true }];
        let (fixed, n) = deterministic_fix("Karawang Timur district", &entries);
        assert_eq!(n, 1);
        assert_eq!(fixed, "Karawang Timur Subdistrict");
    }

    #[test]
    fn do_not_translate_entries_are_left_alone() {
        let entries = vec![Entry { term: "NTP".into(), replacement: "whatever".into(), do_not_translate: true, case_preserve: false }];
        let (fixed, n) = deterministic_fix("Perkembangan NTP", &entries);
        assert_eq!(n, 0);
        assert_eq!(fixed, "Perkembangan NTP");
    }

    /// The stage applies the glossary to the model's output *before* the number census writes
    /// anything, so a non-standard term never reaches the card or the memory.
    #[test]
    fn apply_results_enforces_the_glossary_before_the_census() {
        let mut draft = draft();
        draft.metadata.cards.push(table_card("Karawang Timur district"));
        let units = vec![TransUnit {
            key: "Karawang Timur district".into(),
            text: "Karawang Timur district".into(),
            role: UnitRole::TableTitle,
            card_id: draft.metadata.cards[0].id.clone(),
        }];
        let entries = vec![Entry { term: "district".into(), replacement: "Subdistrict".into(), do_not_translate: false, case_preserve: true }];
        let mut tm = TranslationMemory::default();
        let res = apply_results(&mut draft, &mut tm, &units, &[("Karawang Timur district".into(), "Karawang Timur district".into())], &entries);
        assert_eq!(res.glossary_fixes, 1);
        assert_eq!(res.translated, 1);
        assert_eq!(draft.metadata.cards[0].settings.get("title_en").and_then(|v| v.as_str()), Some("Karawang Timur Subdistrict"));
        assert_eq!(tm.lookup("Karawang Timur district").map(|e| e.target.as_str()), Some("Karawang Timur Subdistrict"));
    }

    #[test]
    fn badge_label_reflects_the_review_state() {
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry { source: "PDRB".into(), target: "GRDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
        assert_eq!(badge_label(&tm, "PDRB"), Some("AI draft"));
        assert!(tm.promote("PDRB"));
        assert_eq!(badge_label(&tm, "PDRB"), Some("Reviewed"));
        tm.insert(TmEntry {
            source: "Kecamatan".into(),
            target: "Subdistrict".into(),
            origin: TmOrigin::Imported,
            role: "table_header".into(),
            updated: now_stamp(),
        });
        assert_eq!(badge_label(&tm, "Kecamatan"), None, "a glossary hit is not a reviewable draft");
        assert_eq!(badge_label(&tm, "missing"), None);
    }

    /// The Accept button's unchanged-target branch: a human decision must survive a later AI run.
    #[test]
    fn accepting_a_draft_promotes_it_to_reviewed() {
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
        assert_eq!(tm.pending_review().len(), 1);
        tm.promote("PDRB");
        assert!(tm.pending_review().is_empty());
        // A later AI run must not undo the human decision.
        tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
        assert_eq!(tm.lookup("PDRB").map(|e| e.origin), Some(TmOrigin::Reviewed));
    }

    /// The Accept button's edited-target branch (`update_target`): the target is replaced too, not
    /// only the origin.
    #[test]
    fn editing_a_draft_promotes_and_replaces() {
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
        assert!(tm.update_target("PDRB", "GRDP"));
        assert_eq!(tm.lookup("PDRB").map(|e| e.target.as_str()), Some("GRDP"));
        assert_eq!(tm.lookup("PDRB").map(|e| e.origin), Some(TmOrigin::Reviewed));
    }

    fn review_harness(tm: TranslationMemory) -> egui_kittest::Harness<'static, (TranslationMemory, bool)> {
        egui_kittest::Harness::builder()
            .with_size(egui::vec2(560.0, 400.0))
            .with_accessibility_check(false)
            .build_ui_state(|ui, (tm, dirty): &mut (TranslationMemory, bool)| review_pane(ui, tm, dirty), (tm, false))
    }

    #[test]
    fn review_pane_accepts_a_draft_through_the_button() {
        use egui_kittest::kittest::Queryable;
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
        let mut h = review_harness(tm);
        h.run();
        h.get_by_label("Accept").click();
        h.run();
        assert!(h.state().0.pending_review().is_empty(), "Accept promotes the draft");
        assert_eq!(h.state().0.lookup("PDRB").map(|e| e.origin), Some(TmOrigin::Reviewed));
        assert!(h.state().1, "accepting marks the workspace dirty");
    }

    #[test]
    fn review_pane_rejects_a_draft_through_the_button() {
        use egui_kittest::kittest::Queryable;
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
        let mut h = review_harness(tm);
        h.run();
        h.get_by_label("Reject").click();
        h.run();
        assert!(h.state().0.entries.is_empty(), "Reject removes the draft");
        assert!(h.state().1, "rejecting marks the workspace dirty");
    }

    /// The pane's edited-target branch: typing a different target into the row and clicking Accept
    /// replaces the memory's target and promotes it (not just `promote`, which keeps "GDP").
    #[test]
    fn review_pane_accepts_an_edited_target_through_the_button() {
        use egui_kittest::kittest::Queryable;
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry { source: "PDRB".into(), target: "GDP".into(), origin: TmOrigin::Ai, role: "table_header".into(), updated: now_stamp() });
        let mut h = review_harness(tm);
        h.run();
        h.get_by_role(egui::Role::TextInput).set_text_value_accesskit("GRDP");
        h.run();
        h.get_by_label("Accept").click();
        h.run();
        assert_eq!(h.state().0.lookup("PDRB").map(|e| e.target.as_str()), Some("GRDP"), "Accept applies the typed target");
        assert_eq!(h.state().0.lookup("PDRB").map(|e| e.origin), Some(TmOrigin::Reviewed));
        assert!(h.state().1, "accepting marks the workspace dirty");
    }

    /// A real run leaves dozens–hundreds of drafts. The dock body is clipped to the panel, so the
    /// rows must live in a scroll area (like every sibling dock body) or all but the first screen
    /// are unreachable.
    #[test]
    fn the_review_pane_scrolls_down_to_the_last_of_many_drafts() {
        use egui_kittest::kittest::Queryable;
        let mut tm = TranslationMemory::default();
        for i in 0..50 {
            tm.insert(TmEntry {
                source: format!("Source {i:02}"),
                target: format!("Target {i:02}"),
                origin: TmOrigin::Ai,
                role: "table_header".into(),
                updated: now_stamp(),
            });
        }
        let mut h = egui_kittest::Harness::builder()
            .with_size(egui::vec2(320.0, 400.0))
            .with_accessibility_check(false)
            .build_ui_state(|ui, (tm, dirty): &mut (TranslationMemory, bool)| review_pane(ui, tm, dirty), (tm, false));
        h.run();
        // Rows are laid out in memory even when clipped, so reachability is a *position*: the
        // first draft is inside the 400 px viewport and the last one is below it.
        let first = h.query_by_label("Source 00").map(|n| n.rect()).expect("the first draft is laid out");
        let last = h.query_by_label("Source 49").map(|n| n.rect()).expect("the last draft is laid out");
        assert!(first.top() >= 0.0 && first.bottom() <= 400.0, "the first draft starts in the viewport: {first:?}");
        assert!(last.top() >= 400.0, "the last draft starts below the viewport's 400 px: {last:?}");
        // A ScrollArea only takes the wheel while the pointer is over it.
        h.event(egui::Event::PointerMoved(egui::pos2(160.0, 200.0)));
        h.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -20_000.0),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: egui::Modifiers::NONE,
        });
        h.run_steps(20); // smooth scrolling needs frames to settle
        let first = h.query_by_label("Source 00").map(|n| n.rect()).expect("the first draft is still laid out after scrolling");
        let last = h.query_by_label("Source 49").map(|n| n.rect()).expect("the last draft is laid out after scrolling");
        assert!(last.top() < 400.0 && last.bottom() > 0.0, "scrolling must bring the last draft into the viewport: {last:?}");
        assert!(first.bottom() <= 0.0, "the first draft scrolled off the top: {first:?}");
    }

    #[test]
    fn narrative_paragraph_count_and_styles_survive_translation() {
        let rtf =
            br"{\rtf1\ansi{\stylesheet{\s1\b\fs40 heading 1;}}\pard\plain \s1\qc Heading_one\par Body paragraph one.\par Body paragraph two.\par}";
        let out = translate_narrative_with(rtf, &|texts| {
            // Canned provider: return one EN string per input paragraph.
            texts.iter().map(|t| format!("EN {t}")).collect()
        })
        .expect("translation");
        let imported = designcraft_textimport::import("t.rtf", &out).expect("reimport");
        // Use the same paragraph-count API the plain-text path uses (`ai.rs:793`).
        assert_eq!(paragraph_count(&imported.story), 3, "paragraph count must be preserved");
        assert!(imported.story.text.contains("EN "), "{:?}", imported.story.text);
        // The heading paragraph's own formatting travels with its translated text.
        assert_eq!(imported.story.paras[0].para.align, Some(designcraft_doc::Align::Center), "the heading stays centred");
        // Guard, and the reason `pages.rs` heading detection needs no change: the rich exporter
        // writes *resolved* formatting, not a `\stylesheet` table, so the reimported body keeps
        // its layout but not its style names. Heading detection reads the Indonesian body
        // (`pages.rs:1202`) and template pages tag by slot label, never by the English body's
        // paragraph style.
        assert_eq!(imported.story.paras[0].style, designcraft_doc::story::BASIC_PARAGRAPH, "export::rtf carries formatting, not style names");
    }

    #[test]
    fn a_paragraph_the_model_dropped_keeps_its_indonesian_text() {
        let rtf = br"{\rtf1\ansi Satu.\par Dua.\par {\b Tiga.}\par}";
        let out = translate_narrative_with(rtf, &|texts| {
            // The model answers every paragraph except `Dua.` (an empty answer is dropped).
            texts.iter().map(|t| if t == "Dua." { String::new() } else { format!("EN {t}") }).collect()
        })
        .expect("must not fail the whole body");
        let imported = designcraft_textimport::import("t.rtf", &out).expect("reimport");
        assert!(imported.story.text.contains("Dua."), "the untranslated paragraph must survive");
        // The paragraph count and the formats after the Indonesian fallback must be untouched.
        let ranges = imported.story.para_ranges();
        assert_eq!(ranges.len(), 3, "the paragraph count is the source's");
        let third = imported.story.slice(ranges[2].clone()).to_string();
        assert!(third.contains("EN Tiga."), "the third paragraph translates: {third:?}");
        let at = ranges[2].start + third.find("Tiga").expect("the third paragraph's text is there");
        let (_, format) = imported.story.runs().find(|(run, _)| run.start <= at && at < run.end).expect("the third paragraph has a run");
        assert_eq!(format.over.font_style.as_deref(), Some("Bold"), "the third paragraph stays bold after an Indonesian fallback");
    }

    /// A TM target is one paragraph. An embedded return would add a paragraph, push every later
    /// paragraph onto the wrong format and make `Story::check` reject the body, so such a target
    /// is treated as missing and that paragraph keeps its Indonesian text.
    #[test]
    fn a_translation_with_an_embedded_newline_keeps_its_indonesian_paragraph() {
        let rtf = br"{\rtf1\ansi Satu.\par Dua.\par}";
        let lookup = |paragraph: &str| match paragraph {
            "Satu." => Some("One.\nInserted paragraph.".to_string()),
            "Dua." => Some("Two.".to_string()),
            _ => None,
        };
        let out = translated_body_rtf_bytes(&rtf[..], &lookup).expect("Dua. translates");
        let imported = designcraft_textimport::import("t.rtf", &out).expect("reimport");
        assert_eq!(imported.story.para_ranges().len(), 2, "a newline in a target must not add a paragraph");
        assert!(imported.story.text.contains("Satu."), "the paragraph with the unusable target keeps its Indonesian text: {:?}", imported.story.text);
        assert!(imported.story.text.contains("Two."), "{:?}", imported.story.text);
    }

    fn paragraph_count(story: &designcraft_doc::Story) -> usize {
        story.para_ranges().len()
    }

    /// The rebuild is a Story, not a flattened text blob: paragraph styles and the run pattern
    /// survive, so the English spread keeps the Indonesian body's layout (T10).
    #[test]
    fn the_rebuilt_body_keeps_the_source_paragraph_styles_and_runs() {
        let rtf = br"{\rtf1\ansi{\stylesheet{\s1\b\fs40 heading 1;}}
\pard\plain \s1 Judul Bagian\par
\pard\plain {\b Tebal} biasa.\par}";
        let imported = designcraft_textimport::import("t.rtf", &rtf[..]).expect("import");
        let rebuilt = rebuilt_story(&imported.story, &[Some("Section title".into()), Some("Bold lead and body text.".into())]);
        assert!(rebuilt.check().is_ok(), "{:?}", rebuilt.check());
        assert_eq!(rebuilt.text, "Section title\nBold lead and body text.");
        assert_eq!(rebuilt.paras[0].style, "Heading 1", "the heading style must be carried over");
        assert_eq!(rebuilt.paras[1].style, designcraft_doc::story::BASIC_PARAGRAPH);
        let bold = rebuilt.text.find("Bold").expect("the translated lead is there");
        assert_eq!(rebuilt.char_format_of(bold).over.font_style.as_deref(), Some("Bold"), "the bold lead stays bold");
        let plain = rebuilt.text.find("and").expect("the body tail is there");
        assert_eq!(rebuilt.char_format_of(plain).over.font_style, None, "the plain tail stays plain");
    }

    /// Numbers are the statistics: a segment whose numbers moved keeps its Indonesian paragraph.
    #[test]
    fn a_paragraph_whose_numbers_moved_keeps_its_indonesian_text() {
        let rtf = br"{\rtf1\ansi Naik 1.234 persen.\par";
        let out =
            translate_narrative_with(&rtf[..], &|texts| texts.iter().map(|t| t.replace("persen", "1.500 percent")).collect()).expect("no failure");
        let imported = designcraft_textimport::import("t.rtf", &out).expect("reimport");
        assert!(imported.story.text.contains("Naik 1.234 persen."), "{:?}", imported.story.text);
    }

    fn table_card(title: &str) -> pubgen_core::PublicationCard {
        pubgen_core::PublicationCard {
            id: format!("table-{title}"),
            kind: CardKind::Table,
            scope: CardScope::NewPage,
            source: None,
            title: title.into(),
            settings: Default::default(),
        }
    }
}
