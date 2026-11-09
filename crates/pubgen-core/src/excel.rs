//! Excel workbook normalization for Orchestrator workspaces.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use crate::workspace::{CardKind, CardScope, Language, PageLayout, PublicationCard, PublicationMetadata};
use calamine::{Data, Reader, Sheets, open_workbook_auto};
use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("cannot open workbook {path}: {message}")]
    Open { path: String, message: String },
    #[error("cannot read worksheet {sheet}: {message}")]
    Sheet { sheet: String, message: String },
    #[error("invalid workbook: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct IngestReport {
    pub profile_id: String,
    pub metadata: PublicationMetadata,
    pub csv: Vec<(String, Vec<u8>)>,
    /// Narrative text files, as RTF (`rtf/<stem>.rtf`): the format that keeps paragraph layout.
    /// Packages from before the RTF change also carry Markdown (`md/<stem>.md`), still read.
    pub text: Vec<(String, Vec<u8>)>,
    pub assets: Vec<(String, Vec<u8>)>,
    pub warnings: Vec<String>,
}

pub fn ingest_workbook(path: &Path) -> Result<IngestReport, IngestError> {
    let mut workbook =
        open_workbook_auto(path).map_err(|source| IngestError::Open { path: path.display().to_string(), message: source.to_string() })?;
    let names = workbook.sheet_names().to_vec();
    if names.is_empty() {
        return Err(IngestError::Invalid("workbook has no worksheets".into()));
    }
    let profile_id = detect_profile(&names);
    let mut metadata = PublicationMetadata {
        profile_id: profile_id.to_string(),
        layout: if profile_id == "generic_report" { PageLayout::Single } else { PageLayout::Double },
        language: if profile_id == "bps_dda" { Language::IdEn } else { Language::Id },
        title: String::new(),
        title_en: None,
        cards: Vec::new(),
        extra: BTreeMap::new(),
    };
    let mut csv = Vec::new();
    let mut texts = Vec::new();
    let mut assets = Vec::new();
    let mut warnings = Vec::new();

    let base_dir = path.parent().unwrap_or_else(|| Path::new(""));

    ingest_cover_sheet(&mut workbook, &names, base_dir, &mut metadata, &mut warnings, &mut assets, &mut csv)?;

    // 2. Process all data sheets
    for name in &names {
        if is_cover(name) || is_ignored_sheet(name) {
            continue;
        }
        let range = workbook.worksheet_range(name).map_err(|source| IngestError::Sheet { sheet: name.clone(), message: source.to_string() })?;
        let rows: Vec<Vec<String>> = range.rows().map(|row| row.iter().map(cell_text).collect()).collect();

        let kv = extract_sheet_kv(&rows);
        let (kind, scope) = classify_sheet(name, profile_id, &kv);
        let file_name = safe_sheet_name(name);
        let csv_path = format!("csv/{file_name}.csv");
        // Table sheets use the merged-header CSV convention the page generator
        // expects; chart/map/narrative sheets keep the raw grid dump (the chart
        // renderer parses those directly).
        let merges = sheet_merges(&mut workbook, name);
        let (csv_bytes, table_title_opt, table_settings) = if kind == CardKind::Table {
            let (bytes, _meta, t_title, t_settings) =
                process_table_sheet_csv_and_meta(&rows, &merges, kv.get("theme").or_else(|| kv.get("tema")).map(String::as_str))?;
            (bytes, Some(t_title), t_settings)
        } else {
            (rows_to_csv(&rows)?, None, BTreeMap::new())
        };
        csv.push((csv_path.clone(), csv_bytes));

        let mut card_source = Some(csv_path);
        let mut card_title = table_title_opt
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| kv.get("title").cloned().unwrap_or_else(|| first_nonempty_row(&rows).unwrap_or_else(|| name.clone())));

        let mut card_settings = table_settings;
        if let Some(ten) = kv.get("title_en").or_else(|| kv.get("judul_en")) {
            // For Table cards, if `title` in header rows was a chapter header (e.g. `Tabel 2 PEMERINTAHAN`),
            // `title_en` was the chapter English header (`Table 2 GOVERNMENT`), NOT the table's own title_en.
            // Only set `title_en` on the card if this is not a Table card or if it doesn't match the chapter English.
            let is_chapter_header_en = card_settings
                .get("chapter_title_en")
                .or_else(|| card_settings.get("chapter_en"))
                .and_then(|v| v.as_str())
                .map(|chen| chen.trim().eq_ignore_ascii_case(ten.trim()))
                .unwrap_or(false);
            if !is_chapter_header_en
                && (kind != CardKind::Table || !ten.to_ascii_uppercase().starts_with("TABLE ") || ten.split_whitespace().count() <= 3)
            {
                card_settings.insert("title_en".to_string(), serde_json::Value::String(ten.clone()));
            }
        }
        if let Some(fen) = kv.get("file_en").or_else(|| kv.get("berkas_en")) {
            let norm_fen = fen.replace('\\', "/");
            card_settings.insert("file_en".to_string(), serde_json::Value::String(norm_fen));
        }
        if let Some(tpl) = kv.get("template").or_else(|| kv.get("diivider")).or_else(|| kv.get("divider")) {
            card_settings.insert("template".to_string(), serde_json::Value::String(tpl.clone()));
        }
        if let Some(thm) = kv.get("theme").or_else(|| kv.get("table_theme")).or_else(|| kv.get("colortheme")) {
            card_settings.insert("theme".to_string(), serde_json::Value::String(thm.clone()));
        }

        if kind == CardKind::Narrative {
            let (t, s) = process_narrative_sheet(base_dir, name, &file_name, &rows, &kv, card_title, &mut card_settings, &mut texts);
            card_title = t;
            card_source = s;

            // In BPS DDA, if an RXX review sheet specifies a divider (template or image_div),
            // a Chapter Divider page must be added 1 page before Page RXX!
            let upper_name = name.to_ascii_uppercase();
            let is_rxx = upper_name.starts_with('R') && upper_name[1..].chars().all(|c| c.is_ascii_digit() || c == '_');
            let div_tpl = kv.get("diivider").or_else(|| kv.get("divider")).or_else(|| kv.get("pembatas"));
            let div_img = kv
                .get("image_div")
                .or_else(|| kv.get("imagediv"))
                .or_else(|| kv.get("image_divider"))
                .or_else(|| kv.get("divider_image"))
                .or_else(|| kv.get("gambar_div"))
                .or_else(|| kv.get("gambar_pembatas"))
                .or_else(|| kv.get("foto_div"));

            if profile_id == "bps_dda" && is_rxx && (div_tpl.is_some() || div_img.is_some()) {
                let mut div_settings = BTreeMap::new();
                let chosen_tpl = div_tpl.cloned().unwrap_or_else(|| "Batik Infographic".to_string());
                div_settings.insert("template".to_string(), serde_json::Value::String(chosen_tpl));
                if let Some(ten) = kv.get("title_en").or_else(|| kv.get("judul_en")) {
                    div_settings.insert("title_en".to_string(), serde_json::Value::String(ten.clone()));
                }
                let mut div_source = None;
                if let Some(img_val) = div_img {
                    let just_name = Path::new(img_val).file_name().and_then(|s| s.to_str()).unwrap_or(img_val).to_string();
                    if let Some(bytes) = resolve_file(base_dir, img_val).and_then(|p| std::fs::read(&p).ok()) {
                        let asset_path = format!("assets/{just_name}");
                        if !assets.iter().any(|(p, _): &(String, Vec<u8>)| p == &asset_path) {
                            assets.push((asset_path.clone(), bytes));
                        }
                        div_source = Some(asset_path.clone());
                        div_settings.insert("image".to_string(), serde_json::Value::String(asset_path));
                    }
                }
                let div_card = PublicationCard {
                    id: format!("DIV_{name}"),
                    kind: CardKind::Divider,
                    scope: CardScope::NewPage,
                    source: div_source,
                    title: card_title.clone(),
                    settings: div_settings,
                };
                metadata.cards.push(div_card);

                // For the RXX narrative itself, remove divider template so it behaves purely as a narrative
                card_settings.remove("template");
            }
        } else if matches!(kind, CardKind::Image | CardKind::Divider) {
            let (t, s) = process_image_or_divider_sheet(base_dir, name, &kv, kind, card_title, &mut assets);
            card_title = t;
            card_source = s;
        } else if kind == CardKind::MapChart {
            let (t, s) = process_map_chart_sheet(base_dir, name, &kv, card_title, &mut card_settings, &mut assets);
            card_title = t;
            card_source = s;
        } else if kind == CardKind::Chart {
            if let Some(tbl) = kv.get("table").or_else(|| kv.get("tabel")) {
                card_source = Some(format!("csv/{tbl}.csv"));
            }
            // Raster chart: bind the image asset so page generation can place
            // it; vector chart specs keep the sheet's data CSV.
            let img_val = kv.get("file").or_else(|| kv.get("image")).or_else(|| kv.get("gambar")).map(|s| s.trim()).filter(|s| !s.is_empty());
            if let Some(img_val) = img_val {
                let lower = img_val.to_lowercase();
                if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
                    let just_name = Path::new(img_val).file_name().and_then(|s| s.to_str()).unwrap_or(img_val).to_string();
                    if let Some(bytes) = resolve_file(base_dir, img_val).and_then(|p| std::fs::read(&p).ok()) {
                        let asset_path = format!("assets/{just_name}");
                        if !assets.iter().any(|(p, _): &(String, Vec<u8>)| p == &asset_path) {
                            assets.push((asset_path.clone(), bytes));
                        }
                        card_settings.insert("image".to_string(), serde_json::Value::String(asset_path));
                    }
                }
            }
            if card_title == *name {
                card_title = format!("Grafik {name}");
            }
        }

        metadata.cards.push(PublicationCard { id: file_name, kind, scope, source: card_source, title: card_title, settings: card_settings });
    }

    Ok(IngestReport { profile_id: profile_id.to_string(), metadata, csv, text: texts, assets, warnings })
}

