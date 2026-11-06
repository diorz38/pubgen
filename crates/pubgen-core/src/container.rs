//! Portable `.pubgen` ZIP container.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use crate::workspace::{Glossary, ProjectManifest, PublicationMetadata};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::{Component, Path};
use zip::write::SimpleFileOptions;

const MAX_ENTRIES: usize = 10_000;
const MAX_ENTRY_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid container: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Default)]
pub struct ProjectContent {
    pub csv: Vec<(String, Vec<u8>)>,
    pub md: Vec<(String, Vec<u8>)>,
    pub assets: Vec<(String, Vec<u8>)>,
    pub template: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct OpenedProject {
    pub manifest: ProjectManifest,
    pub metadata: PublicationMetadata,
    pub glossary: Glossary,
    pub content: ProjectContent,
}

pub fn save_pubgen(
    path: &Path,
    manifest: &ProjectManifest,
    metadata: &PublicationMetadata,
    glossary: &Glossary,
    content: &ProjectContent,
) -> Result<(), ContainerError> {
    manifest.validate().map_err(ContainerError::Invalid)?;
    metadata.validate().map_err(ContainerError::Invalid)?;
    let temp = path.with_extension("pubgen.tmp");
    let file = std::fs::File::create(&temp)?;
    let mut writer = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut names = HashSet::new();
    reserve_name(&mut names, "pubgen.json")?;
    write_json(&mut writer, "pubgen.json", manifest, options)?;
    reserve_name(&mut names, "metadata.json")?;
    write_json(&mut writer, "metadata.json", metadata, options)?;
    reserve_name(&mut names, "glossary.json")?;
    write_json(&mut writer, "glossary.json", glossary, options)?;
    for (prefix, entries) in [("csv", &content.csv), ("md", &content.md), ("assets", &content.assets)] {
        for (name, bytes) in entries {
            let path = internal_path(prefix, name)?;
            reserve_name(&mut names, &path)?;
            if bytes.len() as u64 > MAX_ENTRY_BYTES {
                return Err(ContainerError::Invalid(format!("entry too large: {path}")));
            }
            writer.start_file(path, options)?;
            writer.write_all(bytes)?;
        }
    }
    if let Some(bytes) = &content.template {
        reserve_name(&mut names, "template.designcraft")?;
        if bytes.len() as u64 > MAX_ENTRY_BYTES {
            return Err(ContainerError::Invalid("template too large".into()));
        }
        writer.start_file("template.designcraft", options)?;
        writer.write_all(bytes)?;
    }
    writer.finish()?;
    replace_file(&temp, path)?;
    Ok(())
}

pub fn open_pubgen(path: &Path) -> Result<OpenedProject, ContainerError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    if archive.len() > MAX_ENTRIES {
        return Err(ContainerError::Invalid("too many entries".into()));
    }
    let manifest: ProjectManifest = read_json(&mut archive, "pubgen.json")?;
    let metadata: PublicationMetadata = read_json(&mut archive, "metadata.json")?;
    let glossary: Glossary = read_json(&mut archive, "glossary.json")?;
    manifest.validate().map_err(ContainerError::Invalid)?;
    metadata.validate().map_err(ContainerError::Invalid)?;
    let mut content = ProjectContent::default();
    let mut names = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_string();
        if name.ends_with('/') || name == "pubgen.json" || name == "metadata.json" || name == "glossary.json" || name == "template.designcraft" {
            continue;
        }
        validate_internal_path(&name)?;
        if !names.insert(name.clone()) {
            return Err(ContainerError::Invalid(format!("duplicate entry: {name}")));
        }
        let size = entry.size();
        if size > MAX_ENTRY_BYTES {
            return Err(ContainerError::Invalid(format!("entry too large: {name}")));
        }
        let mut bytes = Vec::with_capacity(size as usize);
        entry.read_to_end(&mut bytes)?;
        if let Some(name) = name.strip_prefix("csv/") {
            content.csv.push((name.to_string(), bytes));
        } else if let Some(name) = name.strip_prefix("md/") {
            content.md.push((name.to_string(), bytes));
        } else if let Some(name) = name.strip_prefix("assets/") {
            content.assets.push((name.to_string(), bytes));
        }
    }
    Ok(OpenedProject { manifest, metadata, glossary, content })
}

