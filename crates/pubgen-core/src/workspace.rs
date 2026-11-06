//! Canonical Orchestrator publication workspace schema.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PUBGEN_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub format_version: u32,
    pub name: String,
    #[serde(default)]
    pub template: bool,
    #[serde(default)]
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    pub id: String,
    #[serde(default)]
    pub word_budget: Option<u32>,
    #[serde(default)]
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationMetadata {
    pub profile_id: String,
    pub layout: PageLayout,
    pub language: Language,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub title_en: Option<String>,
    #[serde(default)]
    pub cards: Vec<PublicationCard>,
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageLayout { Single, Double }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language { Id, En, IdEn }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationCard {
    pub id: String,
    pub kind: CardKind,
    pub scope: CardScope,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub settings: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind { Cover, Toc, Table, Chart, MapChart, Narrative, Image, Divider }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardScope { NewPage, NewObject }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Glossary { #[serde(default)] pub entries: Vec<GlossaryEntry> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry { pub source: String, pub target: String }

impl ProjectManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.format_version == 0 || self.format_version > PUBGEN_FORMAT_VERSION {
            return Err(format!("unsupported pubgen format version: {}", self.format_version));
        }
        if self.name.trim().is_empty() { return Err("project name must not be empty".into()); }
        for slot in &self.slots {
            if slot.id.trim().is_empty() { return Err("slot id must not be empty".into()); }
        }
        Ok(())
    }
}

impl PublicationMetadata {
    pub fn validate(&self) -> Result<(), String> {
        if self.profile_id.trim().is_empty() { return Err("profile_id must not be empty".into()); }
        let mut ids = std::collections::HashSet::new();
        for card in &self.cards {
            if card.id.trim().is_empty() || !ids.insert(&card.id) { return Err("card ids must be unique and non-empty".into()); }
            if let Some(source) = &card.source
                && (source.starts_with('/') || source.contains("..") || source.contains('\\'))
            {
                return Err(format!("invalid internal source path: {source}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_future_version() {
        let manifest = ProjectManifest { format_version: PUBGEN_FORMAT_VERSION + 1, name: "x".into(), template: false, slots: Vec::new() };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_cards() {
        let card = PublicationCard { id: "c1".into(), kind: CardKind::Table, scope: CardScope::NewPage, source: None, title: String::new(), settings: BTreeMap::new() };
        let metadata = PublicationMetadata { profile_id: "bps_brs".into(), layout: PageLayout::Single, language: Language::Id, title: String::new(), title_en: None, cards: vec![card.clone(), card], extra: BTreeMap::new() };
        assert!(metadata.validate().is_err());
    }
}
