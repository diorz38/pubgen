//! Portable `.pubgen` ZIP container.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use crate::translation::TranslationMemory;
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
    /// Narrative text files: RTF (`rtf/<stem>.rtf`) in new packages, Markdown (`md/<stem>.md`) in
    /// ones written before the narrative format went back to RTF.
    pub text: Vec<(String, Vec<u8>)>,
    pub assets: Vec<(String, Vec<u8>)>,
    pub template: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct OpenedProject {
    pub manifest: ProjectManifest,
    pub metadata: PublicationMetadata,
    pub glossary: Glossary,
    pub tm: TranslationMemory,
    pub content: ProjectContent,
}

pub fn save_pubgen(
    path: &Path,
    manifest: &ProjectManifest,
    metadata: &PublicationMetadata,
    glossary: &Glossary,
    tm: &TranslationMemory,
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
    reserve_name(&mut names, "translation_memory.json")?;
    write_json(&mut writer, "translation_memory.json", tm, options)?;
    for (prefix, entries) in [("csv", &content.csv), ("text", &content.text), ("assets", &content.assets)] {
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
    // Optional so packages written before the translation memory existed still open.
    let mut tm: TranslationMemory = read_json_opt(&mut archive, "translation_memory.json")?.unwrap_or_default();
    // Untrusted input: a hostile file must not make the memory (and every save) unbounded.
    tm.truncate_to_cap();
    manifest.validate().map_err(ContainerError::Invalid)?;
    metadata.validate().map_err(ContainerError::Invalid)?;
    let mut content = ProjectContent::default();
    let mut names = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_string();
        if name.ends_with('/')
            || name == "pubgen.json"
            || name == "metadata.json"
            || name == "glossary.json"
            || name == "translation_memory.json"
            || name == "template.designcraft"
        {
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
        } else if let Some(name) = name.strip_prefix("text/").or_else(|| name.strip_prefix("md/")) {
            // `md/` is how packages written before the RTF change stored narrative text.
            content.text.push((name.to_string(), bytes));
        } else if let Some(name) = name.strip_prefix("assets/") {
            content.assets.push((name.to_string(), bytes));
        }
    }
    Ok(OpenedProject { manifest, metadata, glossary, tm, content })
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
/// `Ok(None)` when the entry is absent, so packages written before the entry existed still open.
fn read_json_opt<T: serde::de::DeserializeOwned>(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Result<Option<T>, ContainerError> {
    let mut file = match archive.by_name(name) {
        Ok(file) => file,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if file.size() > MAX_ENTRY_BYTES {
        return Err(ContainerError::Invalid(format!("entry too large: {name}")));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
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
        assert!(save_pubgen(&path, &manifest, &metadata, &Glossary::default(), &TranslationMemory::default(), &content).is_err());
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
        assert!(save_pubgen(&path, &manifest, &metadata, &Glossary::default(), &TranslationMemory::default(), &content).is_err());
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
        save_pubgen(&path, &manifest, &metadata, &Glossary::default(), &TranslationMemory::default(), &content).expect("save");
        let opened = open_pubgen(&path).expect("open");
        assert_eq!(opened.content.csv.len(), 1);
    }

    #[test]
    fn round_trip_narrative_text() {
        let dir = tempfile::tempdir().expect("test tempdir");
        let path = dir.path().join("x.pubgen");
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
        let mut content = ProjectContent::default();
        // New artifacts are RTF inside `rtf/`; a name that carries a folder must come back as it
        // went in, because a card's `source` points at exactly that name.
        content.text.push(("rtf/U1.rtf".into(), b"{\\rtf1\\qc A}".to_vec()));
        content.text.push(("POINT.rtf".into(), b"{\\rtf1 B}".to_vec()));
        save_pubgen(&path, &manifest, &metadata, &Glossary::default(), &TranslationMemory::default(), &content).expect("save");
        let opened = open_pubgen(&path).expect("open");
        let names: Vec<&str> = opened.content.text.iter().map(|(name, _)| name.as_str()).collect();
        assert!(names.contains(&"rtf/U1.rtf"), "narrative artifacts keep their folder: {names:?}");
        assert!(names.contains(&"POINT.rtf"), "a bare narrative name stays bare: {names:?}");
        let rtf = opened.content.text.iter().find(|(name, _)| name == "rtf/U1.rtf").map(|(_, bytes)| bytes.clone());
        assert_eq!(rtf.as_deref(), Some(b"{\\rtf1\\qc A}".as_slice()), "the RTF bytes survive the round trip");
    }

    use crate::translation::{TmEntry, TmOrigin, now_stamp};

    fn meta(title: &str) -> PublicationMetadata {
        PublicationMetadata {
            profile_id: "bps_dda".into(),
            layout: PageLayout::Single,
            language: Language::Id,
            title: title.into(),
            title_en: None,
            cards: Vec::new(),
            extra: Default::default(),
        }
    }

    /// Rewrite the archive omitting one entry, simulating a package written before it existed.
    fn rewrite_zip_without(path: &Path, skip: &str) {
        let file = std::fs::File::open(path).expect("open zip");
        let mut archive = zip::ZipArchive::new(file).expect("zip");
        let tmp = path.with_extension("pubgen.rewrite");
        let out = std::fs::File::create(&tmp).expect("create");
        let mut writer = zip::ZipWriter::new(out);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("entry");
            if entry.is_dir() || entry.name() == skip {
                continue;
            }
            writer.start_file(entry.name().to_string(), options).expect("start entry");
            std::io::copy(&mut entry, &mut writer).expect("copy entry");
        }
        writer.finish().expect("finish");
        drop(archive);
        std::fs::remove_file(path).expect("remove old");
        std::fs::rename(&tmp, path).expect("rename");
    }

    #[test]
    fn tm_round_trips_through_the_package() {
        let dir = std::env::temp_dir().join(format!("pubgen-tm-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("tm.pubgen");
        let manifest = ProjectManifest { format_version: 1, name: "TM test".into(), template: false, slots: Vec::new() };
        let mut tm = TranslationMemory::default();
        tm.insert(TmEntry {
            source: "Kecamatan".into(),
            target: "Subdistrict".into(),
            origin: TmOrigin::Reviewed,
            role: "table_header".into(),
            updated: now_stamp(),
        });
        save_pubgen(&path, &manifest, &meta("TM test"), &Glossary::default(), &tm, &ProjectContent::default()).expect("save");
        let opened = open_pubgen(&path).expect("open");
        assert_eq!(opened.tm.lookup("Kecamatan").map(|e| e.target.as_str()), Some("Subdistrict"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_over_cap_memory_is_truncated_on_open() {
        // A hostile package may deserialize more entries than `insert` ever accepts, so open
        // must bound the memory it hands back (and every later save).
        let dir = std::env::temp_dir().join(format!("pubgen-tm-cap-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("cap.pubgen");
        let manifest = ProjectManifest { format_version: 1, name: "TM cap".into(), template: false, slots: Vec::new() };
        let over = TranslationMemory {
            entries: (0..=crate::translation::MAX_ENTRIES)
                .map(|i| TmEntry {
                    source: format!("s{i}"),
                    target: "t".into(),
                    origin: TmOrigin::Ai,
                    role: "table_header".into(),
                    updated: now_stamp(),
                })
                .collect(),
        };
        save_pubgen(&path, &manifest, &meta("TM cap"), &Glossary::default(), &over, &ProjectContent::default()).expect("save");
        let opened = open_pubgen(&path).expect("open");
        assert_eq!(opened.tm.entries.len(), crate::translation::MAX_ENTRIES);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_package_without_a_tm_entry_opens_with_an_empty_tm() {
        // Backwards compatibility: packages written before this change have no
        // translation_memory.json and must still open.
        let dir = std::env::temp_dir().join(format!("pubgen-notm-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("old.pubgen");
        let manifest = ProjectManifest { format_version: 1, name: "Old".into(), template: false, slots: Vec::new() };
        save_pubgen(&path, &manifest, &meta("Old"), &Glossary::default(), &TranslationMemory::default(), &ProjectContent::default()).expect("save");
        // Remove the entry to simulate an older package.
        rewrite_zip_without(&path, "translation_memory.json");
        let opened = open_pubgen(&path).expect("old package still opens");
        assert!(opened.tm.entries.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }
}