fn reserve_name(names: &mut HashSet<String>, name: &str) -> Result<(), ContainerError> {
    if !names.insert(name.to_owned()) {
        return Err(ContainerError::Invalid(format!("duplicate entry: {name}")));
    }
    Ok(())
}
fn write_json<T: serde::Serialize>(
    writer: &mut zip::ZipWriter<std::fs::File>,
    name: &str,
    value: &T,
    options: SimpleFileOptions,
) -> Result<(), ContainerError> {
    writer.start_file(name, options)?;
    writer.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn read_json<T: serde::de::DeserializeOwned>(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Result<T, ContainerError> {
    let mut file = archive.by_name(name)?;
    if file.size() > MAX_ENTRY_BYTES {
        return Err(ContainerError::Invalid(format!("entry too large: {name}")));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(serde_json::from_slice(&bytes)?)
}
fn internal_path(prefix: &str, name: &str) -> Result<String, ContainerError> {
    validate_internal_path(name)?;
    if name.is_empty() {
        return Err(ContainerError::Invalid("empty entry name".into()));
    }
    Ok(format!("{prefix}/{name}"))
}
fn validate_internal_path(path: &str) -> Result<(), ContainerError> {
    let p = Path::new(path);
    if p.is_absolute() || path.contains('\\') || p.components().any(|c| matches!(c, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(ContainerError::Invalid(format!("invalid internal path: {path}")));
    }
    Ok(())
}
fn replace_file(temp: &Path, target: &Path) -> Result<(), ContainerError> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if target.exists() {
        std::fs::remove_file(target)?;
    }
    std::fs::rename(temp, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{CardKind, CardScope, Language, PageLayout, PublicationCard};
    use std::collections::BTreeMap;

    #[test]
    fn rejects_duplicate_content_on_save() {
        let dir = tempfile::tempdir().expect("test tempdir");
        let path = dir.path().join("duplicate.pubgen");
        let manifest = ProjectManifest { format_version: 1, name: "X".into(), template: false, slots: vec![] };
        let metadata = PublicationMetadata {
            profile_id: "bps_brs".into(),
            layout: PageLayout::Single,
            language: Language::Id,
            title: "X".into(),
            title_en: None,
            cards: vec![],
            extra: BTreeMap::new(),
        };
        let content = ProjectContent { csv: vec![("x.csv".into(), vec![1]), ("x.csv".into(), vec![2])], ..ProjectContent::default() };
        assert!(save_pubgen(&path, &manifest, &metadata, &Glossary::default(), &content).is_err());
    }

    #[test]
    fn rejects_parent_paths() {
        let dir = tempfile::tempdir().expect("test tempdir");
        let path = dir.path().join("unsafe.pubgen");
        let manifest = ProjectManifest { format_version: 1, name: "X".into(), template: false, slots: vec![] };
        let metadata = PublicationMetadata {
            profile_id: "bps_brs".into(),
            layout: PageLayout::Single,
            language: Language::Id,
            title: "X".into(),
            title_en: None,
            cards: vec![],
            extra: BTreeMap::new(),
        };
        let content = ProjectContent { csv: vec![("../x.csv".into(), vec![1])], ..ProjectContent::default() };
        assert!(save_pubgen(&path, &manifest, &metadata, &Glossary::default(), &content).is_err());
    }

    #[test]
    fn round_trip_container() {
        let dir = tempfile::tempdir().expect("test tempdir");
        let path = dir.path().join("x.pubgen");
        let manifest = ProjectManifest { format_version: 1, name: "X".into(), template: false, slots: vec![] };
        let metadata = PublicationMetadata {
            profile_id: "bps_brs".into(),
            layout: PageLayout::Single,
            language: Language::Id,
            title: "X".into(),
            title_en: None,
            cards: vec![PublicationCard {
                id: "t1".into(),
                kind: CardKind::Table,
                scope: CardScope::NewPage,
                source: Some("T1.csv".into()),
                title: "T".into(),
                settings: BTreeMap::new(),
            }],
            extra: BTreeMap::new(),
        };
        let mut content = ProjectContent::default();
        content.csv.push(("T1.csv".into(), b"a,b\n1,2\n".to_vec()));
        save_pubgen(&path, &manifest, &metadata, &Glossary::default(), &content).expect("save");
        let opened = open_pubgen(&path).expect("open");
        assert_eq!(opened.content.csv.len(), 1);
    }
}
