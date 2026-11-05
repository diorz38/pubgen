//! `pubgen-export <PROJECT.pubgen> [--out MANIFEST.json] [--workbook FILE.xlsx]...`
//!
//! Converts a pos.app publication workspace (`.pubgen` zip) into a DesignCraft publication
//! compiler manifest (`pubgen_project.json`). Emits a **version 1** manifest: the publication
//! profile is named by id (one of the four built-ins the compiler bundles), and the styling the
//! pos.app panel does not persist (table style, header/footer, cover theme) is left to the
//! compiler's profile defaults — a half-remembered snapshot would drift, defaults stay honest.
//!
//! Chapters name workbooks, so pass each one with `--workbook`; paths are stored absolute so the
//! manifest compiles from any working directory.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use zip::ZipArchive;

const USAGE: &str = "usage: pubgen-export <PROJECT.pubgen> [--out MANIFEST.json] [--workbook FILE.xlsx]... [--profile ID]";
/// The publication profiles the designcraft compiler bundles (assets/pubgen/profiles).
static BUILTIN_PROFILES: [&str; 4] = ["bps_dda", "bps_brs", "academic_monograph", "corporate_annual"];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match run(&args) {
        Ok(path) => {
            println!("{}", path.display());
            0
        }
        Err(e) => {
            eprintln!("pubgen-export: {e}");
            1
        }
    };
    std::process::exit(code);
}

fn run(args: &[String]) -> Result<PathBuf, String> {
    let mut input: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut workbooks: Vec<PathBuf> = Vec::new();
    let mut profile: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                out = Some(PathBuf::from(args.get(i).filter(|s| !s.starts_with('-')).ok_or("--out needs a path")?));
            }
            "--workbook" => {
                i += 1;
                workbooks.push(PathBuf::from(args.get(i).filter(|s| !s.starts_with('-')).ok_or("--workbook needs a path")?));
            }
            "--profile" => {
                i += 1;
                profile = Some(args.get(i).filter(|s| !s.starts_with('-')).ok_or("--profile needs an id")?.clone());
            }
            a if a.starts_with('-') => return Err(format!("unknown option {a}\n{USAGE}")),
            a if input.is_none() => input = Some(PathBuf::from(a)),
            a => return Err(format!("unexpected argument {a}")),
        }
        i += 1;
    }
    let input = input.ok_or_else(|| format!("missing PROJECT.pubgen\n{USAGE}"))?;
    if !input.is_file() {
        return Err(format!("{}: no such file", input.display()));
    }

    let metadata = read_metadata(&input)?;
    let manifest = build_manifest(&metadata, &input, &workbooks, profile.as_deref())?;
    let out = out.unwrap_or_else(|| default_out(&input));
    let text = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    fs::write(&out, text).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(out)
}

/// The `metadata.json` entry of the .pubgen zip, as raw JSON.
fn read_metadata(input: &Path) -> Result<Value, String> {
    let bytes = fs::read(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let mut archive = ZipArchive::new(std::io::Cursor::new(&bytes)).map_err(|e| format!("{}: not a .pubgen zip: {e}", input.display()))?;
    let mut text = String::new();
    archive
        .by_name("metadata.json")
        .map_err(|_| format!("{}: no metadata.json entry", input.display()))?
        .read_to_string(&mut text)
        .map_err(|e| format!("metadata.json: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("metadata.json: not valid JSON: {e}"))
}

/// `<project dir>/pubgen_project.json` — the conventional name the compiler UI expects.
fn default_out(input: &Path) -> PathBuf {
    input.parent().unwrap_or_else(|| Path::new(".")).join("pubgen_project.json")
}

fn build_manifest(meta: &Value, input: &Path, workbooks: &[PathBuf], profile_override: Option<&str>) -> Result<Value, String> {
    let profile = match profile_override {
        Some(id) => normalize_profile(id)?,
        // pos.app keeps the selected profile in App state; a missing profileId in metadata.json
        // is common. Infer: a release number or the BRS margins (18/18/20/20) mean bps_brs.
        None => {
            let id = meta.get("profileId").and_then(Value::as_str).unwrap_or("").trim().to_lowercase();
            if id.is_empty() {
                let m = meta.get("margins");
                let f = |name: &str| m.and_then(|m| m.get(name)).and_then(Value::as_f64);
                let brs_margins = f("top") == Some(18.0) && f("bottom") == Some(18.0) && f("left") == Some(20.0) && f("right") == Some(20.0);
                let release = meta.get("releaseNo").and_then(Value::as_str).is_some_and(|s| !s.trim().is_empty());
                if brs_margins || release { "bps_brs" } else { "bps_dda" }
            } else {
                normalize_profile(&id)?
            }
        }
    };
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("publication");
    let bilingual = meta.get("lang").and_then(Value::as_str) == Some("IdEn");

    if workbooks.is_empty() {
        return Err(
            "no --workbook given: the compiler builds from chapter workbooks (.xlsx), which the .pubgen zip does not carry (it stores converted csv/md). Pass the original Excel file(s) with --workbook".into(),
        );
    }
    let mut chapters = Vec::new();
    for (i, wb) in workbooks.iter().enumerate() {
        let path = if wb.is_absolute() { wb.clone() } else { std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(wb) };
        if !path.is_file() {
            return Err(format!("--workbook {}: no such file", path.display()));
        }
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.replace(['_', '-'], " "))
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        chapters.push(json!({"chapter": i + 1, "path": path.to_string_lossy(), "title": title}));
    }

    // Contents list always; tables/charts only when such pages exist, so no empty list pages.
    let kinds: Vec<&str> = meta
        .get("pages")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|p| p.get("kind").and_then(Value::as_str)).collect())
        .unwrap_or_default();
    let mut lists = vec!["contents"];
    if kinds.contains(&"Table") {
        lists.push("tables");
    }
    if kinds.contains(&"Chart") {
        lists.push("charts");
    }

    let paper = match meta.get("paper").and_then(Value::as_str) {
        Some("Letter") => "Letter",
        _ => "A4",
    };
    let facing = meta.get("layout").and_then(Value::as_str) == Some("Double");
    let m = meta.get("margins");
    let f = |name: &str, default: f64| m.and_then(|m| m.get(name)).and_then(Value::as_f64).unwrap_or(default);

    Ok(json!({
        "version": 1,
        "publicationTitle": meta.get("title").and_then(Value::as_str).filter(|t| !t.trim().is_empty()).unwrap_or(stem),
        "bilingual": bilingual,
        "profile": profile,
        "output": {"document": format!("{stem}.designcraft"), "pdf": format!("{stem}.pdf")},
        "document": {
            "preset": paper,
            "facingPages": facing,
            "bleedMm": 3.0,
            "marginsMm": {"top": f("top", 20.2), "bottom": f("bottom", 20.0), "inside": f("left", 19.0), "outside": f("right", 15.0)},
        },
        "chapters": chapters,
        "toc": {"lists": lists},
        "build": {"keepIntermediate": true, "exportPdf": true},
    }))
}

