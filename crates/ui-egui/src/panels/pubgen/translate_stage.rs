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
/// written, not even the memory.
pub fn apply_results(draft: &mut WorkspaceDraft, tm: &mut TranslationMemory, units: &[TransUnit], results: &[(String, String)]) -> StageResult {
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
        if translate::mask::number_census(&unit.text) != translate::mask::number_census(target) {
            res.failed.push((unit.key.clone(), format!("number changed in translation: {target}")));
            continue;
        }
        // A reviewed/imported entry gates the memory (AI never clobbers it); the card
        // then shows the memory's target so the editor and the pages agree.
        let target = if tm.insert(TmEntry {
            source: unit.text.clone(),
            target: target.to_string(),
            origin: TmOrigin::Ai,
            role: unit.role.as_str().into(),
            updated: now_stamp(),
        }) {
            target.to_string()
        } else {
            tm.lookup(&unit.text).map(|e| e.target.clone()).unwrap_or_else(|| target.to_string())
        };
        if let Some(card) = draft.metadata.cards.iter_mut().find(|c| c.id == unit.card_id) {
            write_card_english(card, unit.role, &unit.text, &target);
        }
        res.translated += 1;
    }
    res
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
        let res = apply_results(&mut draft, &mut tm, &units, &[("Luas Wilayah".into(), "Total Area".into())]);
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
        let res = apply_results(&mut draft, &mut tm, &units, &[("Tabel 3.1 Luas Wilayah".into(), "Table Total Area".into())]);
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
