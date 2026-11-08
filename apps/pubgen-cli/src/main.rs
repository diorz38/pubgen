//! Headless DesignCraft publication compiler.
//!
//! ```text
//! pubgen-cli compile <MANIFEST> [OPTIONS]
//! pubgen-cli info <MANIFEST>
//! pubgen-cli validate <MANIFEST>
//! pubgen-cli translate <PROJECT.pubgen> [OPTIONS]
//! pubgen-cli profiles
//! pubgen-cli --version
//! ```
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod runtime;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use designcraft_translate::mask::{mask as mask_text, number_census, unmask};
use designcraft_translate::{Lang, OnlineTranslator};
use pubgen_core::translation::{TmEntry, TmOrigin, TranslationMemory, now_stamp};
use pubgen_core::translation_units::{TransUnit, collect_units};
use pubgen_core::workspace::Glossary;
use pubgen_core::workspace_validation::WorkspaceDraft;
use pubgen_core::{ProfileId, ProfileService};
use runtime::{CliDesignCraftRuntime, DesignCraftRuntime, MockDesignCraftRuntime};
use serde::Serialize;
use serde_json::{Value, json};

const USAGE: &str = "\
usage: pubgen-cli compile <MANIFEST> [OPTIONS]
       pubgen-cli info <MANIFEST>
       pubgen-cli validate <MANIFEST>
       pubgen-cli translate <PROJECT.pubgen> [OPTIONS]
       pubgen-cli profiles
       pubgen-cli --version

Compile options:
  --pdf <PATH>               Export PDF to specified path
  --output-dir <DIR>         Directory to store output artifacts
  --json                     Output structured JSON report to stdout
  --dry-run                  Simulate compilation without launching DesignCraft
  --designcraft-exe <PATH>   Explicit path to pubgen-enabled designcraft binary
  --timeout <SECONDS>        Connection/compile timeout in seconds (default: 30)
  --profile <PROFILE_ID>     Override publication profile (bps_brs, bps_dda, generic_book, generic_report)

Translate options:
  --provider <NAME>          groq, custom or local (offline is not built into this CLI)
  --base-url <URL>           OpenAI-compatible endpoint (required for custom and local)
  --model <ID>               Model id (required)
  --key-env <VARIABLE>       Environment variable holding the API key (required for groq and custom)
  --dry-run                  Report the units without calling the provider or writing the package
";

const PUBGEN_DESIGNCRAFT_EXE: &str = "PUBGEN_DESIGNCRAFT_EXE";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("pubgen-cli {}", env!("CARGO_PKG_VERSION"));
            0
        }
        Some("compile") => report(run_compile(&args[1..])),
        Some("info") => report(run_info(&args[1..])),
        Some("validate") => report(run_validate(&args[1..])),
        Some("translate") => {
            report(parse_translate_args(&args[1..]).and_then(|translate| run_translate(&translate)).and_then(print_translate_report))
        }
        Some("profiles") => report(run_profiles()),
        _ => {
            eprintln!("{USAGE}");
            1
        }
    };
    std::process::exit(code);
}

fn report(r: Result<(), String>) -> i32 {
    match r {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("pubgen-cli: {e}");
            1
        }
    }
}

/// CLI options for the `compile` command.
#[derive(Default)]
struct CompileOptions {
    manifest: Option<PathBuf>,
    pdf: Option<String>,
    output_dir: Option<PathBuf>,
    json_output: bool,
    dry_run: bool,
    designcraft_exe: Option<PathBuf>,
    timeout_secs: Option<u64>,
    profile: Option<String>,
}