fn ingest_cover_sheet(
    workbook: &mut Sheets<std::io::BufReader<std::fs::File>>,
    names: &[String],
    base_dir: &Path,
    metadata: &mut PublicationMetadata,
    warnings: &mut Vec<String>,
    assets: &mut Vec<(String, Vec<u8>)>,
    csv: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), IngestError> {
    for name in names {
        if is_cover(name) {
            let range = workbook.worksheet_range(name).map_err(|source| IngestError::Sheet { sheet: name.clone(), message: source.to_string() })?;
            let rows: Vec<Vec<String>> = range.rows().map(|row| row.iter().map(cell_text).collect()).collect();
            parse_cover(&rows, metadata, warnings);

            let file_name = safe_sheet_name(name);
            let csv_path = format!("csv/{file_name}.csv");
            let csv_bytes = rows_to_csv(&rows)?;
            csv.push((csv_path.clone(), csv_bytes));

            let mut card_settings = BTreeMap::new();
            for (k, v) in &metadata.extra {
                card_settings.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            if !metadata.title.is_empty() {
                card_settings.insert("title".to_string(), serde_json::Value::String(metadata.title.clone()));
            }
            if let Some(ten) = &metadata.title_en {
                card_settings.insert("title_en".to_string(), serde_json::Value::String(ten.clone()));
            }
            let card = PublicationCard {
                id: file_name,
                kind: CardKind::Cover,
                scope: CardScope::NewPage,
                source: Some(csv_path),
                title: if !metadata.title.is_empty() {
                    if metadata.profile_id == "bps_dda" { metadata.title.clone() } else { format!("Sampul Rilis: {}", metadata.title) }
                } else if metadata.profile_id == "bps_dda" {
                    "Sampul Publikasi".to_string()
                } else {
                    "Sampul Berita Resmi Statistik (COV)".to_string()
                },
                settings: card_settings,
            };
            metadata.cards.push(card);
            break;
        }
    }

    for key in [
        "image_theme1",
        "image_theme2",
        "qr",
        "pic_photo",
        "image",
        "imagehero",
        "image_hero",
        "image_1",
        "image_landscape",
        "image_lanscape",
        "landscape",
        "logo",
        "officelogo",
        "office_logo",
        "event_theme",
        "themeevent",
        "imageback",
        "backimage",
        "backcover",
        "theme1",
        "theme2",
    ] {
        let Some(file_name) = metadata.extra.get(key) else { continue };
        let Some(resolved) = resolve_file(base_dir, file_name) else { continue };
        let Ok(bytes) = std::fs::read(&resolved) else { continue };
        let just_name = Path::new(file_name).file_name().and_then(|s| s.to_str()).unwrap_or(file_name);
        let normalized = file_name.replace('\\', "/");
        let asset_path = format!("assets/{just_name}");
        if !assets.iter().any(|(p, _): &(String, Vec<u8>)| p == &asset_path) {
            assets.push((asset_path.clone(), bytes.clone()));
        }
        if !assets.iter().any(|(p, _): &(String, Vec<u8>)| p == &normalized) {
            assets.push((normalized, bytes));
        }
    }
    Ok(())
}

/// One RTF paragraph per line, for a narrative that has no source file at all: the worksheet text
/// is the whole story there, so the fallback only has to be valid RTF.
fn plain_text_rtf(text: &str) -> Vec<u8> {
    let mut out = String::from("{\\rtf1\\ansi\\deff0 ");
    for line in text.lines() {
        for ch in line.chars() {
            match ch {
                '\\' | '{' | '}' => {
                    out.push('\\');
                    out.push(ch);
                }
                '\t' => out.push_str("\\tab "),
                c if c.is_ascii() => out.push(c),
                // RTF's Unicode escape takes a signed 16-bit unit; characters past the basic
                // multilingual plane cannot be written that way, so they become '?'.
                c => out.push_str(&format!("\\u{}?", u16::try_from(u32::from(c)).unwrap_or(0xFFFD) as i16)),
            }
        }
        out.push_str("\\par\n");
    }
    out.push('}');
    out.into_bytes()
}

#[allow(clippy::too_many_arguments)]
fn process_narrative_sheet(
    base_dir: &Path,
    name: &str,
    file_name: &str,
    rows: &[Vec<String>],
    kv: &BTreeMap<String, String>,
    mut card_title: String,
    card_settings: &mut BTreeMap<String, serde_json::Value>,
    texts: &mut Vec<(String, Vec<u8>)>,
) -> (String, Option<String>) {
    let explicit_file = kv.get("file").or_else(|| kv.get("berkas")).cloned();
    let candidate_name = explicit_file
        .unwrap_or_else(|| if name.starts_with('U') || name.starts_with('u') { format!("{name}.rtf") } else { format!("{file_name}.rtf") });

    let stem = Path::new(&candidate_name).file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
    let rtf_path = format!("rtf/{stem}.rtf");

    // Narratives travel as RTF: that is the one format that keeps the paragraph layout the page
    // generator places (alignment, indents, space before/after, run formatting). A Word file is
    // imported and written back out as RTF; Markdown or plain text from an older workspace is
    // imported and written as RTF too, instead of being handed on as Markdown.
    let mut resolved_rtf = None;
    if let Some((path, bytes)) = resolve_narrative_file(base_dir, &candidate_name, name).and_then(|p| std::fs::read(&p).ok().map(|b| (p, b))) {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        if ext == "rtf" {
            resolved_rtf = Some(bytes);
        } else if let Ok(converted) = designcraft_textimport::to_rtf(&format!("{stem}.{ext}"), &bytes) {
            resolved_rtf = Some(converted);
        }
    }

    let rtf_bytes = match resolved_rtf {
        Some(bytes) => bytes,
        None => {
            // No narrative file: what the worksheet says becomes the paragraphs.
            let mut content = String::new();
            for row in rows {
                let line = row.iter().map(String::as_str).filter(|s| !s.trim().is_empty()).collect::<Vec<_>>().join(" ");
                if !line.is_empty() {
                    content.push_str(&line);
                    content.push('\n');
                }
            }
            if content.trim().is_empty() {
                content = format!("{card_title}\n\nNarasi untuk bagian {name}.");
            }
            designcraft_textimport::to_rtf("narasi.txt", content.as_bytes()).unwrap_or_else(|_| plain_text_rtf(&content))
        }
    };

    if !texts.iter().any(|(p, _): &(String, Vec<u8>)| p == &rtf_path) {
        texts.push((rtf_path.clone(), rtf_bytes));
    }

    let fen_candidate = kv.get("file_en").or_else(|| kv.get("berkas_en")).map(|s| s.trim()).filter(|s| !s.is_empty());
    let fen_fallback = format!("{stem}_en.rtf");
    let fen_fallback_exists = base_dir.join(&fen_fallback).is_file() || base_dir.join("rtf").join(&fen_fallback).is_file();
    let fen_choice: Option<String> = if let Some(fen) = fen_candidate {
        let fen_stem = Path::new(fen).file_stem().and_then(|s| s.to_str()).unwrap_or(fen);
        if fen == candidate_name.as_str() || fen_stem == stem {
            if fen_fallback_exists { Some(fen_fallback) } else { None }
        } else if base_dir.join(fen).is_file() || base_dir.join("rtf").join(fen).is_file() {
            Some(fen.to_string())
        } else if fen_fallback_exists {
            Some(fen_fallback)
        } else {
            None
        }
    } else if fen_fallback_exists {
        Some(fen_fallback)
    } else {
        None
    };

    let mut actual_fen_path: Option<String> = None;
    if let Some(ref fen) = fen_choice
        && let Some((path, bytes)) = resolve_narrative_file(base_dir, fen, name).and_then(|p| std::fs::read(&p).ok().map(|b| (p, b)))
    {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        let fen_stem = Path::new(fen).file_stem().and_then(|s| s.to_str()).unwrap_or(fen);
        let fen_rtf = if ext == "rtf" { Some(bytes) } else { designcraft_textimport::to_rtf(&format!("{fen_stem}.{ext}"), &bytes).ok() };
        if let Some(b) = fen_rtf {
            let fen_path = format!("rtf/{fen_stem}.rtf");
            if !texts.iter().any(|(p, _): &(String, Vec<u8>)| p == &fen_path) {
                texts.push((fen_path.clone(), b));
            }
            actual_fen_path = Some(fen_path);
        }
    }

    if let Some(fen) = actual_fen_path.or(fen_choice) {
        card_settings.insert("file_en".to_string(), serde_json::Value::String(fen));
    } else {
        card_settings.remove("file_en");
    }

    if name == "U1" {
        card_title = "Ulasan Indikator Utama (U1)".to_string();
    } else if name.starts_with('U') || name.starts_with('u') {
        card_title = format!("Ulasan Narasi {name} ({stem}.rtf)");
    } else if card_title == *name || card_title.eq_ignore_ascii_case("file") || card_title.eq_ignore_ascii_case("berkas") {
        // When sheet only had 1 row (file), title remains empty so no artificial page header is generated,
        // matching 0.8.4 behavior.
        card_title = String::new();
    }

    (card_title, Some(rtf_path))
}

fn process_image_or_divider_sheet(
    base_dir: &Path,
    name: &str,
    kv: &BTreeMap<String, String>,
    kind: CardKind,
    mut card_title: String,
    assets: &mut Vec<(String, Vec<u8>)>,
) -> (String, Option<String>) {
    let lower = card_title.trim().to_ascii_lowercase();
    if card_title.trim().is_empty() || ["file", "image", "berkas", "gambar"].contains(&lower.as_str()) {
        card_title = if kind == CardKind::Divider { format!("Pembatas {name}") } else { format!("Gambar / Infografis {name}") };
    }
    let img_candidate = kv
        .get("file")
        .or_else(|| kv.get("image"))
        .or_else(|| kv.get("berkas"))
        .or_else(|| kv.get("gambar"))
        .cloned()
        .unwrap_or_else(|| format!("{name}.png"));
    let just_name = Path::new(&img_candidate).file_name().and_then(|s| s.to_str()).unwrap_or(&img_candidate).to_string();
    if let Some(bytes) = resolve_file(base_dir, &img_candidate).and_then(|p| std::fs::read(&p).ok()) {
        let asset_path = format!("assets/{just_name}");
        if !assets.iter().any(|(p, _): &(String, Vec<u8>)| p == &asset_path) {
            assets.push((asset_path.clone(), bytes));
        }
        (card_title, Some(asset_path))
    } else {
        if card_title == *name {
            if kind == CardKind::Divider {
                card_title = format!("Pembatas Bab {name}");
            } else {
                card_title = format!("Infografis / Gambar {name} ({just_name})");
            }
        }
        (card_title, Some(format!("assets/{just_name}")))
    }
}

fn process_map_chart_sheet(
    base_dir: &Path,
    name: &str,
    kv: &BTreeMap<String, String>,
    mut card_title: String,
    card_settings: &mut BTreeMap<String, serde_json::Value>,
    assets: &mut Vec<(String, Vec<u8>)>,
) -> (String, Option<String>) {
    let map_val = kv.get("map").or_else(|| kv.get("peta")).map(|s| s.trim()).filter(|s| !s.is_empty());
    let geojson_val =
        kv.get("geojson").or_else(|| kv.get("geojson_path")).or_else(|| kv.get("file_geojson")).map(|s| s.trim()).filter(|s| !s.is_empty());
    let data_val = kv
        .get("data")
        .or_else(|| kv.get("csv"))
        .or_else(|| kv.get("tabel"))
        .or_else(|| kv.get("data_csv"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    if let Some(data) = data_val {
        card_settings.insert("map_data_csv".to_string(), serde_json::Value::String(data.to_string()));
    }

    if let Some(map) = map_val {
        card_settings.insert("map_stock_name".to_string(), serde_json::Value::String(map.to_string()));
    }

    let is_custom = geojson_val.is_some();
    card_settings.insert("map_is_custom".to_string(), serde_json::Value::Bool(is_custom));

    let card_source = if let Some(geojson) = geojson_val {
        card_settings.insert("map_custom_geojson".to_string(), serde_json::Value::String(geojson.to_string()));
        let just_name = Path::new(geojson).file_name().and_then(|s| s.to_str()).unwrap_or(geojson).to_string();
        if let Some(bytes) = resolve_file(base_dir, geojson).and_then(|p| std::fs::read(&p).ok()) {
            let asset_path = format!("assets/{just_name}");
            if !assets.iter().any(|(p, _): &(String, Vec<u8>)| p == &asset_path) {
                assets.push((asset_path.clone(), bytes));
            }
            Some(asset_path)
        } else {
            Some(geojson.to_string())
        }
    } else if let Some(map) = map_val {
        let norm = map.to_uppercase().replace([' ', '-'], "_");
        Some(format!("assets/pubgen/maps/{norm}_DESA.geojson"))
    } else {
        Some(format!("{name}.geojson"))
    };

    if let Some(ten) = kv.get("title_en").or_else(|| kv.get("judul_en")) {
        card_settings.insert("title_en".to_string(), serde_json::Value::String(ten.clone()));
    }

    let lower = card_title.trim().to_ascii_lowercase();
    if card_title.trim().is_empty() || card_title == *name || ["map", "peta", "geojson", "file", "data"].contains(&lower.as_str()) {
        if let Some(t) = kv.get("title").or_else(|| kv.get("judul")) {
            card_title = t.clone();
        } else if let Some(map) = map_val {
            card_title = format!("Peta Tematik {name} ({map})");
        } else {
            card_title = format!("Peta Tematik {name}");
        }
    }

    (card_title, card_source)
}

fn detect_profile(names: &[String]) -> &'static str {
    for name in names {
        let lower = name.trim().to_lowercase();
        if lower.starts_with("brs")
            || lower.contains("ringkasan")
            || lower.contains("utama")
            || lower == "point"
            || (lower == "cov"
                && names.iter().any(|n| {
                    let nl = n.trim().to_lowercase();
                    nl == "point" || nl.starts_with('u') || nl.starts_with('t')
                }))
        {
            return "bps_brs";
        }
        if lower.contains("balancesheet")
            || lower.contains("income_statement")
            || lower.contains("cashflow")
            || lower.contains("financial")
            || lower == "p&l"
        {
            return "corporate_annual";
        }
        if lower.contains("pendahuluan")
            || lower.contains("metodologi")
            || lower.contains("abstrak")
            || lower.contains("pembahasan")
            || lower.contains("kesimpulan")
        {
            return "academic_monograph";
        }
    }
    "bps_dda"
}

pub fn is_ignored_sheet(name: &str) -> bool {
    matches!(name.trim().to_ascii_uppercase().as_str(), "NOTE" | "CATATAN")
}

pub fn is_cover(name: &str) -> bool {
    matches!(name.trim().to_ascii_uppercase().as_str(), "COVER" | "COV" | "SAMPUL")
}

pub fn is_dda_paragraph_sheet(name: &str) -> bool {
    let s = name.trim();
    if s.len() < 2 {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap_or_default().to_ascii_uppercase();
    if first == 'A' || first == 'G' || first == 'R' || first == 'B' {
        let rest: String = chars.collect();
        let rest = rest.trim_start_matches('_');
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

pub fn is_dda_table_sheet(name: &str) -> bool {
    let s = name.trim();
    let parts: Vec<&str> = s.split(['_', '.']).collect();
    if parts.len() >= 2 {
        let first = parts[0];
        let second = parts[1];
        if !first.is_empty() && first.chars().all(|c| c.is_ascii_digit()) {
            let digit_count = second.chars().take_while(|c| c.is_ascii_digit()).count();
            return digit_count >= 1;
        }
    }
    false
}

pub fn is_dda_map_sheet(name: &str) -> bool {
    let s = name.trim();
    if s.len() < 2 {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap_or_default().to_ascii_uppercase();
    if first == 'M' {
        let rest: String = chars.collect();
        let rest = rest.trim_start_matches('_');
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit() || c == '_') && rest.chars().any(|c| c.is_ascii_digit())
    } else {
        false
    }
}

pub fn is_dda_chart_or_image_sheet(name: &str) -> bool {
    let s = name.trim();
    if s.len() < 2 {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap_or_default().to_ascii_uppercase();
    if first == 'C' {
        let rest: String = chars.collect();
        let rest = rest.trim_start_matches('_');
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit() || c == '_') && rest.chars().any(|c| c.is_ascii_digit())
    } else {
        false
    }
}

pub fn is_dda_divider_sheet(name: &str) -> bool {
    let upper = name.trim().to_ascii_uppercase();
    upper == "DIV" || upper.starts_with("DIV") || upper.starts_with("PEMBATAS")
}

#[derive(Default, Debug, Clone)]
pub struct TableSheetMetaCore {
    pub chapter_title: Option<String>,
    pub chapter_title_en: Option<String>,
    pub section_title: Option<String>,
    pub section_title_en: Option<String>,
    pub table_title_row_idx: usize,
}

pub fn extract_table_sheet_metadata_core(rows: &[Vec<String>]) -> TableSheetMetaCore {
    let mut meta = TableSheetMetaCore::default();
    let max_scan = rows.len().min(12);

    for (r, row) in rows.iter().enumerate().take(max_scan) {
        let non_empty: Vec<&str> = row.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if non_empty.is_empty() {
            continue;
        }

        let first = non_empty[0];
        let norm: String = first.trim_end_matches(':').chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect();

        let is_meta = match norm.as_str() {
            "title" | "title1" | "chapter" | "bab" | "judul" => {
                let val = if non_empty.len() >= 2 {
                    non_empty[1].trim_matches('"').trim().to_string()
                } else if let Some((_, v)) = first.split_once(':') {
                    v.trim_matches('"').trim().to_string()
                } else {
                    String::new()
                };
                if !val.is_empty() {
                    meta.chapter_title = Some(val);
                }
                true
            }
            "titleen" | "title1en" | "chapteren" | "baben" | "judulen" => {
                let val = if non_empty.len() >= 2 {
                    non_empty[1].trim_matches('"').trim().to_string()
                } else if let Some((_, v)) = first.split_once(':') {
                    v.trim_matches('"').trim().to_string()
                } else {
                    String::new()
                };
                if !val.is_empty() {
                    meta.chapter_title_en = Some(val);
                }
                true
            }
            "subtitle" | "subtitle1" | "section" | "subbab" | "subjudul" => {
                let val = if non_empty.len() >= 2 {
                    non_empty[1].trim_matches('"').trim().to_string()
                } else if let Some((_, v)) = first.split_once(':') {
                    v.trim_matches('"').trim().to_string()
                } else {
                    String::new()
                };
                if !val.is_empty() {
                    meta.section_title = Some(val);
                }
                true
            }
            "subtitleen" | "subtitle1en" | "sectionen" | "subbaben" | "subjudulen" => {
                let val = if non_empty.len() >= 2 {
                    non_empty[1].trim_matches('"').trim().to_string()
                } else if let Some((_, v)) = first.split_once(':') {
                    v.trim_matches('"').trim().to_string()
                } else {
                    String::new()
                };
                if !val.is_empty() {
                    meta.section_title_en = Some(val);
                }
                true
            }
            "template" | "desain" | "style" | "theme" | "tabletheme" | "color" => true,
            _ => false,
        };

        if !is_meta {
            meta.table_title_row_idx = r;
            return meta;
        }
    }

    meta
}

fn process_table_sheet_csv_and_meta(
    rows: &[Vec<String>],
    merges: &[((usize, usize), (usize, usize))],
    theme: Option<&str>,
) -> Result<(Vec<u8>, TableSheetMetaCore, String, BTreeMap<String, serde_json::Value>), IngestError> {
    let meta = extract_table_sheet_metadata_core(rows);
    let start_r = meta.table_title_row_idx;
    let sliced_rows = if start_r < rows.len() { &rows[start_r..] } else { rows };
    let adjusted_merges: Vec<((usize, usize), (usize, usize))> = merges
        .iter()
        .filter_map(
            |&((sc, sr), (ec, er))| {
                if er < start_r { None } else { Some(((sc, sr.saturating_sub(start_r)), (ec, er.saturating_sub(start_r)))) }
            },
        )
        .collect();
    let bytes = merged_table_csv(sliced_rows, &adjusted_merges, theme).map(String::into_bytes).unwrap_or(rows_to_csv(sliced_rows)?);

    let title =
        rows.get(meta.table_title_row_idx).and_then(|r| r.iter().find(|c| !c.trim().is_empty())).map(|s| s.trim().to_string()).unwrap_or_default();

    let mut settings = BTreeMap::new();
    if let Some(ref ch) = meta.chapter_title {
        settings.insert("chapter".to_string(), serde_json::Value::String(ch.clone()));
        settings.insert("chapter_title".to_string(), serde_json::Value::String(ch.clone()));
    }
    if let Some(ref chen) = meta.chapter_title_en {
        settings.insert("chapter_en".to_string(), serde_json::Value::String(chen.clone()));
        settings.insert("chapter_title_en".to_string(), serde_json::Value::String(chen.clone()));
    }
    if let Some(ref sec) = meta.section_title {
        settings.insert("section".to_string(), serde_json::Value::String(sec.clone()));
        settings.insert("section_title".to_string(), serde_json::Value::String(sec.clone()));
        settings.insert("subtitle".to_string(), serde_json::Value::String(sec.clone()));
    }
    if let Some(ref secen) = meta.section_title_en {
        settings.insert("section_en".to_string(), serde_json::Value::String(secen.clone()));
        settings.insert("section_title_en".to_string(), serde_json::Value::String(secen.clone()));
        settings.insert("subtitle_en".to_string(), serde_json::Value::String(secen.clone()));
    }

    Ok((bytes, meta, title, settings))
}

fn extract_sheet_kv(rows: &[Vec<String>]) -> BTreeMap<String, String> {
    let mut kv = BTreeMap::new();
    for row in rows {
        if row.len() >= 2 {
            let k = row[0].trim().to_ascii_lowercase();
            let v = row[1].trim();
            if !k.is_empty() && !v.is_empty() {
                kv.entry(k).or_insert_with(|| v.to_string());
            }
        }
    }
    kv
}

fn classify_cxx_sheet(kv: &BTreeMap<String, String>) -> (CardKind, CardScope) {
    let has_table = kv.get("table").or_else(|| kv.get("tabel")).is_some_and(|v| !v.trim().is_empty());
    let has_chart_type = kv.get("chart_type").or_else(|| kv.get("type")).or_else(|| kv.get("tipe_grafik")).is_some_and(|v| !v.trim().is_empty());

    let file_candidate = kv.get("file").or_else(|| kv.get("berkas")).or_else(|| kv.get("image")).or_else(|| kv.get("gambar"));
    let is_file_image = file_candidate
        .map(|f| {
            let lower = f.to_ascii_lowercase();
            lower.ends_with(".png")
                || lower.ends_with(".jpg")
                || lower.ends_with(".jpeg")
                || lower.ends_with(".webp")
                || lower.ends_with(".svg")
                || lower.ends_with(".gif")
                || lower.ends_with(".bmp")
        })
        .unwrap_or(false);

    if has_table && has_chart_type {
        (CardKind::Chart, CardScope::NewPage)
    } else if is_file_image {
        (CardKind::Image, CardScope::NewPage)
    } else if has_chart_type {
        (CardKind::Chart, CardScope::NewPage)
    } else {
        (CardKind::Image, CardScope::NewPage)
    }
}

fn classify_sheet(name: &str, profile: &str, kv: &BTreeMap<String, String>) -> (CardKind, CardScope) {
    let upper = name.to_ascii_uppercase();
    if profile == "bps_dda" {
        if is_dda_paragraph_sheet(&upper) {
            return (CardKind::Narrative, CardScope::NewPage);
        }
        if is_dda_table_sheet(&upper) {
            return (CardKind::Table, CardScope::NewPage);
        }
        if is_dda_map_sheet(&upper) {
            return (CardKind::MapChart, CardScope::NewPage);
        }
        if is_dda_chart_or_image_sheet(&upper) {
            return classify_cxx_sheet(kv);
        }
        if is_dda_divider_sheet(&upper) {
            return (CardKind::Divider, CardScope::NewPage);
        }
    }
    if profile == "bps_brs" && (upper == "POINT" || upper == "RINGKASAN") {
        (CardKind::Narrative, CardScope::NewPage)
    } else if profile == "bps_brs" && upper.starts_with('U') {
        (CardKind::Narrative, if upper == "U1" { CardScope::NewPage } else { CardScope::NewObject })
    } else if upper.starts_with('C') || upper.starts_with("CHART") || upper.starts_with("GRAFIK") {
        (CardKind::Chart, CardScope::NewObject)
    } else if upper.starts_with('G') || upper.starts_with("GAMBAR") || upper.starts_with("IMG") {
        (CardKind::Image, CardScope::NewObject)
    } else if upper.starts_with('M') {
        (CardKind::MapChart, CardScope::NewObject)
    } else {
        (CardKind::Table, if profile == "bps_dda" { CardScope::NewPage } else { CardScope::NewObject })
    }
}

fn parse_cover(rows: &[Vec<String>], metadata: &mut PublicationMetadata, warnings: &mut Vec<String>) {
    let mut release_no = String::new();
    let mut release_date = String::new();
    let mut release_month = String::new();
    let mut agency = String::new();

    for row in rows {
        if row.len() < 2 {
            continue;
        }
        let raw_key = row.first().map(String::as_str).unwrap_or("").trim();
        let value = row[1..].iter().map(String::as_str).filter(|s| !s.trim().is_empty()).collect::<Vec<_>>().join(" ");
        if value.trim().is_empty() {
            continue;
        }
        let norm: String = raw_key.to_ascii_lowercase().chars().filter(|c| !matches!(c, '_' | ' ' | '-')).collect();
        match norm.as_str() {
            "style" | "coverstyle" | "template" => {
                metadata.extra.insert("style".into(), value);
            }
            "level" => {
                metadata.extra.insert("level".into(), value);
            }
            "title" | "title1" | "titleid" => {
                metadata.title = value.clone();
                metadata.extra.insert("title".into(), value);
            }
            "title2" => {
                if metadata.title_en.is_none() {
                    metadata.title_en = Some(value.clone());
                }
                metadata.extra.insert("title2".into(), value);
            }
            "titleen" | "title_en" => {
                metadata.title_en = Some(value.clone());
                metadata.extra.insert("title_en".into(), value);
            }
            "year" | "tahun" => {
                metadata.extra.insert("year".into(), value);
            }
            "volume" | "vol" | "volumeno" => {
                metadata.extra.insert("volume".into(), value);
            }
            "nocatalogue" | "catalogue" | "catalog" | "katalog" | "nokatalog" | "katalogbps" => {
                metadata.extra.insert("catalogue".into(), value);
            }
            "issn" | "isbn" | "code" => {
                metadata.extra.insert("code".into(), value);
            }
            "agency" | "puboffice" | "officename" | "instansi" | "lembaga" => {
                agency = value.clone();
                metadata.extra.insert("agency".into(), value.clone());
                metadata.extra.insert("pub_office".into(), value);
            }
            "officeaddress" | "address" | "alamat" | "alamatkantor" => {
                metadata.extra.insert("office_address".into(), value);
            }
            "tabletheme" | "theme" | "tablecolor" | "color" | "colortheme" | "warna" => {
                metadata.extra.insert("table_theme".into(), value.clone());
                metadata.extra.insert("theme".into(), value);
            }
            "eventtheme" | "themeevent" | "imageback" | "backimage" | "backcover" => {
                metadata.extra.insert("event_theme".into(), value);
            }
            "palette" | "aipalette" | "colorpalette" | "palet" => {
                metadata.extra.insert("palette".into(), value);
            }
            "image1" | "imagehero" | "image" => {
                metadata.extra.insert("image_hero".into(), value.clone());
                metadata.extra.insert("image_1".into(), value);
            }
            "imagelanscape" | "imagelandscape" | "landscape" => {
                metadata.extra.insert("image_landscape".into(), value.clone());
                metadata.extra.insert("image_lanscape".into(), value);
            }
            "officelogo" | "logo" => {
                metadata.extra.insert("logo".into(), value.clone());
                metadata.extra.insert("office_logo".into(), value);
            }
            "releaseno" | "no_release" | "norelease" => {
                release_no = value.clone();
                metadata.extra.insert("release_no".into(), value);
            }
            "no" | "nomor" => {
                // If it is pure digits or dotted digits (like catalogue), store as catalogue if catalogue not yet set
                let is_pure_dotted_num = !value.is_empty() && value.chars().all(|c| c.is_ascii_digit() || c == '.');
                if is_pure_dotted_num && !metadata.extra.contains_key("catalogue") {
                    metadata.extra.insert("catalogue".into(), value);
                } else {
                    release_no = value.clone();
                    metadata.extra.insert("release_no".into(), value);
                }
            }
            "date" | "tanggal" | "tgl" | "releasedate" | "volumedate" => {
                let fmt = format_excel_date(&value);
                release_date = fmt.clone();
                metadata.extra.insert("release_date".into(), fmt);
            }
            "month" | "bulan" | "releasemonth" => {
                let fmt = format_excel_month_year(&value);
                release_month = fmt.clone();
                metadata.extra.insert("release_month".into(), fmt);
            }
            "point" | "poin" | "highlight" => {
                metadata.extra.insert("point".into(), value);
            }
            "qr" | "qrcode" | "qr_code" => {
                metadata.extra.insert("qr".into(), value);
            }
            "picname" | "pic" | "narasumber" | "nama" => {
                metadata.extra.insert("pic_name".into(), value);
            }
            "picphoto" | "photo" | "foto" => {
                metadata.extra.insert("pic_photo".into(), value);
            }
            "position" | "jabatan" => {
                metadata.extra.insert("pic_position".into(), value);
            }
            "office" | "kantor" => {
                metadata.extra.insert("pic_office".into(), value);
            }
            "phone" | "telepon" | "telp" => {
                metadata.extra.insert("pic_phone".into(), value);
            }
            "email" | "surel" => {
                metadata.extra.insert("pic_email".into(), value);
            }
            "imagetheme1" | "theme1" => {
                metadata.extra.insert("image_theme1".into(), value);
            }
            "imagetheme2" | "theme2" => {
                metadata.extra.insert("image_theme2".into(), value);
            }
            _ => {
                metadata.extra.insert(raw_key.to_ascii_lowercase(), value);
            }
        }
    }

    if metadata.title.is_empty() {
        warnings.push("COVER sheet has no title field".into());
    }

    if metadata.title_en.is_none() && (!release_date.is_empty() || !agency.is_empty()) && !release_no.is_empty() {
        metadata.title_en = Some(if !release_date.is_empty() && !agency.is_empty() {
            format!("{release_date} · {agency}")
        } else if !release_date.is_empty() {
            release_date
        } else {
            agency
        });
    }

    if !release_no.is_empty() {
        metadata.extra.entry("release_no".into()).or_insert(release_no);
    }
    if !release_month.is_empty() {
        metadata.extra.entry("release_month".into()).or_insert(release_month);
    }
}

pub fn format_excel_date(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(serial) = trimmed.parse::<i64>() else {
        return trimmed.to_string();
    };
    if serial <= 1_000 || serial >= 100_000 {
        return trimmed.to_string();
    }
    let Some(base) = NaiveDate::from_ymd_opt(1899, 12, 30) else {
        return trimmed.to_string();
    };
    let Some(date) = base.checked_add_signed(chrono::Duration::days(serial)) else {
        return trimmed.to_string();
    };
    let months = ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];
    let m_idx = (date.month() as usize).saturating_sub(1);
    if let Some(m_str) = months.get(m_idx) { format!("{} {} {}", date.day(), m_str, date.year()) } else { trimmed.to_string() }
}

pub fn format_excel_month_year(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(serial) = trimmed.parse::<i64>() else {
        return trimmed.to_string();
    };
    if serial <= 1_000 || serial >= 100_000 {
        return trimmed.to_string();
    }
    let Some(base) = NaiveDate::from_ymd_opt(1899, 12, 30) else {
        return trimmed.to_string();
    };
    let Some(date) = base.checked_add_signed(chrono::Duration::days(serial)) else {
        return trimmed.to_string();
    };
    let months = ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];
    let m_idx = (date.month() as usize).saturating_sub(1);
    if let Some(m_str) = months.get(m_idx) { format!("{} {}", m_str, date.year()) } else { trimmed.to_string() }
}

fn resolve_file(base: &Path, name: &str) -> Option<PathBuf> {
    let direct_p = Path::new(name);
    if direct_p.is_file() {
        return Some(direct_p.to_path_buf());
    }
    let norm_name = name.replace('\\', "/");
    let direct_rel = base.join(&norm_name);
    if direct_rel.is_file() {
        return Some(direct_rel);
    }
    let just_name = direct_p.file_name().and_then(|s| s.to_str()).unwrap_or(name);
    let direct = base.join(just_name);
    if direct.is_file() {
        return Some(direct);
    }
    for sub in ["assets", "images", "img"] {
        let sub_p = base.join(sub).join(just_name);
        if sub_p.is_file() {
            return Some(sub_p);
        }
    }
    if let Some(parent) = base.parent() {
        let parent_p = parent.join(just_name);
        if parent_p.is_file() {
            return Some(parent_p);
        }
        for sub in ["assets", "images", "img"] {
            let sub_p = parent.join(sub).join(just_name);
            if sub_p.is_file() {
                return Some(sub_p);
            }
        }
    }
    None
}

fn resolve_narrative_file(base: &Path, file_name: &str, sheet_name: &str) -> Option<PathBuf> {
    let direct_p = Path::new(file_name);
    if direct_p.is_file() {
        return Some(direct_p.to_path_buf());
    }
    let norm_name = file_name.replace('\\', "/");
    let direct_rel = base.join(&norm_name);
    if direct_rel.is_file() {
        return Some(direct_rel);
    }
    let just_name = direct_p.file_name().and_then(|s| s.to_str()).unwrap_or(file_name);
    let candidates = [
        just_name.to_string(),
        file_name.to_string(),
        format!("{sheet_name}.rtf"),
        format!("{sheet_name}.docx"),
        format!("{sheet_name}.md"),
        sheet_name.replace('_', ".").to_string() + ".rtf",
    ];

    for candidate in &candidates {
        let direct = base.join(candidate);
        if direct.is_file() {
            return Some(direct);
        }
        for sub in ["rtf", "docx", "md"] {
            let sub_p = base.join(sub).join(candidate);
            if sub_p.is_file() {
                return Some(sub_p);
            }
        }
        if let Some(parent) = base.parent() {
            let parent_p = parent.join(candidate);
            if parent_p.is_file() {
                return Some(parent_p);
            }
            for sub in ["rtf", "docx", "md"] {
                let sub_p = parent.join(sub).join(candidate);
                if sub_p.is_file() {
                    return Some(sub_p);
                }
            }
        }
    }
    None
}

/// True for characters that only ever appear in RTF/WordML debris (punctuation, digits and
/// whitespace), never as the start of a narrative paragraph.
fn is_debris_char(ch: char) -> bool {
    ch.is_whitespace() || "()[]{}.,;:'\"`~!@#$%^&*_-+=|\\/<>?0123456789".contains(ch)
}

/// Repair a Markdown narrative that an older RTF converter polluted with WordML debris.
///
/// Two kinds of damage are healed, both from the same legacy converter:
/// - the leaked `{\*\xmlnstbl …}` group left the schema URL plus the run of punctuation and hex
///   digits that follows it (`http://schemas.microsoft.com/office/word/2003/wordml2450...)()()`)
///   in front of the real first paragraph, and a hex data run (`0000…`) after the last one;
/// - the whole embedded OOXML package was dumped as one hex paragraph (`504b0304…`, followed by
///   Word style names) after the last real paragraph, which inflated a 1 200-character narrative
///   to 18 000 characters and broke pagination.
///
/// Clean Markdown is returned untouched: the head repair only runs when a schema URL sits in the
/// first 400 characters, and the paragraph tail only when a ZIP magic shows up, which lets legacy
/// `.pubgen` packages heal on open without risking a rewrite of good text.
pub fn sanitize_markdown(text: &str) -> String {
    let cleaned = trim_embedded_package_dump(&strip_head_debris(text));
    strip_toc_bookmarks(&cleaned)
}

pub fn strip_toc_bookmarks(text: &str) -> String {
    let mut out = text.to_string();
    while let Some(start) = out.find("Toc") {
        let actual_start = if start > 0 && out.as_bytes()[start - 1] == b'_' { start - 1 } else { start };
        let digits_start = start + 3;
        let mut digits_end = digits_start;
        while digits_end < out.len() && out.as_bytes()[digits_end].is_ascii_digit() {
            digits_end += 1;
        }
        if digits_end > digits_start {
            let actual_end = if digits_end < out.len() && out.as_bytes()[digits_end] == b'_' { digits_end + 1 } else { digits_end };
            out.replace_range(actual_start..actual_end, "");
        } else {
            break;
        }
    }
    out
}

/// Drop the hex dump of an embedded OOXML package that an old converter appended to a narrative.
///
/// `504b0304` / `504b0102` are the hex spellings of the ZIP local-file-header and
/// central-directory magic (`PK\x03\x04` / `PK\x01\x02`): no sentence contains that run, so
/// everything from the paragraph that starts with it to the end of the text is converter debris.
fn trim_embedded_package_dump(text: &str) -> String {
    const ZIP_MAGICS: [&str; 2] = ["504b0304", "504b0102"];
    let lower = text.to_ascii_lowercase();
    let Some(dump_pos) = ZIP_MAGICS.iter().filter_map(|magic| lower.find(magic)).min() else {
        return text.to_owned();
    };
    let Some(head) = text.get(..dump_pos) else {
        return text.to_owned();
    };
    if head.trim().is_empty() {
        return String::new();
    }
    // Only cut when the dump starts its own paragraph; text glued onto a sentence is left alone.
    let Some(boundary) = head.rfind("\n\n") else {
        return text.to_owned();
    };
    text.get(..boundary).unwrap_or(text).trim_end().to_owned()
}

/// Drop the WordML debris in front of the first real paragraph of a legacy narrative.
fn strip_head_debris(text: &str) -> String {
    const MARKER: &str = "schemas.microsoft.com";
    let head_len = text.char_indices().nth(400).map_or(text.len(), |(index, _)| index);
    let Some(head) = text.get(..head_len) else {
        return text.to_owned();
    };
    let Some(url_pos) = head.find(MARKER) else {
        return text.to_owned();
    };
    let tail = text.get(url_pos..).unwrap_or_default();
    // Skip the leaked URL token (`http://…/wordml2450`), then the punctuation/digit run
    // behind it, stopping at the first character that can start real text.
    let token_end = tail.char_indices().find(|(_, ch)| !(ch.is_ascii_alphanumeric() || "./:-_".contains(*ch))).map_or(tail.len(), |(index, _)| index);
    let resume = tail
        .get(token_end..)
        .unwrap_or_default()
        .char_indices()
        .find(|(_, ch)| !is_debris_char(*ch))
        .map_or(tail.len(), |(index, _)| token_end.saturating_add(index));

    // The debris is the head of the converted file, so everything in front of the first real
    // paragraph goes — including the `http://` scheme that precedes the matched URL. A heading
    // marker printed before the debris (`# http://…debris…Title`) is kept.
    let prefix = text.get(..url_pos).unwrap_or_default();
    let body = text.get(url_pos.saturating_add(resume)..).unwrap_or_default();
    let mut out = String::new();
    out.push_str(&leading_heading_marker(prefix));
    out.push_str(body);
    trim_trailing_debris(&out)
}

/// Drop the hex data run the old converter appended after the last paragraph (`0000…ffff…`):
/// a long uninterrupted block of hex digits at the very end, which no sentence ends with.
fn trim_trailing_debris(text: &str) -> String {
    let mut out = text.trim_end().to_owned();
    while let Some(run_start) = out.char_indices().rev().take_while(|(_, ch)| ch.is_ascii_hexdigit()).last().map(|(index, _)| index) {
        let run = out.get(run_start..).unwrap_or_default();
        if run.len() < 40 {
            break;
        }
        out = out.get(..run_start).unwrap_or_default().trim_end().to_owned();
    }
    out
}

/// A Markdown heading marker (`#`, `##`, … plus its space) that sits in front of the debris and
/// must survive the repair, so the first paragraph keeps its heading style.
fn leading_heading_marker(prefix: &str) -> String {
    let trimmed = prefix.trim_start();
    let hash_count = trimmed.chars().take_while(|ch| *ch == '#').count();
    if hash_count == 0 || hash_count > 6 {
        return String::new();
    }
    let Some(rest) = trimmed.get(hash_count..) else {
        return String::new();
    };
    if !rest.starts_with(char::is_whitespace) {
        return String::new();
    }
    trimmed.get(..hash_count.saturating_add(1)).unwrap_or_default().to_owned()
}

fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(v) => v.clone(),
        Data::Float(v) => {
            if (v.fract()).abs() < 1e-9 {
                format!("{:.0}", v)
            } else {
                v.to_string()
            }
        }
        Data::Int(v) => v.to_string(),
        Data::Bool(v) => v.to_string(),
        Data::DateTime(v) => v.to_string(),
        Data::DateTimeIso(v) | Data::DurationIso(v) => v.clone(),
        Data::Error(v) => format!("#ERROR:{v:?}"),
    }
}

fn rows_to_csv(rows: &[Vec<String>]) -> Result<Vec<u8>, IngestError> {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    for row in rows {
        writer.write_record(row).map_err(|e| IngestError::Invalid(format!("CSV conversion failed: {e}")))?;
    }
    writer.into_inner().map_err(|e| IngestError::Invalid(format!("CSV finalize failed: {e}")))
}

/// Merged-cell list for one worksheet in UI catalog convention: ((col_start, row_start), (col_end, row_end)).
/// Only XLSX/XLS carry merge info; other formats report none.
fn sheet_merges(workbook: &mut Sheets<std::io::BufReader<std::fs::File>>, name: &str) -> Vec<((usize, usize), (usize, usize))> {
    let cells = match workbook {
        Sheets::Xlsx(x) => x.worksheet_merge_cells(name).and_then(Result::ok).unwrap_or_default(),
        Sheets::Xls(x) => x.worksheet_merge_cells(name).unwrap_or_default(),
        _ => Vec::new(),
    };
    cells.iter().map(|d| (((d.start.1) as usize, (d.start.0) as usize), ((d.end.1) as usize, (d.end.0) as usize))).collect()
}

/// True when the first cell of a row marks a footnote/source row under a table.
pub fn matches_table_note_keyword(col1: &str) -> bool {
    let lower = col1.trim().to_lowercase();
    lower.starts_with("keterangan")
        || lower.starts_with("ket.")
        || lower.starts_with("ket ")
        || lower == "ket"
        || lower.starts_with("catatan")
        || lower.starts_with("note")
        || lower.starts_with("sumber")
        || lower.starts_with("source")
}

pub fn is_table_note_or_source_row(row: &[impl AsRef<str>]) -> bool {
    let Some(col1) = row.first().map(|s| s.as_ref().trim()) else {
        return false;
    };
    if col1.is_empty() || !matches_table_note_keyword(col1) {
        return false;
    }

    row.get(1).map(|s| s.as_ref().trim().contains(':')).unwrap_or(false)
}

/// Reconstruct table column headers by propagating horizontal & vertical
/// cell merges across header rows and joining ancestor levels with `--`.
#[allow(clippy::needless_range_loop)]
pub fn build_merged_table_headers(
    grid: &[Vec<String>],
    merges: &[((usize, usize), (usize, usize))],
    header_start_row: usize,
    header_end_row: usize,
    num_cols: usize,
) -> Vec<String> {
    let header_rows_count = (header_end_row.saturating_sub(header_start_row) + 1).max(1);
    let mut eff_grid = vec![vec![String::new(); num_cols]; header_rows_count];

    for (r_offset, r) in (header_start_row..=header_end_row).enumerate() {
        if let Some(row_cells) = grid.get(r) {
            for c in 0..num_cols {
                if let Some(val) = row_cells.get(c) {
                    eff_grid[r_offset][c] = val.trim().to_string();
                }
            }
        }
    }

    for &((sc, sr), (ec, er)) in merges {
        if sr <= header_end_row && er >= header_start_row {
            let val = grid.get(sr).and_then(|row| row.get(sc)).map(|s| s.trim()).unwrap_or("");
            if !val.is_empty() && sc < ec {
                for r in sr.max(header_start_row)..=er.min(header_end_row) {
                    let r_off = r.saturating_sub(header_start_row);
                    for c in sc..=ec.min(num_cols.saturating_sub(1)) {
                        if eff_grid[r_off][c].is_empty() {
                            eff_grid[r_off][c] = val.to_string();
                        }
                    }
                }
            }
        }
    }

    let mut col_headers = Vec::with_capacity(num_cols);
    for c in 0..num_cols {
        let mut parts: Vec<String> = Vec::new();
        for r_off in 0..header_rows_count {
            let val = eff_grid[r_off][c].trim();
            let abs_row = r_off + header_start_row;
            let is_vert_merge_continuation = merges.iter().any(|&((sc, sr), (ec, er))| sc <= c && c <= ec && sr < abs_row && er >= abs_row);
            if !val.is_empty() && !is_vert_merge_continuation && parts.last().map(|s| s.as_str()) != Some(val) {
                parts.push(val.to_string());
            }
        }
        if parts.is_empty() {
            col_headers.push(format!("Column {}", c + 1));
        } else {
            col_headers.push(parts.join("--"));
        }
    }
    col_headers
}

/// Determine where the multi-row table header ends (typically 1-3 rows under the title row).
pub fn detect_header_end_row(grid: &[Vec<String>], merges: &[((usize, usize), (usize, usize))], header_start_row: usize, num_cols: usize) -> usize {
    if grid.len() <= header_start_row + 1 {
        return header_start_row;
    }
    let mut end_row = header_start_row;
    for &((_, sr), (_, er)) in merges {
        if sr == header_start_row && er > header_start_row {
            end_row = end_row.max(er);
        }
    }
    let candidate = header_start_row + 1;
    if candidate < grid.len() && candidate <= end_row + 1 && candidate < header_start_row + 3 {
        let has_horizontal_merges_in_parent = merges.iter().any(|&((sc, sr), (ec, _))| sr == header_start_row && sc < ec);
        if has_horizontal_merges_in_parent {
            let row_cells = &grid[candidate];
            let non_empty_count = row_cells.iter().take(num_cols).filter(|s| !s.trim().is_empty()).count();
            if non_empty_count > 0 && non_empty_count < num_cols {
                end_row = end_row.max(candidate);
            }
        }
    }
    end_row.min(header_start_row + 2).min(grid.len().saturating_sub(1))
}

/// `true` for a cell holding a plain number (`149.18`, `-0.05`, `1,2 %`): a sub-header label like
/// `Agustus 2026` keeps its letters, so it is not a number.
fn is_numeric_cell(cell: Option<&String>) -> bool {
    let text = cell.map(|cell| cell.trim()).unwrap_or("");
    if text.is_empty() {
        return false;
    }
    let mut digits = 0usize;
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            digits += 1;
        } else if !matches!(ch, '%' | ',' | '.' | ' ' | '-' | '+' | '(' | ')' | '\u{a0}') {
            return false;
        }
    }
    digits > 0
}

