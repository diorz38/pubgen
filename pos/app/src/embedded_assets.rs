use std::io::{Cursor, Read};
use std::path::PathBuf;
use zip::ZipArchive;

static TEMPLATES_ZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/embedded_templates.zip"));
static MAPS_ZIP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/embedded_maps.zip"));

/// Retrieves the raw bytes of an embedded asset (templates or maps).
pub fn get_embedded_asset(path: &str) -> Option<Vec<u8>> {
    let clean = path.replace('\\', "/");
    let norm = clean.strip_prefix("assets/").unwrap_or(&clean);

    if norm.starts_with("templates/") {
        let sub = norm.strip_prefix("templates/").unwrap_or(norm);
        if let Some(bytes) = read_from_zip(TEMPLATES_ZIP, sub) {
            return Some(bytes);
        }
    }
    if norm.starts_with("maps/") {
        let sub = norm.strip_prefix("maps/").unwrap_or(norm);
        if let Some(bytes) = read_from_zip(MAPS_ZIP, sub) {
            return Some(bytes);
        }
    }
    // Direct lookup in both archives
    if let Some(bytes) = read_from_zip(TEMPLATES_ZIP, norm) {
        return Some(bytes);
    }
    read_from_zip(MAPS_ZIP, norm)
}

/// Ensures an embedded asset is available on disk for external processes
/// (like `designcraft-cli.exe` or file pickers).
/// Writes to `%TEMP%/pubgen_assets/<path>` and returns the path.
pub fn ensure_asset_on_disk(path: &str) -> Option<PathBuf> {
    let bytes = get_embedded_asset(path)?;
    let clean = path.replace('\\', "/");
    let target = std::env::temp_dir().join("pubgen_assets").join(&clean);
    if let Some(parent) = target.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let needs_write = match std::fs::metadata(&target) {
        Ok(m) => m.len() != bytes.len() as u64,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(&target, &bytes).ok()?;
    }
    Some(target)
}

fn read_from_zip(zip_bytes: &[u8], entry_name: &str) -> Option<Vec<u8>> {
    let cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(cursor).ok()?;
    let clean_entry = entry_name.replace('\\', "/");
    for i in 0..archive.len() {
        if let Ok(mut file) = archive.by_index(i) {
            let name = file.name().replace('\\', "/");
            if name.eq_ignore_ascii_case(&clean_entry) {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).ok()?;
                return Some(buf);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_templates_present() {
        let tpl = get_embedded_asset("assets/templates/covers/brs/brs.designcraft");
        assert!(tpl.is_some(), "brs.designcraft should be found in embedded templates");
        assert!(!tpl.unwrap().is_empty());
    }

    #[test]
    fn test_embedded_maps_present() {
        let map = get_embedded_asset("assets/maps/CIAMPEL.geojson");
        assert!(map.is_some(), "CIAMPEL.geojson should be found in embedded maps");
        assert!(!map.unwrap().is_empty());
    }

    #[test]
    fn test_ensure_asset_on_disk() {
        let path = ensure_asset_on_disk("assets/templates/covers/007_modern_horizon.designcraft");
        assert!(path.is_some(), "should extract to temp disk");
        let p = path.unwrap();
        assert!(p.exists());
    }
}