fn parse_compile_options(args: &[String]) -> Result<CompileOptions, String> {
    let mut opts = CompileOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--pdf" => {
                i = i.saturating_add(1);
                opts.pdf = Some(args.get(i).filter(|s| !s.starts_with('-')).cloned().ok_or("--pdf needs a path")?);
            }
            "--output-dir" => {
                i = i.saturating_add(1);
                let dir_str = args.get(i).filter(|s| !s.starts_with('-')).ok_or("--output-dir needs a path")?;
                opts.output_dir = Some(PathBuf::from(dir_str));
            }
            "--json" => {
                opts.json_output = true;
            }
            "--dry-run" => {
                opts.dry_run = true;
            }
            "--designcraft-exe" => {
                i = i.saturating_add(1);
                let exe_str = args.get(i).filter(|s| !s.starts_with('-')).ok_or("--designcraft-exe needs a path")?;
                opts.designcraft_exe = Some(PathBuf::from(exe_str));
            }
            "--timeout" => {
                i = i.saturating_add(1);
                let sec_str = args.get(i).filter(|s| !s.starts_with('-')).ok_or("--timeout needs seconds")?;
                let secs = sec_str.parse::<u64>().map_err(|_| "--timeout must be a positive integer")?;
                opts.timeout_secs = Some(secs);
            }
            "--profile" => {
                i = i.saturating_add(1);
                opts.profile = Some(args.get(i).filter(|s| !s.starts_with('-')).cloned().ok_or("--profile needs a profile ID")?);
            }
            a if a.starts_with('-') => return Err(format!("unknown option {a}")),
            a if opts.manifest.is_none() => opts.manifest = Some(PathBuf::from(a)),
            a => return Err(format!("unexpected argument {a}")),
        }
        i = i.saturating_add(1);
    }
    Ok(opts)
}

/// `compile <MANIFEST> [OPTIONS]`.
fn run_compile(args: &[String]) -> Result<(), String> {
    let opts = parse_compile_options(args)?;
    let manifest = opts.manifest.ok_or("missing MANIFEST (pubgen_project.json)")?;
    if !manifest.is_file() {
        return Err(format!("{}: no such file", manifest.display()));
    }

    // Validate override profile if provided
    if let Some(ref prof_str) = opts.profile
        && ProfileService::resolve(prof_str).is_none()
    {
        return Err(format!("unknown publication profile `{prof_str}`. Run `pubgen-cli profiles` to list known profiles."));
    }

    let timeout = Duration::from_secs(opts.timeout_secs.unwrap_or(30));
    let started = Instant::now();

    // Prepare parameters for pubgen.compileProject
    let mut params = json!({"path": manifest.to_string_lossy()});
    if let Some(ref pdf) = opts.pdf {
        params["pdf"] = json!(pdf);
    }
    if let Some(ref out_dir) = opts.output_dir {
        params["outputDir"] = json!(out_dir.to_string_lossy());
    }
    if let Some(ref prof) = opts.profile {
        params["profile"] = json!(prof);
    }

    let report = if opts.dry_run {
        let mut mock = MockDesignCraftRuntime::default();
        mock.execute("pubgen.compileProject", params)?
    } else {
        let candidates = if let Some(ref explicit) = opts.designcraft_exe {
            if !explicit.is_file() {
                return Err(format!("specified designcraft executable not found: {}", explicit.display()));
            }
            vec![explicit.clone()]
        } else {
            resolve_candidates()
        };

        let mut tried = Vec::new();
        let mut active_runtime: Option<CliDesignCraftRuntime> = None;

        for exe in &candidates {
            match CliDesignCraftRuntime::spawn(exe, timeout) {
                Ok(mut rt) => {
                    let caps = rt.capabilities().unwrap_or_default();
                    if caps.iter().any(|c| c == "pubgen.compileProject") {
                        active_runtime = Some(rt);
                        break;
                    }
                    tried.push(format!("{} (no pubgen.compileProject)", exe.display()));
                }
                Err(e) => {
                    tried.push(format!("{} ({e})", exe.display()));
                }
            }
        }

        let Some(mut rt) = active_runtime else {
            return Err(format!(
                "no pubgen-enabled designcraft found (tried: {}); build one with `cargo build -p designcraft --features pubgen` or set {PUBGEN_DESIGNCRAFT_EXE}",
                tried.join("; ")
            ));
        };

        rt.execute("pubgen.compileProject", params)?
    };

    let pages = report.get("pages").and_then(Value::as_u64).unwrap_or(0);
    let chapter_count = report.get("chapters").and_then(Value::as_array).map_or(0, Vec::len);

    if opts.json_output {
        let json_str = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
        println!("{json_str}");
    } else {
        let elapsed = report.get("elapsedMs").and_then(Value::as_u64).unwrap_or_else(|| started.elapsed().as_millis() as u64);

        if opts.dry_run {
            println!("[DRY RUN] plan validated for {}", manifest.display());
        } else {
            println!("compiled {} in {} ms: {} pages, {} chapters", manifest.display(), elapsed, pages, chapter_count);
        }

        if let Some(doc) = report.get("document").and_then(Value::as_str) {
            println!("  document: {doc}");
        }
        if let Some(out) = report.get("pdf").and_then(Value::as_str) {
            println!("  pdf: {out}");
        }
        for w in report.get("warnings").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
            eprintln!("  warning: {}", w.as_str().unwrap_or("?"));
        }
    }

    if pages == 0 && !opts.dry_run {
        return Err("compile produced no pages".into());
    }

    Ok(())
}