/// Fold a two-row table header into the single-row grouped form the table layout draws.
///
/// The layout expresses a grouped header as one row whose columns read `Group--Child` (the same
/// convention [`merged_table_csv`] writes). Packages produced by the old converter kept the two
/// header rows of the workbook, so the sub-header row (`| | Agustus | September |`) was inserted as
/// the table's first data row. Recognising that shape and folding it keeps the group as a spanning
/// header cell and the children as its second header row. Returns `true` when it folded the rows.
pub fn merge_multirow_table_header(rows: &mut Vec<Vec<String>>) -> bool {
    // Older packages kept the caption row ("Tabel 3 …") on top of the header.
    let header_idx = rows
        .first()
        .map(|first| {
            let filled: Vec<&str> = first.iter().map(|cell| cell.trim()).filter(|cell| !cell.is_empty()).collect();
            let is_caption = filled.len() == 1
                && filled.first().is_some_and(|text| {
                    let lower = text.to_lowercase();
                    lower.starts_with("tabel ") || lower.starts_with("table ")
                });
            usize::from(is_caption)
        })
        .unwrap_or(0);
    // A data row must follow the sub-header row, and the header itself must be present.
    if rows.len() < header_idx + 3 {
        return false;
    }
    let (Some(header), Some(sub)) = (rows.get(header_idx).cloned(), rows.get(header_idx + 1).cloned()) else {
        return false;
    };
    let cols = header.len().max(sub.len());
    if cols < 2 {
        return false;
    }
    // A sub-header row starts with empty cells (the row-spanning first column) and then labels
    // several child columns; anything numeric there means it is really a data row.
    let sub_labels = (0..cols).filter(|c| sub.get(*c).map(|cell| !cell.trim().is_empty()).unwrap_or(false)).count();
    if header.first().map(|cell| cell.trim().is_empty()).unwrap_or(true) || sub.first().map(|cell| !cell.trim().is_empty()).unwrap_or(false) {
        return false;
    }
    if sub_labels < 2 || (0..cols).any(|c| is_numeric_cell(sub.get(c))) {
        return false;
    }
    if (0..cols).any(|c| header.get(c).map(|cell| cell.contains("--")).unwrap_or(false)) {
        return false;
    }

    let mut merged = vec![String::new(); cols];
    let mut group = String::new();
    let last_label = (0..cols).rev().find(|c| sub.get(*c).map(|cell| !cell.trim().is_empty()).unwrap_or(false)).unwrap_or(0);
    for (c, slot) in merged.iter_mut().enumerate() {
        let head = header.get(c).map(|cell| cell.trim()).unwrap_or("");
        if !head.is_empty() {
            group = head.to_string();
        }
        let leaf = sub.get(c).map(|cell| cell.trim()).unwrap_or("");
        *slot = match (head.is_empty(), leaf.is_empty()) {
            (false, false) => format!("{head}--{leaf}"),
            (false, true) => head.to_string(),
            (true, false) => format!("{group}--{leaf}"),
            // Nothing on either level (or a trailing padding column): leave it empty.
            (true, true) if c > last_label || group.is_empty() => String::new(),
            (true, true) => group.clone(),
        };
    }
    rows[header_idx] = merged;
    rows.remove(header_idx + 1);
    true
}