/// Map the pos.app profile id onto a compiler built-in (same ids; case and aliases tolerated).
fn normalize_profile(id: &str) -> Result<&'static str, String> {
    let clean = id.trim().to_lowercase();
    let hit = BUILTIN_PROFILES.iter().find(|p| **p == clean.as_str() || p.split('_').next_back().is_some_and(|last| last == clean));
    hit.copied().ok_or_else(|| {
        format!("profile {id:?} is not one the compiler bundles ({}); pick one of those in the pos.app profile", BUILTIN_PROFILES.join(", "))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_ids_and_aliases_resolve() {
        assert_eq!(normalize_profile("bps_dda"), Ok("bps_dda"));
        assert_eq!(normalize_profile("BPS_BRS"), Ok("bps_brs"));
        assert_eq!(normalize_profile("brs"), Ok("bps_brs")); // tail alias, like pos.app's fallback
        assert_eq!(normalize_profile("dda"), Ok("bps_dda"));
        assert!(normalize_profile("custom").is_err());
    }

    #[test]
    fn manifest_shape_matches_the_compiler_schema() {
        let dir = std::env::temp_dir().join(format!("pubgen-export-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let wb = dir.join("BAB_01.xlsx");
        std::fs::write(&wb, b"xlsx").unwrap();
        let meta = serde_json::json!({
            "profileId": "bps_brs", "lang": "Id", "layout": "Double", "paper": "A4",
            "title": "Rilis NTP", "margins": {"top": 18.0, "bottom": 18.0, "left": 20.0, "right": 20.0},
            "pages": [
                {"kind": "Cover", "scope": "NewPage"},
                {"kind": "Table", "scope": "NewPage", "source": "csv/T1.csv"},
                {"kind": "Chart", "scope": "NewObject"},
            ],
        });
        let m = build_manifest(&meta, Path::new("BRS.pubgen"), std::slice::from_ref(&wb), None).unwrap();
        assert_eq!(m["version"], 1);
        assert_eq!(m["profile"], "bps_brs");
        assert_eq!(m["publicationTitle"], "Rilis NTP");
        assert_eq!(m["bilingual"], false);
        let path = m["chapters"][0]["path"].as_str().unwrap();
        assert!(Path::new(path).is_absolute(), "chapter paths are stored absolute: {path}");
        assert!(path.ends_with("BAB_01.xlsx"), "wrong workbook: {path}");
        assert_eq!(m["toc"]["lists"], serde_json::json!(["contents", "tables", "charts"]));
        assert_eq!(m["document"]["marginsMm"]["inside"], 20.0);
        assert!(m.get("profileData").is_none(), "version 1 manifest carries no profileData");
        assert!(m.get("generateSettings").is_none(), "version 1 manifest carries no generateSettings");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_profile_id_infers_brs_from_release_and_margins() {
        let meta = serde_json::json!({"releaseNo": "61/10/32", "margins": {"top": 18.0, "bottom": 18.0, "left": 20.0, "right": 20.0}});
        let dir = std::env::temp_dir();
        let wb = dir.join("X.xlsx");
        std::fs::write(&wb, b"x").unwrap();
        let m = build_manifest(&meta, Path::new("p.pubgen"), std::slice::from_ref(&wb), None).unwrap();
        assert_eq!(m["profile"], "bps_brs");
        std::fs::remove_file(&wb).unwrap();
    }
}