/// `info <MANIFEST>`.
fn run_info(args: &[String]) -> Result<(), String> {
    let manifest = manifest_arg(args)?;
    let v = read_json(&manifest)?;
    println!("pubgen_project.json (version {})", v.get("version").and_then(Value::as_u64).unwrap_or(0));
    println!("  title: {}", v.get("publicationTitle").and_then(Value::as_str).unwrap_or("Untitled"));
    println!("  bilingual: {}", v.get("bilingual").and_then(Value::as_bool).unwrap_or(false));
    println!("  profile: {}", v.get("profile").and_then(Value::as_str).unwrap_or("(embedded or default)"));
    println!("  chapters: {}", v.get("chapters").and_then(Value::as_array).map(Vec::len).unwrap_or(0));
    println!("  pdf export: {}", v.pointer("/build/exportPdf").and_then(Value::as_bool).unwrap_or(false));
    Ok(())
}

/// `validate <MANIFEST>`: JSON parses, chapters resolve against the manifest folder.
fn run_validate(args: &[String]) -> Result<(), String> {
    let manifest = manifest_arg(args)?;
    let v = read_json(&manifest)?;
    let dir = manifest.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let chapters = v.get("chapters").and_then(Value::as_array).ok_or("missing `chapters` array")?;
    if chapters.is_empty() {
        return Err("manifest has no chapters".into());
    }
    for (i, c) in chapters.iter().enumerate() {
        let num = c.get("chapter").and_then(Value::as_u64).unwrap_or((i + 1) as u64);
        let path = c.get("path").and_then(Value::as_str).unwrap_or_default();
        let state = if path.is_empty() {
            "no path".to_string()
        } else if dir.join(path).is_file() {
            "found".to_string()
        } else {
            "MISSING".to_string()
        };
        println!("  chapter {num}: {} — {state}", c.get("title").and_then(Value::as_str).unwrap_or(path));
        if state == "MISSING" {
            return Err(format!("chapter {num} source not found: {}", dir.join(path).display()));
        }
    }
    println!(
        "  version-2 recipe: profileData={}, generateSettings={}, pagePlan={}",
        v.get("profileData").is_some(),
        v.get("generateSettings").is_some(),
        v.get("pagePlan").is_some()
    );
    Ok(())
}

/// `profiles`: lists known canonical profiles.
fn run_profiles() -> Result<(), String> {
    println!("Available Publication Profiles:");
    let profiles = [ProfileId::BpsBrs, ProfileId::BpsDda, ProfileId::GenericBook, ProfileId::GenericReport];
    for p in &profiles {
        let prof = ProfileService::for_id(*p);
        println!(
            "  • {:<16} — {} (Paper: {}, Margins: T:{} B:{} L:{} R:{} mm, Facing: {})",
            p.id_str(),
            p.label(),
            prof.geometry.paper_size,
            prof.geometry.margin_top_mm,
            prof.geometry.margin_bottom_mm,
            prof.geometry.margin_left_mm,
            prof.geometry.margin_right_mm,
            if prof.geometry.facing_pages { "Yes" } else { "No" },
        );
    }
    Ok(())
}

/// CLI options for the `translate` command.
pub struct TranslateArgs {
    pub project: PathBuf,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub key_env: Option<String>,
    pub dry_run: bool,
}

/// What one `translate` run did: the counts printed as JSON on stdout.
#[derive(Debug, Default, Serialize)]
pub struct TranslateReport {
    pub units: usize,
    pub translated: usize,
    pub reused: usize,
    pub skipped: usize,
    /// (unit key, why it was rejected).
    pub failed: Vec<(String, String)>,
}