fn escape_csv_field(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') || val.contains(';') {
        let escaped = val.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        val.to_string()
    }
}

/// Convert one table sheet grid into the catalog CSV convention the page
/// generator expects: merged multi-row headers joined with `--`, the title
/// row dropped, note/source rows kept, and an optional `theme,<name>` trailer.
pub fn merged_table_csv(grid: &[Vec<String>], merges: &[((usize, usize), (usize, usize))], theme: Option<&str>) -> Option<String> {
    if grid.is_empty() {
        return None;
    }

    // Determine the active column count of the table (ignoring trailing empty columns and note rows)
    let mut num_cols = 0usize;
    for r in 1..grid.len() {
        if let Some(row) = grid.get(r) {
            if is_table_note_or_source_row(row) {
                continue;
            }
            for (c, cell) in row.iter().enumerate() {
                if !cell.trim().is_empty() {
                    num_cols = num_cols.max(c + 1);
                }
            }
        }
    }
    for &((sc, sr), (ec, _)) in merges {
        if sr >= 1 {
            num_cols = num_cols.max(sc + 1).max(ec + 1);
        }
    }

    if num_cols == 0 {
        return None;
    }

    let header_start_row = 1;
    let header_end_row = detect_header_end_row(grid, merges, header_start_row, num_cols);
    let headers = build_merged_table_headers(grid, merges, header_start_row, header_end_row, num_cols);

    let mut csv_lines = Vec::new();
    csv_lines.push(headers.iter().map(|h| escape_csv_field(h)).collect::<Vec<_>>().join(","));

    let data_start = header_end_row + 1;
    for row in grid.iter().skip(data_start) {
        if is_table_note_or_source_row(row) {
            let last_col = row.iter().rposition(|c| !c.trim().is_empty()).unwrap_or(row.len().saturating_sub(1));
            let max_idx = last_col.max(2.min(row.len().saturating_sub(1)));
            let note_cols = row[..=max_idx].iter().map(|c| escape_csv_field(c.trim())).collect::<Vec<_>>();
            csv_lines.push(note_cols.join(","));
            continue;
        }

        let has_any = (0..num_cols).any(|c| row.get(c).map(|s| !s.trim().is_empty()).unwrap_or(false));
        if !has_any {
            continue;
        }

        let line = (0..num_cols)
            .map(|c| {
                let cell_val = row.get(c).map(String::as_str).unwrap_or("").trim();
                escape_csv_field(cell_val)
            })
            .collect::<Vec<_>>()
            .join(",");
        csv_lines.push(line);
    }

    if let Some(th) = theme.filter(|t| !t.trim().is_empty()) {
        csv_lines.push(format!("theme,{}", escape_csv_field(th)));
    }

    Some(csv_lines.join("\n"))
}

