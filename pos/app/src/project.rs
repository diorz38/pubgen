use crate::metadata_form::{Metadata, PageEntry};
use crate::slots::Slot;
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const MANIFEST: &str = "pubgen.json";
pub const METADATA: &str = "metadata.json";
pub const TEMPLATE: &str = "template.designcraft";
pub const GLOSSARY: &str = "glossary.json";
pub const FORMAT_VERSION: u64 = 1;

#[derive(Serialize, Deserialize)]
struct Manifest {
    format_version: u64,
    name: String,
    #[serde(default)]
    template: bool,
    #[serde(default)]
    slots: Vec<Slot>,
}

#[derive(Clone, Default)]
pub struct Project {
    pub name: String,
    pub metadata: Metadata,
    pub content: Vec<(String, Vec<u8>)>,
    pub template: Option<Vec<u8>>,
    pub slots: Vec<Slot>,
    pub glossary: crate::glossary::Glossary,
    pub extras: Vec<(String, Vec<u8>)>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Project {
        Project { name: name.into(), ..Default::default() }
    }

    pub fn add_content(&mut self, entry: String, bytes: Vec<u8>) {
        self.content.retain(|(p, _)| p != &entry);
        self.content.push((entry, bytes));
        self.content.sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub fn page_source(&self, entry: &PageEntry) -> Option<&(String, Vec<u8>)> {
        let name = entry.source.as_deref()?;
        if let Some(hit) = self.content.iter().find(|(n, _)| n == name) {
            return Some(hit);
        }
        // Fallback: bind by file stem, so an Excel link "U1.docx" resolves the
        // separately imported / converted "content/U1.md".
        let stem = name
            .rsplit(['/', '\\'])
            .next()?
            .rsplit_once('.')
            .map(|(s, _)| s.to_lowercase())?;
        self.content.iter().find(|(n, _)| {
            n.rsplit(['/', '\\'])
                .next()
                .and_then(|f| f.rsplit_once('.'))
                .is_some_and(|(s, _)| s.eq_ignore_ascii_case(&stem))
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let manifest = Manifest {
            format_version: FORMAT_VERSION,
            name: self.name.clone(),
            template: self.template.is_some(),
            slots: self.slots.clone(),
        };
        let manifest_bytes = serde_json::to_string_pretty(&manifest)?.into_bytes();
        let metadata_bytes = serde_json::to_string_pretty(&self.metadata.to_json())?.into_bytes();

        let tmp = tmp_path(path);
        let file = std::fs::File::create(&tmp)
            .with_context(|| format!("create {}", tmp.display()))?;
        let mut w = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let mut write_entry = |name: &str, bytes: &[u8]| -> Result<()> {
            w.start_file(name, opts)
                .with_context(|| format!("zip entry {name}"))?;
            w.write_all(bytes).context("zip write")?;
            Ok(())
        };
        write_entry(MANIFEST, &manifest_bytes)?;
        write_entry(METADATA, &metadata_bytes)?;
        let glossary_bytes =
            serde_json::to_string_pretty(&self.glossary).context("glossary json")?.into_bytes();
        write_entry(GLOSSARY, &glossary_bytes)?;
        for (entry, bytes) in &self.content {
            write_entry(entry, bytes)?;
        }
        if let Some(template) = &self.template {
            write_entry(TEMPLATE, template)?;
        }
        for (name, bytes) in &self.extras {
            if name == MANIFEST || name == METADATA || name == TEMPLATE {
                continue;
            }
            write_entry(name, bytes)?;
        }
        w.finish().context("zip finish")?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Project> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("open {}", path.display()))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| anyhow!("{} is not a .pubgen (zip): {e}", path.display()))?;
        let mut manifest_bytes = Vec::new();
        {
            let mut f = archive
                .by_name(MANIFEST)
                .map_err(|_| anyhow!("{} has no {MANIFEST}", path.display()))?;
            f.read_to_end(&mut manifest_bytes).context("read manifest")?;
        }
        let manifest: Manifest = serde_json::from_slice(&manifest_bytes).context("bad manifest")?;
        if manifest.format_version > FORMAT_VERSION {
            bail!("{} was written by a newer pubgen (format {})", path.display(), manifest.format_version);
        }
        let mut project = Project { name: manifest.name, slots: manifest.slots, ..Default::default() };
        if let Ok(mut f) = archive.by_name(METADATA) {
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes)?;
            project.metadata = Metadata::from_json(&serde_json::from_slice::<Value>(&bytes)?);
        }
        if let Ok(mut f) = archive.by_name(TEMPLATE) {
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes)?;
            project.template = Some(bytes);
        }
        if let Ok(mut f) = archive.by_name(GLOSSARY) {
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes)?;
            project.glossary = serde_json::from_slice(&bytes).context("bad glossary")?;
        }
        for i in 0..archive.len() {
            let mut f = archive.by_index(i)?;
            let name = f.name().to_owned();
            if is_known_entry(&name) {
                if is_content_entry(&name) {
                    let mut bytes = Vec::new();
                    f.read_to_end(&mut bytes)?;
                    project.content.push((name, bytes));
                }
                continue;
            }
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes)?;
            project.extras.push((name, bytes));
        }
        project.content.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(project)
    }
}