pub fn parse_translate_args(args: &[String]) -> Result<TranslateArgs, String> {
    let mut project: Option<PathBuf> = None;
    let mut provider: Option<String> = None;
    let mut base_url: Option<String> = None;
    let mut model: Option<String> = None;
    let mut key_env: Option<String> = None;
    let mut dry_run = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--provider" => {
                i = i.saturating_add(1);
                provider = Some(option_value(args, i, "--provider")?);
            }
            "--base-url" => {
                i = i.saturating_add(1);
                base_url = Some(option_value(args, i, "--base-url")?);
            }
            "--model" => {
                i = i.saturating_add(1);
                model = Some(option_value(args, i, "--model")?);
            }
            "--key-env" => {
                i = i.saturating_add(1);
                key_env = Some(option_value(args, i, "--key-env")?);
            }
            "--dry-run" => dry_run = true,
            // A literal key argument must not exist: the key only ever comes from --key-env.
            a if a.starts_with('-') => return Err(format!("unknown option {a}")),
            a if project.is_none() => project = Some(PathBuf::from(a)),
            a => return Err(format!("unexpected argument {a}")),
        }
        i = i.saturating_add(1);
    }
    let project = project.ok_or("missing PROJECT (.pubgen)")?;
    let provider = provider.ok_or("missing --provider (groq, custom or local)")?.to_lowercase();
    let model = model.ok_or("missing --model")?;
    let base_url = base_url.unwrap_or_default();
    match provider.as_str() {
        "groq" => {}
        "custom" | "local" => {
            if base_url.trim().is_empty() {
                return Err(format!("--provider {provider} needs --base-url"));
            }
        }
        "offline" => return Err("--provider offline is not built into this CLI; use groq, custom or local".into()),
        other => return Err(format!("unknown provider `{other}` (groq, custom or local)")),
    }
    if provider != "local" && key_env.is_none() {
        return Err(format!("--provider {provider} needs --key-env <VARIABLE>"));
    }
    Ok(TranslateArgs { project, provider, base_url, model, key_env, dry_run })
}

fn option_value(args: &[String], index: usize, option: &str) -> Result<String, String> {
    args.get(index).filter(|s| !s.is_empty() && !s.starts_with('-')).cloned().ok_or_else(|| format!("{option} needs a value"))
}

/// `translate <PROJECT.pubgen> [OPTIONS]`: runs the GUI's translation stage in-process and
/// warms the package's translation memory. Card settings and package content are untouched.
pub fn run_translate(args: &TranslateArgs) -> Result<TranslateReport, String> {
    let opened = pubgen_core::open_pubgen(&args.project).map_err(|e| format!("{}: {e}", args.project.display()))?;
    let draft = WorkspaceDraft {
        manifest: opened.manifest.clone(),
        metadata: opened.metadata.clone(),
        glossary: opened.glossary.clone(),
        tm: opened.tm.clone(),
        csv: opened.content.csv.clone(),
        text: opened.content.text.clone(),
        assets: opened.content.assets.clone(),
        warnings: Vec::new(),
    };
    let pairs = glossary_pairs(&draft.glossary);
    let units = collect_units(&draft);
    let mut tm: TranslationMemory = draft.tm.clone();
    let mut report = TranslateReport { units: units.len(), ..TranslateReport::default() };
    let mut misses: Vec<TransUnit> = Vec::new();
    for unit in units {
        if tm.lookup(&unit.text).is_some_and(|entry| !entry.target.trim().is_empty()) {
            report.reused += 1;
        } else if let Some(english) = glossary_lookup(&draft.glossary, &unit.text) {
            // Deterministic English: never sent to the provider, entered into the memory below.
            report.skipped += 1;
            tm.insert(TmEntry {
                source: unit.text.clone(),
                target: english.to_string(),
                origin: TmOrigin::Imported,
                role: unit.role.as_str().into(),
                updated: now_stamp(),
            });
        } else {
            misses.push(unit);
        }
    }
    // A dry run ends here: no translator, no environment lookup, no package write.
    if args.dry_run {
        return Ok(report);
    }

    let key = match &args.key_env {
        Some(var) => {
            std::env::var(var).ok().filter(|value| !value.trim().is_empty()).ok_or_else(|| format!("environment variable {var} is unset or empty"))?
        }
        None => String::new(),
    };
    let mut translator = if args.provider == "groq" && args.base_url.trim().is_empty() {
        OnlineTranslator::new(&key, &args.model)
    } else {
        OnlineTranslator::with_endpoint(&key, &args.model, &args.base_url)
    };

    if !misses.is_empty() {
        // Only misses go to the provider, masked with the glossary pairs the restore uses.
        let masked: Vec<_> = misses.iter().map(|unit| mask_text(&unit.text, &pairs)).collect();
        let requests: Vec<String> = masked.iter().map(|m| m.text.clone()).collect();
        let cancel = AtomicBool::new(false);
        let mut progress = |done: usize, total: usize| eprintln!("pubgen-cli: translated {done}/{total}");
        let outputs = translator
            .translate(&requests, Lang::Indonesian, Lang::English, Some(&pairs), &cancel, &mut progress)
            .map_err(|e| format!("translation failed: {e}"))?;
        if outputs.len() != misses.len() {
            return Err(format!("provider returned {} results for {} units", outputs.len(), misses.len()));
        }
        for ((unit, masked), output) in misses.iter().zip(masked.iter()).zip(outputs.iter()) {
            let restored = match unmask(output, masked) {
                Ok(restored) => restored,
                Err(e) => {
                    report.failed.push((unit.key.clone(), e.to_string()));
                    continue;
                }
            };
            let target = restored.trim();
            if target.is_empty() {
                report.failed.push((unit.key.clone(), "model returned an empty translation".into()));
            } else if number_census(&unit.text) != number_census(target) {
                // The guard that makes statistical tables safe: a changed number is a failure.
                report.failed.push((unit.key.clone(), format!("number changed in translation: {target}")));
            } else {
                tm.insert(TmEntry {
                    source: unit.text.clone(),
                    target: target.to_string(),
                    origin: TmOrigin::Ai,
                    role: unit.role.as_str().into(),
                    updated: now_stamp(),
                });
                report.translated += 1;
            }
        }
    }

    // Only the memory changes; `content` (CSV, narrative text, assets, template) is saved verbatim.
    pubgen_core::save_pubgen(&args.project, &opened.manifest, &opened.metadata, &opened.glossary, &tm, &opened.content)
        .map_err(|e| format!("{}: {e}", args.project.display()))?;
    Ok(report)
}