fn safe_sheet_name(name: &str) -> String {
    name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect()
}

fn first_nonempty_row(rows: &[Vec<String>]) -> Option<String> {
    rows.iter().find_map(|row| row.iter().find(|cell| !cell.trim().is_empty()).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_follow_sheet_names() {
        assert_eq!(detect_profile(&["POINT".into(), "U1".into()]), "bps_brs");
        assert_eq!(detect_profile(&["1.1.1".into()]), "bps_dda");
    }

    #[test]
    fn csv_escapes_values() {
        let bytes = rows_to_csv(&[vec!["a,b".into(), "x\n y".into()]]).expect("csv");
        assert_eq!(String::from_utf8(bytes).expect("utf8"), "\"a,b\",\"x\n y\"\n");
    }

    #[test]
    fn test_excel_date_serial_conversion() {
        assert_eq!(format_excel_date("46296"), "1 Oktober 2026");
        assert_eq!(format_excel_month_year("46266"), "September 2026");
    }

    #[test]
    fn probe_brs_ingest() {
        let p = std::path::Path::new("../../plan/pubgen/samples/brs/ntp202610.xlsx");
        let Ok(r) = ingest_workbook(p) else {
            eprintln!("skip: no sample");
            return;
        };
        eprintln!("profile={} title={}", r.profile_id, r.metadata.title);
        for c in &r.metadata.cards {
            eprintln!("CARD {:?} id={} src={:?} title={:.60}", c.kind, c.id, c.source, c.title);
        }
        for (n, b) in &r.csv {
            let head = String::from_utf8_lossy(b).lines().take(3).collect::<Vec<_>>().join(" | ");
            eprintln!("CSV {n}: {head:.150}");
        }
        for (n, b) in &r.assets {
            eprintln!("ASSET {n} {}", b.len());
        }
    }

    /// Table sheets ingest into the merged-header catalog CSV convention the
    /// page generator expects: no raw title row, merged headers joined `--`,
    /// and chart/vector sheets keep the raw grid.
    #[test]
    fn test_brs_ingest_table_csv_merged_headers() {
        let p = std::path::Path::new("../../plan/pubgen/samples/brs/ntp202610.xlsx");
        let Ok(r) = ingest_workbook(p) else {
            eprintln!("skip: no sample");
            return;
        };
        let t1 = String::from_utf8_lossy(&r.csv.iter().find(|(n, _)| n == "csv/T1.csv").expect("T1 csv").1).into_owned();
        let first = t1.lines().next().unwrap_or_default();
        assert!(first.starts_with("Kode"), "T1 header should be the real merged header, got: {first}");
        assert!(!t1.contains("Tabel 1 Indeks"), "title row must be dropped from T1.csv");
        let c1 = String::from_utf8_lossy(&r.csv.iter().find(|(n, _)| n == "csv/C1.csv").expect("C1 csv").1).into_owned();
        assert!(c1.contains("type,line"), "chart sheet keeps raw spec grid");
    }

    /// A narrative Markdown written by the old RTF converter starts with the leaked
    /// WordML group; opening such a `.pubgen` must heal it instead of placing the debris.
    #[test]
    fn test_sanitize_markdown_strips_wordml_debris() {
        let debris = "http://schemas.microsoft.com/office/word/2003/wordml2450...)()()()()()A. Perkembangan Nilai Tukar Petani\n\n# Nilai Tukar Petani (NTP)\n\nNilai Tukar Petani (NTP) diperoleh …";
        let clean = sanitize_markdown(debris);
        assert!(clean.starts_with("A. Perkembangan Nilai Tukar Petani"), "debris must be stripped, got: {clean:.120}");
        assert!(!clean.contains("schemas.microsoft.com"));
        assert!(!clean.contains("()()"));

        // A heading marker printed before the debris survives, and the trailing hex data run
        // the old converter appended after the last paragraph is dropped.
        let headed = "# http://schemas.microsoft.com/office/word/2003/wordml2450...)()()NTP Menurut Subsektor\n\nIsi.\n\n0000000000000000000000000000000000000001";
        let fixed = sanitize_markdown(headed);
        assert_eq!(fixed, "# NTP Menurut Subsektor\n\nIsi.");

        // The same converter dumped an embedded OOXML package as one hex paragraph after the
        // last real paragraph; the paragraph and everything behind it goes.
        let dumped = "A. Judul\n\nIsi paragraf.\n\n504b030414000600080000002100e9de0fbf\n\nGrid Table 1 Light Accent 6;";
        assert_eq!(sanitize_markdown(dumped), "A. Judul\n\nIsi paragraf.");

        // Clean Markdown is untouched, even when it happens to mention a schema URL later on.
        let good = "A. Perkembangan Nilai Tukar Petani\n\n# Nilai Tukar Petani (NTP)\n\nIsi paragraf.\n";
        assert_eq!(sanitize_markdown(good), good);
    }

    /// A workbook header grouped over child columns arrives as two CSV rows (`Kode,Subkelompok,
    /// Indeks Gabungan Subsektor` then `,,Agustus,September,Perubahan`). Folding it keeps the group
    /// as a spanning header instead of inserting the child labels as the first data row.
    #[test]
    fn test_merge_multirow_table_header_folds_subheader() {
        let mut rows: Vec<Vec<String>> = [
            vec!["Tabel 1 Indeks Harga Diterima Petani", "", "", "", ""],
            vec!["Kode", "Subkelompok Pengeluaran", "Indeks Gabungan Subsektor", "", ""],
            vec!["", "", "Agustus 2026", "September 2026", "Perubahan (%)"],
            vec!["1", "INDEKS HARGA YANG DITERIMA PETANI", "149.18", "150.77", "1.06"],
        ]
        .into_iter()
        .map(|row| row.into_iter().map(str::to_string).collect::<Vec<String>>())
        .collect();
        assert!(merge_multirow_table_header(&mut rows), "the sub-header row is folded");
        assert_eq!(rows.len(), 3, "caption row kept (the renderer strips it), sub-header row merged");
        assert_eq!(
            rows[1],
            vec![
                "Kode".to_string(),
                "Subkelompok Pengeluaran".to_string(),
                "Indeks Gabungan Subsektor--Agustus 2026".to_string(),
                "Indeks Gabungan Subsektor--September 2026".to_string(),
                "Indeks Gabungan Subsektor--Perubahan (%)".to_string(),
            ]
        );
        assert_eq!(rows[2][0], "1", "the first data row stays a data row");

        // A plain one-row-header table is untouched, including its numeric data rows.
        let mut plain: Vec<Vec<String>> = [vec!["Kode", "Nilai"], vec!["1", "2"], vec!["3", "4"]]
            .into_iter()
            .map(|row| row.into_iter().map(str::to_string).collect::<Vec<String>>())
            .collect();
        assert!(!merge_multirow_table_header(&mut plain));
        assert_eq!(plain.len(), 3);
    }

    /// A narrative sheet keeps paragraph layout by travelling as RTF: an RTF source is passed on
    /// untouched, and a Word source is converted to RTF (alignment included) instead of Markdown.
    #[test]
    fn test_narrative_sheet_writes_rtf_artifacts() {
        use std::io::Write as _;
        let dir = tempfile::tempdir().expect("temp dir");
        let rtf = br"{\rtf1\ansi\deff0{\fonttbl{\f0 Arial;}}\qc\f0\fs24 Centred\par\ql Left\par}";
        std::fs::write(dir.path().join("U3.rtf"), rtf).expect("write rtf");
        let mut kv = BTreeMap::new();
        kv.insert("file".to_string(), "U3.rtf".to_string());
        let mut texts: Vec<(String, Vec<u8>)> = Vec::new();
        let mut settings = BTreeMap::new();
        let (title, source) = process_narrative_sheet(dir.path(), "U3", "U3", &[], &kv, "Ulasan".to_string(), &mut settings, &mut texts);
        assert_eq!(source.as_deref(), Some("rtf/U3.rtf"));
        assert!(title.contains("U3.rtf"), "{title}");
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].0, "rtf/U3.rtf");
        assert_eq!(texts[0].1.as_slice(), &rtf[..], "an RTF source is passed on untouched");

        // A Word source becomes RTF so the page generator keeps the paragraph layout.
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        w.start_file("word/document.xml", zip::write::SimpleFileOptions::default()).expect("zip entry");
        let xml = "<w:document><w:body>\
            <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:t>Tengah</w:t></w:r></w:p>\
            <w:p><w:r><w:t>Kiri</w:t></w:r></w:p>\
            </w:body></w:document>";
        w.write_all(xml.as_bytes()).expect("write xml");
        let docx = w.finish().expect("finish zip").into_inner();
        std::fs::write(dir.path().join("U5.docx"), &docx).expect("write docx");
        let mut kv = BTreeMap::new();
        kv.insert("file".to_string(), "U5.docx".to_string());
        let mut texts: Vec<(String, Vec<u8>)> = Vec::new();
        let mut settings = BTreeMap::new();
        let (_title, source) = process_narrative_sheet(dir.path(), "U5", "U5", &[], &kv, "Ulasan".to_string(), &mut settings, &mut texts);
        assert_eq!(source.as_deref(), Some("rtf/U5.rtf"));
        let out = String::from_utf8_lossy(&texts[0].1).into_owned();
        assert!(out.starts_with("{\\rtf"), "{out}");
        assert!(out.contains("\\qc"), "Word alignment must survive as RTF: {out}");
        assert!(!out.contains("<w:"), "no WordML markup may leak: {out}");
    }

    #[test]
    fn test_dda_sheet_rules_classification() {
        // 1. COVER is COVER
        assert!(is_cover("COVER"));
        assert!(is_cover("cover"));
        assert!(is_cover("COV"));

        // 2. sheet AXX, GXX, RXX -> Content Type Paragraph (Narrative)
        assert!(is_dda_paragraph_sheet("A02"));
        assert!(is_dda_paragraph_sheet("G00"));
        assert!(is_dda_paragraph_sheet("G01"));
        assert!(is_dda_paragraph_sheet("G02"));
        assert!(is_dda_paragraph_sheet("R01"));
        assert!(is_dda_paragraph_sheet("R02"));
        assert!(is_dda_paragraph_sheet("R_03"));
        assert!(is_dda_paragraph_sheet("B01"));

        let dummy_kv = BTreeMap::new();
        let (k_a, _) = classify_sheet("A02", "bps_dda", &dummy_kv);
        assert_eq!(k_a, CardKind::Narrative);
        let (k_g, _) = classify_sheet("G00", "bps_dda", &dummy_kv);
        assert_eq!(k_g, CardKind::Narrative);
        let (k_r, _) = classify_sheet("R01", "bps_dda", &dummy_kv);
        assert_eq!(k_r, CardKind::Narrative);

        // 3. sheet XX_XXZZ -> Content Type Tabel
        assert!(is_dda_table_sheet("01_01"));
        assert!(is_dda_table_sheet("01_02"));
        assert!(is_dda_table_sheet("02_01"));
        assert!(is_dda_table_sheet("02_03"));
        assert!(is_dda_table_sheet("02_01a"));
        assert!(is_dda_table_sheet("02_01ZZ"));
        let (k_tbl, _) = classify_sheet("02_01", "bps_dda", &dummy_kv);
        assert_eq!(k_tbl, CardKind::Table);

        // 4. sheet MXX_XX -> Content Type Map Chart
        assert!(is_dda_map_sheet("M01_01"));
        assert!(is_dda_map_sheet("M02_01"));
        assert!(is_dda_map_sheet("M01"));
        let (k_map, _) = classify_sheet("M01_01", "bps_dda", &dummy_kv);
        assert_eq!(k_map, CardKind::MapChart);

        // 5. sheet CXX_XX:
        // Case A: file contains link to image, table & chart_type empty -> Gambar (Image)
        let mut kv_image = BTreeMap::new();
        kv_image.insert("title".into(), "Grafik 02.01 Panjang Jalan".into());
        kv_image.insert("file".into(), "C02_01.png".into());
        let (k_img, _) = classify_sheet("C02_01", "bps_dda", &kv_image);
        assert_eq!(k_img, CardKind::Image, "C02_01 with image file must be Image (Gambar)");

        // Case B: table and chart_type are not null -> Chart
        let mut kv_chart = BTreeMap::new();
        kv_chart.insert("title".into(), "Grafik 02.01 Panjang Jalan".into());
        kv_chart.insert("table".into(), "02_01".into());
        kv_chart.insert("chart_type".into(), "bar".into());
        let (k_cht, _) = classify_sheet("C02_01", "bps_dda", &kv_chart);
        assert_eq!(k_cht, CardKind::Chart, "C02_01 with table & chart_type must be Chart");
    }

    #[test]
    fn test_sample_dda_ingest_bab01_dual_lang() {
        let path = Path::new("../../plan/pubgen/samples/DUAL_LANG/BAB_01_2025_ai_translate.xlsx");
        if path.is_file() {
            let report = ingest_workbook(path).expect("ingest BAB_01_2025_ai_translate.xlsx");
            assert_eq!(report.profile_id, "bps_dda");
            let m03 = report.metadata.cards.iter().find(|c| c.id == "M03_02").expect("M03_02 card");
            assert_eq!(m03.title, "Grafik 03,01 Perbandingan Jumlah Penduduk antar Desa di Kecamatan Cilamaya Wetan");
            assert_eq!(
                m03.settings.get("title_en").and_then(|v| v.as_str()),
                Some("Chart 03.01 Comparison of Population Size across Villages in Cilamaya Wetan District")
            );
        }
    }

    #[test]
    fn test_sample_dda_ingest_bab02() {
        let path = Path::new("../../plan/pubgen/samples/SINGLE_LANG/BAB_02_2025.xlsx");
        if path.is_file() {
            let report = ingest_workbook(path).expect("ingest BAB_02_2025.xlsx");
            assert_eq!(report.profile_id, "bps_dda");

            // Verify sheets: R02 (Paragraph), 02_01, 02_02, 02_03 (Table), C02_01 (Image)
            let r02 = report.metadata.cards.iter().find(|c| c.id == "R02").expect("R02 card exists");
            assert_eq!(r02.kind, CardKind::Narrative);

            let t02_01 = report.metadata.cards.iter().find(|c| c.id == "02_01").expect("02_01 card exists");
            assert_eq!(t02_01.kind, CardKind::Table);

            let c02_01 = report.metadata.cards.iter().find(|c| c.id == "C02_01").expect("C02_01 card exists");
            assert_eq!(c02_01.kind, CardKind::Image, "C02_01 should be Image (Gambar)");
            assert!(c02_01.title.contains("Panjang Jalan"), "Title extracted from C02_01 row: {}", c02_01.title);

            // Note sheet should not be in publication cards
            assert!(!report.metadata.cards.iter().any(|c| c.id.eq_ignore_ascii_case("note")));
        }
    }

    #[test]
    fn test_sample_brs_ingest() {
        let path = Path::new("../../plan/pubgen/samples/brs/ntp202610.xlsx");
        if path.is_file() {
            let report = ingest_workbook(path).expect("ingest ntp202610.xlsx");
            assert_eq!(report.profile_id, "bps_brs");
            assert_eq!(report.metadata.title, "Perkembangan Nilai Tukar Petani");
            assert!(!report.csv.is_empty(), "CSV files extracted");
            // The linked narrative files come out as RTF, named after the sheet's source file.
            assert!(report.text.iter().any(|(name, _)| name == "rtf/U1.rtf"), "{:?}", report.text.iter().map(|(n, _)| n).collect::<Vec<_>>());
            assert!(
                report.text.iter().all(|(name, bytes)| name.ends_with(".rtf") && bytes.starts_with(b"{\\rtf")),
                "every narrative artifact is RTF"
            );
            assert_eq!(report.metadata.extra.get("release_no").map(String::as_str), Some("61/10/32/Th. XXVIII"));

            // Verify sheet COV is converted to CSV and updates card Cover
            assert!(report.csv.iter().any(|(p, _)| p == "csv/COV.csv"), "COV sheet converted to CSV");
            let cov_card = report.metadata.cards.first().expect("Cover card exists");
            assert_eq!(cov_card.kind, CardKind::Cover);
            assert_eq!(cov_card.source.as_deref(), Some("csv/COV.csv"));
            assert!(cov_card.settings.contains_key("release_no"));
            assert!(cov_card.settings.contains_key("release_date"));
            assert!(cov_card.settings.contains_key("agency"));
        }
    }
}
