//! Deterministic XLSX export from normalized publication workspaces.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use crate::workspace::{CardKind, PublicationCard};
use crate::workspace_validation::WorkspaceDraft;
use rust_xlsxwriter::{Workbook, XlsxError};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("workspace validation failed: {0}")] Validation(String),
    #[error("XLSX export failed: {0}")] Xlsx(#[from] XlsxError),
    #[error("invalid CSV source {0}: {1}")] Csv(String, String),
}

pub fn export_workbook(draft: &WorkspaceDraft, output: &Path) -> Result<(), ExportError> {
    let diagnostics = draft.validate();
    if let Some(item) = diagnostics.iter().find(|item| item.severity == crate::workspace_validation::ValidationSeverity::Error) { return Err(ExportError::Validation(item.message.clone())); }
    let mut workbook = Workbook::new();
    write_cover(&mut workbook, draft)?;
    let mut point_written = false;
    for card in &draft.metadata.cards {
        if card.kind == CardKind::Narrative && !point_written { write_point(&mut workbook, card, draft)?; point_written = true; }
        if let Some(source) = &card.source
            && let Some((_, bytes)) = draft.csv.iter().find(|(name, _)| name == source)
        {
            let name = sheet_name(card, source);
            write_csv_sheet(&mut workbook, &name, bytes)?;
        }
    }
    workbook.save(output)?;
    Ok(())
}

fn write_cover(workbook: &mut Workbook, draft: &WorkspaceDraft) -> Result<(), ExportError> {
    let sheet = workbook.add_worksheet().set_name("COVER")?;
    sheet.write_string(0, 0, "title")?.write_string(0, 1, &draft.metadata.title)?;
    if let Some(title) = &draft.metadata.title_en { sheet.write_string(1, 0, "title_en")?.write_string(1, 1, title)?; }
    for (index, (key, value)) in draft.metadata.extra.iter().enumerate() { let row = (index + 2) as u32; sheet.write_string(row, 0, key)?.write_string(row, 1, value)?; }
    Ok(())
}
fn write_point(workbook: &mut Workbook, card: &PublicationCard, draft: &WorkspaceDraft) -> Result<(), ExportError> {
    let sheet = workbook.add_worksheet().set_name("POINT")?;
    sheet.write_string(0, 0, "item")?;
    if let Some(source) = &card.source
        && let Some((_, bytes)) = draft.csv.iter().find(|(name, _)| name == source)
    {
        let rows = parse_csv(bytes, source)?;
        for (index, row) in rows.iter().enumerate() {
            if let Some(value) = row.first() {
                sheet.write_string((index + 1) as u32, 0, value)?;
            }
        }
    }
    Ok(())
}
fn write_csv_sheet(workbook: &mut Workbook, name: &str, bytes: &[u8]) -> Result<(), ExportError> { let sheet = workbook.add_worksheet().set_name(name)?; for (row_index, row) in parse_csv(bytes, name)?.iter().enumerate() { for (col_index, value) in row.iter().enumerate() { sheet.write_string(row_index as u32, col_index as u16, value)?; } } Ok(()) }
fn parse_csv(bytes: &[u8], name: &str) -> Result<Vec<Vec<String>>, ExportError> { let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(bytes); reader.records().map(|record| record.map(|row| row.iter().map(str::to_owned).collect()).map_err(|error| ExportError::Csv(name.to_owned(), error.to_string()))).collect() }
fn sheet_name(card: &PublicationCard, source: &str) -> String { let prefix = match card.kind { CardKind::Chart => "C", CardKind::MapChart => "M", CardKind::Table => "T", _ => "S" }; let stem = source.rsplit('/').next().unwrap_or(source).trim_end_matches(".csv"); let mut name = format!("{prefix}_{stem}"); name.truncate(31); name }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::excel::IngestReport;
    use crate::workspace::{CardScope, Language, PageLayout, PublicationCard, PublicationMetadata};
    use std::collections::BTreeMap;
    #[test]
    fn exports_workspace_to_xlsx() { let dir = tempfile::tempdir().expect("temp"); let output = dir.path().join("out.xlsx"); let metadata = PublicationMetadata { profile_id: "bps_brs".into(), layout: PageLayout::Double, language: Language::Id, title: "Report".into(), title_en: None, cards: vec![PublicationCard { id: "t1".into(), kind: CardKind::Table, scope: CardScope::NewPage, source: Some("T1.csv".into()), title: "Table".into(), settings: BTreeMap::new() }], extra: BTreeMap::new() }; let draft = WorkspaceDraft::from_ingest("x".into(), IngestReport { profile_id: "bps_brs".into(), metadata, csv: vec![("T1.csv".into(), b"a,b\n1,2\n".to_vec())], warnings: vec![] }); export_workbook(&draft, &output).expect("export"); assert!(output.is_file()); }
}