/// The official English for a whole cell, case-insensitively (mirrors the GUI's
/// `BpsGlossary::lookup`); `None` while an entry has no target.
fn glossary_lookup<'a>(glossary: &'a Glossary, cell: &str) -> Option<&'a str> {
    let key = cell.trim().to_lowercase();
    glossary
        .entries
        .iter()
        .find(|entry| !entry.source.trim().is_empty() && !entry.target.trim().is_empty() && entry.source.trim().to_lowercase() == key)
        .map(|entry| entry.target.trim())
}

/// `(source, target)` pairs for masking and the provider prompt: every entry naming both sides.
fn glossary_pairs(glossary: &Glossary) -> Vec<(String, String)> {
    glossary
        .entries
        .iter()
        .filter(|entry| !entry.source.trim().is_empty() && !entry.target.trim().is_empty())
        .map(|entry| (entry.source.trim().to_string(), entry.target.trim().to_string()))
        .collect()
}

/// Print the JSON report and turn a non-empty `failed` list into a non-zero exit.
fn print_translate_report(report: TranslateReport) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&report).map_err(|e| format!("report: {e}"))?;
    println!("{json}");
    if report.failed.is_empty() { Ok(()) } else { Err(format!("{} unit(s) failed to translate", report.failed.len())) }
}

fn manifest_arg(args: &[String]) -> Result<PathBuf, String> {
    let Some(first) = args.first().filter(|a| !a.starts_with('-')) else {
        return Err(format!("missing MANIFEST\n{USAGE}"));
    };
    Ok(PathBuf::from(first))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: not valid JSON: {e}", path.display()))
}

