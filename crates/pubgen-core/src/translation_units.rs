//! Translation unit collection: what gets translated, per card kind, with skip rules.
//!
//! Titles, table headers/row labels/notes, chart series labels, map titles, image captions,
//! and the paragraph bodies of narrative cards (one unit per paragraph, keyed by its trimmed
//! text so the page generator's `translated_body_rtf` finds the English again).
use crate::workspace::{CardKind, PublicationCard};
use crate::workspace_validation::WorkspaceDraft;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Guard: packages are untrusted input; collection must not grow without bound.
const MAX_UNITS: usize = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitRole {
    CardTitle,
    CoverTitle,
    TableTitle,
    TableHeader,
    TableRowLabel,
    TableNote,
    NarrativeTitle,
    NarrativeBody,
    ChartTitle,
    ChartSeries,
    MapTitle,
    ImageCaption,
    TocTitle,
    DividerTitle,
    HeaderFooter,
}

impl UnitRole {
    pub fn as_str(self) -> &'static str {
        match self {
            UnitRole::CardTitle => "card_title",
            UnitRole::CoverTitle => "cover_title",
            UnitRole::TableTitle => "table_title",
            UnitRole::TableHeader => "table_header",
            UnitRole::TableRowLabel => "table_row_label",
            UnitRole::TableNote => "table_note",
            UnitRole::NarrativeTitle => "narrative_title",
            UnitRole::NarrativeBody => "narrative_body",
            UnitRole::ChartTitle => "chart_title",
            UnitRole::ChartSeries => "chart_series",
            UnitRole::MapTitle => "map_title",
            UnitRole::ImageCaption => "image_caption",
            UnitRole::TocTitle => "toc_title",
            UnitRole::DividerTitle => "divider_title",
            UnitRole::HeaderFooter => "header_footer",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransUnit {
    pub key: String,
    pub text: String,
    pub role: UnitRole,
    pub card_id: String,
}

/// Skip rules, in order: trim/empty, numeric, identifier, protected list, bilingual pair.
pub fn is_skippable(text: &str, protected: &[String]) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    if is_numeric(trimmed) {
        return true;
    }
    if is_identifier(trimmed) {
        return true;
    }
    let normalized = normalize(trimmed);
    if protected.iter().any(|p| normalize(p) == normalized) {
        return true;
    }
    trimmed.contains(" / ") || trimmed.contains('\u{2028}')
}

/// Numeric cell scan without a regex: at least one digit, everything else a
/// numeric decoration ( separators, percent, parentheses, sign, whitespace ). A lone `-`
/// is an empty placeholder cell.
fn is_numeric(text: &str) -> bool {
    if text == "-" {
        return true;
    }
    let mut digits = 0;
    for c in text.chars() {
        match c {
            '0'..='9' => digits += 1,
            '.' | ',' | '%' | '(' | ')' | '+' | '-' | '/' | '–' | '—' => {}
            c if c.is_whitespace() => {}
            _ => return false,
        }
    }
    digits > 0
}

fn is_identifier(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.starts_with("issn") || lower.starts_with("isbn") || lower.starts_with("katalog bps:") || lower.starts_with("http") {
        return true;
    }
    if text.contains('@') || text.contains("://") {
        return true;
    }
    (text.contains('/') || text.contains('\\')) && has_known_extension(&lower)
}

fn has_known_extension(lower: &str) -> bool {
    [".csv", ".json", ".geojson", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".pdf", ".xlsx", ".xls", ".rtf", ".txt", ".md"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

/// Trim plus collapse internal whitespace, lowercased: the protected-list comparison form.
fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Merged header stacks: split on `--`, capped at 3 levels (mirrors the table
/// renderer's header auto-detection, `tables.rs:131-149`).
pub fn split_header_stacks(cell: &str) -> Vec<String> {
    let trimmed = cell.trim();
    if trimmed.contains("--") { trimmed.splitn(3, "--").map(|s| s.trim().to_string()).collect() } else { vec![trimmed.to_string()] }
}

/// Note/source row prefixes, copied from the sheet ingest's `matches_table_note_keyword`
/// (`excel.rs:707-717`): do not invent new ones here.
fn is_table_note_row(first_cell: &str) -> bool {
    let lower = first_cell.trim().to_lowercase();
    lower.starts_with("keterangan")
        || lower.starts_with("ket.")
        || lower.starts_with("ket ")
        || lower == "ket"
        || lower.starts_with("catatan")
        || lower.starts_with("note")
        || lower.starts_with("sumber")
        || lower.starts_with("source")
}

/// Names that must never be machine-translated: GeoJSON region names referenced by
/// MapChart cards (their `nmdesa`/`nmkec` columns) plus agency/institution and
/// person-name fields from Cover cards.
pub fn protected_names(draft: &WorkspaceDraft) -> Vec<String> {
    let mut out = Vec::new();
    for card in &draft.metadata.cards {
        match card.kind {
            CardKind::MapChart => {
                if let Some(rows) = csv_rows(draft, card_source(card)) {
                    let region_cols = header_indexes(&rows, &["nmdesa", "nmkec"]);
                    for row in rows.iter().skip(1) {
                        for index in &region_cols {
                            if let Some(name) = row.get(*index) {
                                let trimmed = name.trim();
                                if !trimmed.is_empty() {
                                    out.push(trimmed.to_string());
                                }
                            }
                        }
                    }
                }
            }
            CardKind::Cover => {
                for key in ["agency", "pub_office", "pic_name", "cover_agency", "cover_pub_office", "cover_pic_name"] {
                    if let Some(value) = card.settings.get(key).and_then(|v| v.as_str()) {
                        let trimmed = value.trim();
                        if !trimmed.is_empty() {
                            out.push(trimmed.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

pub fn collect_units(draft: &WorkspaceDraft) -> Vec<TransUnit> {
    let protected = protected_names(draft);
    let mut out: Vec<TransUnit> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for card in &draft.metadata.cards {
        collect_card(draft, card, &protected, &mut out, &mut seen);
    }
    out
}

fn collect_card(draft: &WorkspaceDraft, card: &PublicationCard, protected: &[String], out: &mut Vec<TransUnit>, seen: &mut BTreeSet<String>) {
    match card.kind {
        CardKind::Cover => {
            push(out, seen, protected, &card.title, UnitRole::CoverTitle, &card.id);
            for key in ["cover_title2", "title2"] {
                if let Some(value) = card.settings.get(key).and_then(|v| v.as_str()) {
                    push(out, seen, protected, value, UnitRole::CoverTitle, &card.id);
                }
            }
        }
        CardKind::Toc => push(out, seen, protected, &card.title, UnitRole::TocTitle, &card.id),
        CardKind::Table => {
            push(out, seen, protected, &card.title, UnitRole::TableTitle, &card.id);
            if let Some(rows) = csv_rows(draft, card_source(card)) {
                for (index, row) in rows.iter().enumerate() {
                    if index == 0 {
                        // The renderer splits every header column (`tables.rs:132-146`),
                        // so every cell of row 0 is a header, not just the first.
                        for cell in row {
                            for part in split_header_stacks(cell) {
                                push(out, seen, protected, &part, UnitRole::TableHeader, &card.id);
                            }
                        }
                        continue;
                    }
                    let Some(first) = row.first().map(String::as_str) else { continue };
                    if is_table_note_row(first) {
                        push(out, seen, protected, first, UnitRole::TableNote, &card.id);
                    } else {
                        push(out, seen, protected, first, UnitRole::TableRowLabel, &card.id);
                    }
                }
            }
        }
        CardKind::Chart => {
            push(out, seen, protected, &card.title, UnitRole::ChartTitle, &card.id);
            for (key, value) in &card.settings {
                if key.starts_with("series_") && key.ends_with("_label") {
                    push(out, seen, protected, value.as_str().unwrap_or_default(), UnitRole::ChartSeries, &card.id);
                }
            }
        }
        CardKind::MapChart => {
            push(out, seen, protected, &card.title, UnitRole::MapTitle, &card.id);
            if let Some(rows) = csv_rows(draft, card_source(card)) {
                let title_cols = header_indexes(&rows, &["title"]);
                for row in rows.iter().skip(1) {
                    for index in &title_cols {
                        if let Some(value) = row.get(*index) {
                            push(out, seen, protected, value, UnitRole::MapTitle, &card.id);
                        }
                    }
                }
            }
        }
        CardKind::Narrative => {
            push(out, seen, protected, &card.title, UnitRole::NarrativeTitle, &card.id);
            for paragraph in narrative_paragraphs(draft, card) {
                push(out, seen, protected, &paragraph, UnitRole::NarrativeBody, &card.id);
            }
        }
        CardKind::Divider => push(out, seen, protected, &card.title, UnitRole::DividerTitle, &card.id),
        CardKind::Image => {
            let caption = card.settings.get("caption").and_then(|v| v.as_str()).unwrap_or(card.title.as_str());
            push(out, seen, protected, caption, UnitRole::ImageCaption, &card.id);
        }
    }
}

/// Append one unit unless skipped or already present; `key` is the trimmed source text,
/// the same key `st.translations` uses. Capped at [`MAX_UNITS`].
fn push(out: &mut Vec<TransUnit>, seen: &mut BTreeSet<String>, protected: &[String], text: &str, role: UnitRole, card_id: &str) {
    if out.len() >= MAX_UNITS {
        return;
    }
    let trimmed = text.trim();
    if is_skippable(trimmed, protected) || seen.contains(trimmed) {
        return;
    }
    seen.insert(trimmed.to_string());
    out.push(TransUnit { key: trimmed.to_string(), text: trimmed.to_string(), role, card_id: card_id.to_string() });
}

/// Staged CSV source: the card's `source` field, or a `source` setting.
fn card_source(card: &PublicationCard) -> Option<&str> {
    card.source.as_deref().or_else(|| card.settings.get("source").and_then(|v| v.as_str()))
}

/// The paragraph texts of a narrative card's staged text file, trimmed and non-empty.
///
/// The extraction is the one the page generator's `rtf_paragraphs` (`ai.rs`) uses, so a unit's
/// key is exactly the paragraph text `translated_body_rtf` looks up when it rebuilds the
/// English body: the two must not drift apart. An unreadable file contributes nothing.
fn narrative_paragraphs(draft: &WorkspaceDraft, card: &PublicationCard) -> Vec<String> {
    let Some((name, bytes)) = narrative_source(draft, card_source(card)) else { return Vec::new() };
    let Ok(imported) = designcraft_textimport::import(name, bytes) else { return Vec::new() };
    imported.story.text.lines().map(str::trim).filter(|line| !line.is_empty()).take(MAX_UNITS).map(str::to_string).collect()
}

/// The staged narrative bytes (`name`, bytes) for a card source: the exact staged name, or a
/// staged name ending in `/<source>` (a package may stage `rtf/U5.rtf` under `text/rtf/U5.rtf`,
/// the same aliases `WorkspaceDraft::validate` accepts).
fn narrative_source<'a>(draft: &'a WorkspaceDraft, source: Option<&str>) -> Option<(&'a str, &'a [u8])> {
    let source = source?.replace('\\', "/");
    let suffix = format!("/{source}");
    draft.text.iter().find(|(name, _)| *name == source || name.ends_with(&suffix)).map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
}

fn header_indexes(rows: &[Vec<String>], names: &[&str]) -> Vec<usize> {
    let Some(header) = rows.first() else { return Vec::new() };
    header.iter().enumerate().filter(|(_, cell)| names.contains(&cell.trim().to_lowercase().as_str())).map(|(index, _)| index).collect()
}

/// Parse a staged CSV, sniffing the delimiter from the header line (BPS sheets use `;`).
/// Hostile bytes: lossy decode, stop at the first malformed record, bound rows to the
/// unit cap. Returns `None` when the card has no staged CSV.
fn csv_rows(draft: &WorkspaceDraft, source: Option<&str>) -> Option<Vec<Vec<String>>> {
    let source = source?;
    let (_, bytes) = draft.csv.iter().find(|(name, _)| name == source)?;
    let text = String::from_utf8_lossy(bytes);
    let first_line = text.split('\n').next().unwrap_or_default().to_string();
    // Tie-break towards `;` (BPS convention) by ordering it last for `max_by_key`.
    let mut counts = [(',', 0usize), ('\t', 0), ('|', 0), (';', 0)];
    for c in first_line.chars() {
        for entry in counts.iter_mut() {
            if entry.0 == c {
                entry.1 += 1;
            }
        }
    }
    let delimiter = counts.iter().max_by_key(|entry| entry.1).filter(|entry| entry.1 > 0).map_or(b',', |entry| entry.0 as u8);
    let mut reader = csv::ReaderBuilder::new().has_headers(false).delimiter(delimiter).from_reader(bytes.as_slice());
    let mut rows: Vec<Vec<String>> = Vec::new();
    for record in reader.records() {
        match record {
            Ok(row) => rows.push(row.iter().map(str::to_owned).collect()),
            Err(_) => break,
        }
        if rows.len() >= MAX_UNITS {
            break;
        }
    }
    Some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translation::TranslationMemory;
    use crate::workspace::{CardKind, CardScope, Glossary, Language, PageLayout, ProjectManifest, PublicationCard, PublicationMetadata};
    use crate::workspace_validation::WorkspaceDraft;

    /// `WorkspaceDraft` has no `Default` (verified: `#[derive(Debug, Clone)]`,
    /// `workspace_validation.rs:21`), so tests build it explicitly.
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
    fn numeric_cells_are_skipped() {
        for n in ["1.234,56", "2026", "(1)", "12,5%", "-", "1 234", "0", "3.1", "2024/2025", "1–2"] {
            assert!(is_skippable(n, &[]), "must skip {n}");
        }
    }

    #[test]
    fn identifiers_are_skipped() {
        for s in ["ISSN 2089-1234", "Katalog BPS: 1101001", "someone@bps.go.id", "0812-3456-7890", "https://example.com", "csv/T1.csv"] {
            assert!(is_skippable(s, &[]), "must skip {s}");
        }
    }

    #[test]
    fn protected_proper_nouns_are_skipped() {
        let protected = vec!["Karawang Timur".to_string()];
        assert!(is_skippable("Karawang Timur", &protected));
        assert!(is_skippable("karawang  timur", &protected), "normalised match");
        assert!(!is_skippable("Luas Wilayah", &protected));
    }

    #[test]
    fn already_bilingual_text_is_skipped() {
        assert!(is_skippable("Geografi / Geography", &[]));
        assert!(is_skippable("Geografi\u{2028}Geography", &[]));
    }

    #[test]
    fn real_indonesian_labels_are_collected() {
        for s in ["Luas Wilayah menurut Kecamatan", "Jumlah Penduduk", "Sumber: BPS Kabupaten Karawang", "Ringkasan Poin Indikator Utama"] {
            assert!(!is_skippable(s, &[]), "must collect {s}");
        }
    }

    #[test]
    fn empty_and_whitespace_are_skipped() {
        assert!(is_skippable("", &[]));
        assert!(is_skippable("   ", &[]));
    }

    #[test]
    fn header_stacks_split_on_double_dash() {
        assert_eq!(split_header_stacks("Kecamatan--Luas Wilayah--Jumlah Penduduk"), vec!["Kecamatan", "Luas Wilayah", "Jumlah Penduduk"]);
        assert_eq!(split_header_stacks("Kecamatan"), vec!["Kecamatan"]);
    }

    #[test]
    fn cover_identifiers_are_not_collected() {
        let mut draft = draft();
        draft.metadata.cards.push(card(
            CardKind::Cover,
            "Perkembangan NTP Oktober 2026",
            &[
                ("cover_year", "2026"),
                ("cover_catalogue", "Katalog BPS: 1101001"),
                ("cover_code_num", "ISSN 2089-1234"),
                ("cover_pic_name", "Budi Santoso"),
                ("cover_office_address", "Jl. PHH Mustofa No. 1"),
            ],
        ));
        let units = collect_units(&draft);
        let texts: Vec<&str> = units.iter().map(|u| u.text.as_str()).collect();
        assert!(texts.iter().any(|t| t.contains("Perkembangan NTP")), "{texts:?}");
        for forbidden in ["2026", "Katalog BPS: 1101001", "ISSN 2089-1234", "Budi Santoso", "Jl. PHH Mustofa No. 1"] {
            assert!(!texts.contains(&forbidden), "must not collect {forbidden}: {texts:?}");
        }
    }

    #[test]
    fn map_region_names_are_protected_but_the_title_is_collected() {
        let mut draft = draft();
        draft.csv.push(("csv/M1.csv".into(), b"nmdesa;title;title_en;value\nKarawang Timur;Peta Kepadatan;;120\n".to_vec()));
        draft.metadata.cards.push(card(CardKind::MapChart, "Peta Kepadatan Penduduk", &[("source", "csv/M1.csv")]));
        let protected = protected_names(&draft);
        assert!(protected.iter().any(|p| p == "Karawang Timur"));
        let units = collect_units(&draft);
        assert!(units.iter().any(|u| u.role == UnitRole::MapTitle));
        assert!(!units.iter().any(|u| u.text == "Karawang Timur"));
    }

    #[test]
    fn all_header_columns_are_collected_as_table_headers() {
        let mut draft = draft();
        draft.csv.push(("csv/T1.csv".into(), b"Kecamatan;Luas Wilayah (km2);Jumlah Penduduk (jiwa)\nBatujaya;1.234,5;56.789\n".to_vec()));
        draft.metadata.cards.push(card(CardKind::Table, "Luas Wilayah", &[("source", "csv/T1.csv")]));
        let units = collect_units(&draft);
        for header in ["Kecamatan", "Luas Wilayah (km2)", "Jumlah Penduduk (jiwa)"] {
            assert!(units.iter().any(|u| u.role == UnitRole::TableHeader && u.text == header), "missing header column {header}: {units:?}");
        }
    }

    #[test]
    fn units_are_deduplicated_by_key() {
        let mut draft = draft();
        draft.metadata.cards.push(card(CardKind::Table, "Luas Wilayah", &[]));
        draft.metadata.cards.push(card(CardKind::Table, "Luas Wilayah", &[]));
        let units = collect_units(&draft);
        assert_eq!(units.iter().filter(|u| u.text == "Luas Wilayah").count(), 1);
    }

    /// A narrative card's paragraph bodies are units of their own, keyed by their trimmed text
    /// (the key `translated_body_rtf` looks up), while numeric-only paragraphs stay out.
    #[test]
    fn narrative_paragraphs_are_collected_as_body_units() {
        let mut draft = draft();
        let rtf = br"{\rtf1\ansi\deff0{\fonttbl{\f0 Arial;}}\pard\plain Ulasan wilayah Karawang.\par 1.234,56\par\par Tutup.}";
        draft.text.push(("rtf/R01.rtf".into(), rtf.to_vec()));
        draft.metadata.cards.push(narrative_card("BAB 01 Geografi", "rtf/R01.rtf"));
        let units = collect_units(&draft);
        assert!(units.iter().any(|u| u.role == UnitRole::NarrativeTitle && u.text == "BAB 01 Geografi"), "{units:?}");
        for paragraph in ["Ulasan wilayah Karawang.", "Tutup."] {
            assert!(units.iter().any(|u| u.role == UnitRole::NarrativeBody && u.text == paragraph), "missing body paragraph {paragraph}: {units:?}");
        }
        assert!(!units.iter().any(|u| u.text == "1.234,56"), "a numeric paragraph is not a translation unit");
        assert_eq!(units.iter().filter(|u| u.role == UnitRole::NarrativeBody).count(), 2, "the empty paragraph adds nothing");
    }

    /// The staged name need not equal the card's source: a package stages `text/rtf/R01.rtf`.
    #[test]
    fn a_narrative_body_is_found_under_a_staging_prefix() {
        let mut draft = draft();
        let rtf = br"{\rtf1\ansi Satu paragraf.}";
        draft.text.push(("text/rtf/R01.rtf".into(), rtf.to_vec()));
        draft.metadata.cards.push(narrative_card("R01", "rtf/R01.rtf"));
        let units = collect_units(&draft);
        assert!(units.iter().any(|u| u.role == UnitRole::NarrativeBody && u.text == "Satu paragraf."), "{units:?}");
    }

    fn narrative_card(title: &str, source: &str) -> PublicationCard {
        PublicationCard {
            id: format!("narrative-{title}"),
            kind: CardKind::Narrative,
            scope: CardScope::NewPage,
            source: Some(source.into()),
            title: title.into(),
            settings: Default::default(),
        }
    }

    fn card(kind: CardKind, title: &str, settings: &[(&str, &str)]) -> PublicationCard {
        PublicationCard {
            id: format!("{kind:?}-{title}"),
            kind,
            scope: CardScope::NewPage,
            source: None,
            title: title.into(),
            settings: settings.iter().map(|(k, v)| ((*k).to_string(), serde_json::json!(v))).collect(),
        }
    }
}