fn is_known_entry(name: &str) -> bool {
    name == MANIFEST || name == METADATA || name == TEMPLATE || name == GLOSSARY || is_content_entry(name)
}

/// Content prefixes: `md/` + `csv/` are current; `content/` + `tables/` kept
/// so older `.pubgen` containers still open.
fn is_content_entry(name: &str) -> bool {
    name.starts_with("md/") || name.starts_with("csv/") || name.starts_with("content/") || name.starts_with("tables/")
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".tmp");
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_content_and_extras() {
        let dir = std::env::temp_dir().join(format!("pubgen-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.pubgen");

        let mut p = Project::new("DDA2026");
        p.metadata.pages.clear();
        p.metadata.title = "Catalog".into();
        p.metadata.layout = crate::metadata_form::PageLayout::Double;
        p.metadata.lang = crate::metadata_form::Lang::IdEn;
        p.metadata.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Image,
            scope: crate::metadata_form::PageItemScope::NewPage,
            title: "Cover photo".into(),
            source: Some("md/a.md".into()),
            ..Default::default()
        });
        p.add_content("md/a.md".into(), b"# Hello\n".to_vec());
        p.add_content("csv/t.csv".into(), b"a,b\r\n1,2\r\n".to_vec());
        // Legacy prefixes from older containers must still round-trip as content.
        p.add_content("content/legacy.md".into(), b"# Legacy\n".to_vec());
        p.add_content("tables/legacy.csv".into(), b"x\r\n1\r\n".to_vec());

        // Excel link "U1.docx" must resolve the converted "md/U1.md" by stem.
        p.metadata.pages.push(crate::metadata_form::PageEntry {
            kind: crate::metadata_form::PageKind::Paragraph,
            scope: crate::metadata_form::PageItemScope::NewObject,
            title: "Ulasan".into(),
            source: Some("U1.docx".into()),
            ..Default::default()
        });
        p.add_content("md/U1.md".into(), b"# Ulasan\n".to_vec());
        let linked = p.metadata.pages.last().unwrap();
        assert!(p.page_source(linked).is_some_and(|(n, _)| n == "md/U1.md"));
        p.template = Some(b"PK-TEMPLATE".to_vec());
        p.extras.push(("assets/x.bin".into(), vec![1, 2, 3]));
        p.glossary.push(crate::glossary::Entry {
            term: "quantum leap".into(),
            replacement: "lompatan kuantum".into(),
            do_not_translate: false,
            case_preserve: true,
        });
        p.slots.push(Slot { name: "slot:title".into(), frame: 10, story: 90, budget: Some(50), source: String::new(), text: "Hi".into() });
        p.save(&path).unwrap();

        let q = Project::open(&path).unwrap();
        assert_eq!(q.name, "DDA2026");
        assert_eq!(q.metadata.title, "Catalog");
        assert_eq!(q.metadata.layout, crate::metadata_form::PageLayout::Double);
        assert_eq!(q.metadata.lang, crate::metadata_form::Lang::IdEn);
        assert_eq!(q.metadata.pages.len(), 2);
        assert_eq!(q.metadata.pages[0].title, "Cover photo");
        assert_eq!(q.page_source(&q.metadata.pages[0]).map(|(n, _)| n.as_str()), Some("md/a.md"));
        let linked = q.metadata.pages.last().unwrap();
        assert!(q.page_source(linked).is_some_and(|(n, _)| n == "md/U1.md"));
        assert_eq!(q.content.len(), 5);
        assert!(q.content.iter().any(|(n, _)| n == "md/a.md"));
        assert!(q.content.iter().any(|(n, _)| n == "csv/t.csv"));
        assert!(q.content.iter().any(|(n, _)| n == "content/legacy.md"));
        assert!(q.content.iter().any(|(n, _)| n == "tables/legacy.csv"));
        assert_eq!(q.template.as_deref(), Some(&b"PK-TEMPLATE"[..]));
        assert_eq!(q.extras, vec![("assets/x.bin".to_owned(), vec![1, 2, 3])]);
        assert_eq!(q.slots[0].budget, Some(50));
        assert_eq!(q.glossary.len(), 1);
        assert_eq!(q.glossary[0].replacement, "lompatan kuantum");
        std::fs::remove_file(&path).ok();
    }
}