/// Candidate designcraft executables.
fn resolve_candidates() -> Vec<PathBuf> {
    let ext = std::env::consts::EXE_SUFFIX;
    let mut out = Vec::new();
    if let Ok(exe) = std::env::var(PUBGEN_DESIGNCRAFT_EXE) {
        out.push(PathBuf::from(exe));
    }
    let name = format!("designcraft{ext}");
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
        out.push(dir.join(&name));
        out.push(dir.join("assets/designcraft-pubgen").join(&name));
    }
    for base in ["target/debug", "target/release", "../target/debug", "../target/release", "assets/designcraft-pubgen"] {
        out.push(Path::new(base).join(&name));
    }
    if let Some(path) = std::env::var_os("PATH") {
        out.extend(std::env::split_paths(&path).map(|d| d.join(&name)));
    }
    out.into_iter().filter(|p| p.is_file()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compile_options_flags() {
        let args = vec![
            "my_project.json".to_string(),
            "--pdf".to_string(),
            "out.pdf".to_string(),
            "--json".to_string(),
            "--dry-run".to_string(),
            "--timeout".to_string(),
            "45".to_string(),
            "--profile".to_string(),
            "bps_brs".to_string(),
            "--output-dir".to_string(),
            "dist".to_string(),
        ];
        let opts = parse_compile_options(&args).unwrap();
        assert_eq!(opts.manifest, Some(PathBuf::from("my_project.json")));
        assert_eq!(opts.pdf, Some("out.pdf".to_string()));
        assert!(opts.json_output);
        assert!(opts.dry_run);
        assert_eq!(opts.timeout_secs, Some(45));
        assert_eq!(opts.profile, Some("bps_brs".to_string()));
        assert_eq!(opts.output_dir, Some(PathBuf::from("dist")));
    }

    #[test]
    fn test_dry_run_compilation_flow() {
        // Create temporary synthetic project manifest
        let temp = std::env::temp_dir().join(format!("test_pubgen_{}.json", std::process::id()));
        std::fs::write(&temp, r#"{"version":1,"publicationTitle":"Test","chapters":[{"chapter":1,"title":"Intro"}]}"#).unwrap();

        let args = vec![temp.to_string_lossy().to_string(), "--dry-run".to_string(), "--json".to_string()];
        let res = run_compile(&args);
        let _ = std::fs::remove_file(&temp);

        assert!(res.is_ok());
    }

    #[test]
    fn test_run_profiles_succeeds() {
        assert!(run_profiles().is_ok());
    }

    /// A tiny `.pubgen` package with one Table card whose staged CSV yields translatable
    /// units (title, headers, row labels), the shape the GUI's translation stage consumes.
    fn write_fixture_package(path: &Path) {
        use pubgen_core::workspace::{CardKind, CardScope, Language, PageLayout, ProjectManifest, PublicationCard, PublicationMetadata};
        let manifest = ProjectManifest { format_version: 1, name: "CLI translate".into(), template: false, slots: Vec::new() };
        let metadata = PublicationMetadata {
            profile_id: "bps_dda".into(),
            layout: PageLayout::Single,
            language: Language::Id,
            title: "CLI translate".into(),
            title_en: None,
            cards: vec![PublicationCard {
                id: "t1".into(),
                kind: CardKind::Table,
                scope: CardScope::NewPage,
                source: Some("T1.csv".into()),
                title: "Luas Wilayah menurut Kecamatan".into(),
                settings: std::collections::BTreeMap::new(),
            }],
            extra: std::collections::BTreeMap::new(),
        };
        let mut content = pubgen_core::ProjectContent::default();
        content.csv.push(("T1.csv".into(), b"Kecamatan;Luas Wilayah (km2)\nKarawang;1.234,5\n".to_vec()));
        pubgen_core::save_pubgen(path, &manifest, &metadata, &Glossary::default(), &TranslationMemory::default(), &content).expect("save fixture");
    }

    #[test]
    fn translate_dry_run_reports_units_without_network() {
        let dir = std::env::temp_dir().join(format!("pgs-cli-tr-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("t.pubgen");
        write_fixture_package(&path);
        let args = parse_translate_args(&[
            // The brief's snippet passes "t.pubgen"; the package lives under the temp dir, so
            // the real path is used here (a relative name would open nothing).
            path.to_string_lossy().into_owned(),
            "--provider".into(),
            "local".into(),
            "--base-url".into(),
            "http://localhost:11434/v1".into(),
            "--model".into(),
            "m".into(),
            "--dry-run".into(),
        ])
        .expect("args parse");
        let report = run_translate(&args).expect("dry run");
        assert!(report.units > 0);
        assert_eq!(report.translated, 0, "a dry run must not call the provider");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_key_is_read_from_the_environment_not_an_argument() {
        // --key-env names a variable; the key itself must never appear in argv,
        // shell history, or a process listing.
        let args = parse_translate_args(&[
            "t.pubgen".to_string(),
            "--provider".into(),
            "custom".into(),
            "--base-url".into(),
            "https://x/v1".into(),
            "--model".into(),
            "m".into(),
            "--key-env".into(),
            "MY_KEY".into(),
        ])
        .expect("args parse");
        assert_eq!(args.key_env.as_deref(), Some("MY_KEY"));
        let err = parse_translate_args(&["t.pubgen".into(), "--api-key".into(), "secret".into()]);
        assert!(err.is_err(), "a literal key argument must not exist");
    }
}
