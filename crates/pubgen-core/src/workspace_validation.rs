//! Normalized workspace validation and diagnostics.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use crate::excel::IngestReport;
use crate::workspace::{ProjectManifest, PublicationMetadata};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity { Warning, Error }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDiagnostic { pub code: &'static str, pub severity: ValidationSeverity, pub message: String }

#[derive(Debug, Clone)]
pub struct WorkspaceDraft { pub manifest: ProjectManifest, pub metadata: PublicationMetadata, pub glossary: crate::workspace::Glossary, pub csv: Vec<(String, Vec<u8>)>, pub warnings: Vec<String> }

impl WorkspaceDraft {
    pub fn from_ingest(project_name: String, report: IngestReport) -> Self {
        Self { manifest: ProjectManifest { format_version: crate::workspace::PUBGEN_FORMAT_VERSION, name: project_name, template: false, slots: Vec::new() }, metadata: report.metadata, glossary: crate::workspace::Glossary::default(), csv: report.csv, warnings: report.warnings }
    }
    pub fn validate(&self) -> Vec<WorkspaceDiagnostic> {
        let mut out = Vec::new();
        if let Err(message) = self.manifest.validate() { out.push(WorkspaceDiagnostic { code: "manifest.invalid", severity: ValidationSeverity::Error, message }); }
        if let Err(message) = self.metadata.validate() { out.push(WorkspaceDiagnostic { code: "metadata.invalid", severity: ValidationSeverity::Error, message }); }
        let sources: BTreeSet<&str> = self.csv.iter().map(|(name, _)| name.as_str()).collect();
        for card in &self.metadata.cards {
            if let Some(source) = &card.source && !sources.contains(source.as_str()) {
                out.push(WorkspaceDiagnostic { code: "card.source_missing", severity: ValidationSeverity::Error, message: format!("card {} references missing source {source}", card.id) });
            }
        }
        for warning in &self.warnings { out.push(WorkspaceDiagnostic { code: "ingest.warning", severity: ValidationSeverity::Warning, message: warning.clone() }); }
        out
    }
    pub fn is_valid(&self) -> bool { !self.validate().iter().any(|item| item.severity == ValidationSeverity::Error) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{CardKind, CardScope, Language, PageLayout, PublicationCard};
    use std::collections::BTreeMap;
    fn report(card_source: Option<&str>, csv: Vec<(&str, &[u8])>) -> IngestReport {
        IngestReport { profile_id: "bps_brs".into(), metadata: PublicationMetadata { profile_id: "bps_brs".into(), layout: PageLayout::Double, language: Language::Id, title: "T".into(), title_en: None, cards: vec![PublicationCard { id: "c1".into(), kind: CardKind::Table, scope: CardScope::NewPage, source: card_source.map(str::to_owned), title: "C".into(), settings: BTreeMap::new() }], extra: BTreeMap::new() }, csv: csv.into_iter().map(|(name, bytes)| (name.into(), bytes.to_vec())).collect(), warnings: Vec::new() }
    }
    #[test] fn missing_source_is_error() { let draft = WorkspaceDraft::from_ingest("x".into(), report(Some("T1.csv"), vec![])); assert!(!draft.is_valid()); assert_eq!(draft.validate()[0].code, "card.source_missing"); }
    #[test] fn matching_source_is_valid() { let draft = WorkspaceDraft::from_ingest("x".into(), report(Some("T1.csv"), vec![("T1.csv", b"x") ])); assert!(draft.is_valid()); }
}
