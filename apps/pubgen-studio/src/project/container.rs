//! Project container reader and writer for `.pubgen` ZIP files.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde_json::Value;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub const MANIFEST_NAME: &str = "pubgen.json";
pub const METADATA_NAME: &str = "metadata.json";
pub const GLOSSARY_NAME: &str = "glossary.json";
pub const CACHED_LAYOUT_NAME: &str = "output/publication.designcraft";

/// An in-memory representation of a `.pubgen` project container.
#[derive(Clone, Debug, Default)]
pub struct PubgenContainer {
    pub name: String,
    pub path: Option<PathBuf>,
    pub metadata: Value,
    pub manifest: Option<Value>,
    pub glossary: Option<Value>,
    pub content: Vec<(String, Vec<u8>)>,
    pub cached_layout: Option<Vec<u8>>,
    pub extras: Vec<(String, Vec<u8>)>,
}

impl PubgenContainer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            metadata: Value::Object(Default::default()),
            ..Default::default()
        }
    }

    /// Open and unpack an existing `.pubgen` ZIP container.
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("Cannot open {}: {e}", path.display()))?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Invalid .pubgen zip archive {}: {e}", path.display()))?;

        let mut container = Self {
            name: path.file_stem().and_then(|s| s.to_str()).unwrap_or("project").to_string(),
            path: Some(path.to_path_buf()),
            ..Default::default()
        };

        // Read manifest if present
        if let Ok(mut f) = archive.by_name(MANIFEST_NAME) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                container.manifest = serde_json::from_slice(&buf).ok();
            }
        }

        // Read metadata if present
        if let Ok(mut f) = archive.by_name(METADATA_NAME) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                container.metadata = serde_json::from_slice(&buf).unwrap_or(Value::Object(Default::default()));
            }
        }

        // Read glossary if present
        if let Ok(mut f) = archive.by_name(GLOSSARY_NAME) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                container.glossary = serde_json::from_slice(&buf).ok();
            }
        }

        // Read cached compiled layout if bundled
        if let Ok(mut f) = archive.by_name(CACHED_LAYOUT_NAME) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                container.cached_layout = Some(buf);
            }
        }

        // Read content and extras
        for i in 0..archive.len() {
            if let Ok(mut f) = archive.by_index(i) {
                let name = f.name().to_string();
                if name == MANIFEST_NAME || name == METADATA_NAME || name == GLOSSARY_NAME || name == CACHED_LAYOUT_NAME {
                    continue;
                }
                let mut bytes = Vec::new();
                if f.read_to_end(&mut bytes).is_ok() {
                    if is_content_entry(&name) {
                        container.content.push((name, bytes));
                    } else {
                        container.extras.push((name, bytes));
                    }
                }
            }
        }

        // Also check if a sidecar .designcraft exists next to this file if no cached layout inside
        if container.cached_layout.is_none() {
            let sidecar = path.with_extension("designcraft");
            if sidecar.is_file() {
                if let Ok(bytes) = std::fs::read(&sidecar) {
                    container.cached_layout = Some(bytes);
                }
            }
        }

        Ok(container)
    }

    /// Save the container atomically by writing to `.tmp` first, then renaming.
    pub fn save(&self, destination: &Path) -> Result<(), String> {
        let tmp = tmp_path(destination);
        let file = File::create(&tmp).map_err(|e| format!("Create temp file {}: {e}", tmp.display()))?;
        let mut writer = ZipWriter::new(file);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let mut write_entry = |name: &str, data: &[u8]| -> Result<(), String> {
            writer.start_file(name, opts).map_err(|e| format!("Zip entry {name}: {e}"))?;
            writer.write_all(data).map_err(|e| format!("Zip write {name}: {e}"))?;
            Ok(())
        };

        // Write manifest
        if let Some(ref m) = self.manifest {
            let json_bytes = serde_json::to_vec_pretty(m).map_err(|e| e.to_string())?;
            write_entry(MANIFEST_NAME, &json_bytes)?;
        }

        // Write metadata
        let meta_bytes = serde_json::to_vec_pretty(&self.metadata).map_err(|e| e.to_string())?;
        write_entry(METADATA_NAME, &meta_bytes)?;

        // Write glossary
        if let Some(ref g) = self.glossary {
            let gloss_bytes = serde_json::to_vec_pretty(g).map_err(|e| e.to_string())?;
            write_entry(GLOSSARY_NAME, &gloss_bytes)?;
        }

        // Write cached layout if exists
        if let Some(ref layout_bytes) = self.cached_layout {
            write_entry(CACHED_LAYOUT_NAME, layout_bytes)?;
        }

        // Write content
        for (name, bytes) in &self.content {
            write_entry(name, bytes)?;
        }

        // Write extras
        for (name, bytes) in &self.extras {
            write_entry(name, bytes)?;
        }

        writer.finish().map_err(|e| format!("Zip finalize: {e}"))?;

        // Atomic replace
        std::fs::rename(&tmp, destination)
            .map_err(|e| format!("Rename {} -> {}: {e}", tmp.display(), destination.display()))?;

        Ok(())
    }
}

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
    fn test_container_round_trip() {
        let temp_dir = std::env::temp_dir().join(format!("pubgen_studio_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test_publication.pubgen");

        let mut container = PubgenContainer::new("Test Publication");
        container.metadata = serde_json::json!({
            "title": "Buku Statistik 2026",
            "profileId": "bps_dda"
        });
        container.content.push(("md/chapter1.md".into(), b"# Bab 1 Geografi\nData teks".to_vec()));
        container.content.push(("csv/table1.csv".into(), b"No,Kecamatan,Jumlah\n1,Kecamatan A,500".to_vec()));
        container.cached_layout = Some(b"MOCK_DESIGNCRAFT_BYTES".to_vec());

        // Save
        container.save(&file_path).expect("Failed to save container");
        assert!(file_path.exists());

        // Reopen
        let loaded = PubgenContainer::open(&file_path).expect("Failed to open container");
        assert_eq!(loaded.name, "test_publication");
        assert_eq!(loaded.metadata["title"], "Buku Statistik 2026");
        assert_eq!(loaded.content.len(), 2);
        assert_eq!(loaded.cached_layout, Some(b"MOCK_DESIGNCRAFT_BYTES".to_vec()));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
