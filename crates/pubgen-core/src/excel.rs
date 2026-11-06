//! Excel workbook normalization for Orchestrator workspaces.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use crate::workspace::{CardKind, CardScope, Language, PageLayout, PublicationCard, PublicationMetadata};
use calamine::{Data, Reader, open_workbook_auto};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("cannot open workbook {path}: {message}")] Open { path: String, message: String },
    #[error("cannot read worksheet {sheet}: {message}")] Sheet { sheet: String, message: String },
    #[error("invalid workbook: {0}")] Invalid(String),
}

#[derive(Debug, Clone)]
pub struct IngestReport {
    pub profile_id: String,
    pub metadata: PublicationMetadata,
    pub csv: Vec<(String, Vec<u8>)>,
    pub warnings: Vec<String>,
}

pub fn ingest_workbook(path: &Path) -> Result<IngestReport, IngestError> {
    let mut workbook = open_workbook_auto(path).map_err(|source| IngestError::Open { path: path.display().to_string(), message: source.to_string() })?;
    let names = workbook.sheet_names().to_vec();
    if names.is_empty() { return Err(IngestError::Invalid("workbook has no worksheets".into())); }
    let profile_id = detect_profile(&names);
    let mut metadata = PublicationMetadata { profile_id: profile_id.to_string(), layout: if profile_id == "bps_brs" { PageLayout::Double } else { PageLayout::Single }, language: if profile_id == "bps_dda" { Language::IdEn } else { Language::Id }, title: String::new(), title_en: None, cards: Vec::new(), extra: BTreeMap::new() };
    let mut csv = Vec::new();
    let mut warnings = Vec::new();
    for name in names {
        let range = workbook.worksheet_range(&name).map_err(|source| IngestError::Sheet { sheet: name.clone(), message: source.to_string() })?;
        let rows: Vec<Vec<String>> = range.rows().map(|row| row.iter().map(cell_text).collect()).collect();
        if is_cover(&name) { parse_cover(&rows, &mut metadata, &mut warnings); continue; }
        let (kind, scope) = classify_sheet(&name, profile_id);
        let file_name = safe_sheet_name(&name);
        let source = format!("{file_name}.csv");
        csv.push((source.clone(), rows_to_csv(&rows)?));
        metadata.cards.push(PublicationCard { id: file_name.clone(), kind, scope, source: Some(source), title: first_nonempty_row(&rows).unwrap_or_default(), settings: BTreeMap::new() });
    }
    Ok(IngestReport { profile_id: profile_id.to_string(), metadata, csv, warnings })
}

fn detect_profile(names: &[String]) -> &'static str {
    let upper: Vec<String> = names.iter().map(|n| n.to_ascii_uppercase()).collect();
    if upper.iter().any(|n| n == "POINT" || n == "RINGKASAN" || n.starts_with('U') && n.len() == 2) { "bps_brs" } else if upper.iter().any(|n| n.contains('.') || n.starts_with("BAB")) { "bps_dda" } else { "generic_report" }
}
fn is_cover(name: &str) -> bool { matches!(name.to_ascii_uppercase().as_str(), "COVER" | "COV" | "SAMPUL") }
fn classify_sheet(name: &str, profile: &str) -> (CardKind, CardScope) {
    let upper = name.to_ascii_uppercase();
    if profile == "bps_dda" {
        if upper.starts_with('M') { return (CardKind::MapChart, CardScope::NewPage); }
        if upper.starts_with('G') { return (CardKind::Image, CardScope::NewPage); }
        if upper.starts_with('A') || upper.starts_with('R') { return (CardKind::Narrative, CardScope::NewPage); }
        if is_dda_table_name(&upper) { return (CardKind::Table, CardScope::NewPage); }
    }
    if profile == "bps_brs" && (upper == "POINT" || upper == "RINGKASAN") { (CardKind::Narrative, CardScope::NewPage) }
    else if profile == "bps_brs" && upper.starts_with('U') { (CardKind::Narrative, if upper == "U1" { CardScope::NewPage } else { CardScope::NewObject }) }
    else if upper.starts_with('C') { (CardKind::Chart, CardScope::NewObject) }
    else if upper.starts_with('M') { (CardKind::MapChart, CardScope::NewObject) }
    else { (CardKind::Table, if profile == "bps_dda" { CardScope::NewPage } else { CardScope::NewObject }) }
}

fn is_dda_table_name(name: &str) -> bool {
    let mut parts = name.split('_');
    let Some(left) = parts.next() else { return false };
    let Some(right) = parts.next() else { return false };
    parts.next().is_none() && left.len() == 2 && right.len() >= 2 && left.chars().all(|c| c.is_ascii_digit()) && right.chars().all(|c| c.is_ascii_digit())
}
fn parse_cover(rows: &[Vec<String>], metadata: &mut PublicationMetadata, warnings: &mut Vec<String>) {
    for row in rows {
        if row.len() < 2 { continue; }
        let key = row[0].trim().to_ascii_lowercase(); let value = row[1].trim();
        if value.is_empty() { continue; }
        match key.as_str() { "title" | "title1" => metadata.title = value.to_string(), "title_en" | "title2" => metadata.title_en = Some(value.to_string()), _ => { metadata.extra.insert(key, value.to_string()); } }
    }
    if metadata.title.is_empty() { warnings.push("COVER sheet has no title field".into()); }
}
fn cell_text(cell: &Data) -> String { match cell { Data::Empty => String::new(), Data::String(v) => v.clone(), Data::Float(v) => v.to_string(), Data::Int(v) => v.to_string(), Data::Bool(v) => v.to_string(), Data::DateTime(v) => v.to_string(), Data::DateTimeIso(v) | Data::DurationIso(v) => v.clone(), Data::Error(v) => format!("#ERROR:{v:?}") } }
fn rows_to_csv(rows: &[Vec<String>]) -> Result<Vec<u8>, IngestError> { let mut writer = csv::WriterBuilder::new().from_writer(Vec::new()); for row in rows { writer.write_record(row).map_err(|e| IngestError::Invalid(format!("CSV conversion failed: {e}")))?; } writer.into_inner().map_err(|e| IngestError::Invalid(format!("CSV finalize failed: {e}"))) }
fn safe_sheet_name(name: &str) -> String { name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect() }
fn first_nonempty_row(rows: &[Vec<String>]) -> Option<String> { rows.iter().find_map(|row| row.iter().find(|cell| !cell.trim().is_empty()).cloned()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn profiles_follow_sheet_names() { assert_eq!(detect_profile(&["POINT".into(), "U1".into()]), "bps_brs"); assert_eq!(detect_profile(&["1.1.1".into()]), "bps_dda"); }
    #[test] fn csv_escapes_values() { let bytes = rows_to_csv(&[vec!["a,b".into(), "x\n y".into()]]).expect("csv"); assert_eq!(String::from_utf8(bytes).expect("utf8"), "\"a,b\",\"x\n y\"\n"); }
}
