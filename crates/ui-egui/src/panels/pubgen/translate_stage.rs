//! The pre-compile translation stage: plan the worksheet's units, resolve the
//! local hits (translation memory, whole-string glossary), send only the misses
//! to the provider, validate the numbers survived, and write the English back
//! onto the cards and into the memory (plan docs/plans/2026-11-08, task 7).
//!
//! The stage operates on the **core** `WorkspaceDraft` (`pubgen-core`): that is
//! what `collect_units` consumes and what the Orchestrator's worksheet maps into
//! via `to_metadata`. The UI card's `title_en` is refreshed from the core card's
//! `settings["title_en"]` after the stage runs.

use pubgen_core::translation::{TmEntry, TmOrigin, TranslationMemory, now_stamp};
use pubgen_core::translation_units::{TransUnit, UnitRole, collect_units};
use pubgen_core::workspace::PublicationCard;
use pubgen_core::workspace_validation::WorkspaceDraft;

use super::PubgenState;
use super::ai::BpsGlossary;
use designcraft_translate as translate;

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
/// there is anything to render.
pub fn seed_pubgen_state(st: &mut PubgenState, tm: &TranslationMemory) {
    if tm.entries.is_empty() {
        return;
    }
    st.translations.extend(tm.to_map());
    if st.active_profile().structure.bilingual {
        st.bilingual_output = true;
    }
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
