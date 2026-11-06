//! Excel workbook normalization for Orchestrator workspaces.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use crate::workspace::{CardKind, CardScope, Language, PageLayout, PublicationCard, PublicationMetadata};
use calamine::{Data, Reader, Sheets, open_workbook_auto};
use chrono::{Datelike, NaiveDate};
use std::collections::{BTreeMap, HashMap};
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
    pub md: Vec<(String, Vec<u8>)>,
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
        layout: if profile_id == "bps_brs" { PageLayout::Double } else { PageLayout::Single },
        language: if profile_id == "bps_dda" { Language::IdEn } else { Language::Id },
        title: String::new(),
        title_en: None,
        cards: Vec::new(),
        extra: BTreeMap::new(),
    };
    let mut csv = Vec::new();
    let mut md = Vec::new();
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
        let csv_bytes = rows_to_csv(&rows)?;
        csv.push((csv_path.clone(), csv_bytes));

        let mut card_source = Some(csv_path);
        let mut card_title = kv.get("title").cloned().unwrap_or_else(|| first_nonempty_row(&rows).unwrap_or_else(|| name.clone()));

        let mut card_settings = BTreeMap::new();
        if let Some(ten) = kv.get("title_en").or_else(|| kv.get("judul_en")) {
            card_settings.insert("title_en".to_string(), serde_json::Value::String(ten.clone()));
        }

        if kind == CardKind::Narrative {
            let (t, s) = process_narrative_sheet(base_dir, name, &file_name, &rows, &kv, card_title, &mut md);
            card_title = t;
            card_source = s;
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
            if card_title == *name {
                card_title = format!("Grafik {name}");
            }
        }

        metadata.cards.push(PublicationCard { id: file_name, kind, scope, source: card_source, title: card_title, settings: card_settings });
    }

    Ok(IngestReport { profile_id: profile_id.to_string(), metadata, csv, md, assets, warnings })
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
            if let Some(ref ten) = metadata.title_en {
                card_settings.insert("title_en".to_string(), serde_json::Value::String(ten.clone()));
            }

            metadata.cards.push(PublicationCard {
                id: file_name,
                kind: CardKind::Cover,
                scope: CardScope::NewPage,
                source: Some(csv_path),
                title: if !metadata.title.is_empty() {
                    format!("Sampul Rilis: {}", metadata.title)
                } else {
                    "Sampul Berita Resmi Statistik (COV)".to_string()
                },
                settings: card_settings,
            });
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
        "imagelandscape",
        "image_landscape",
        "logo",
        "officelogo",
        "event_theme",
        "themeevent",
        "theme1",
        "theme2",
    ] {
        let Some(file_name) = metadata.extra.get(key) else { continue };
        let Some(resolved) = resolve_file(base_dir, file_name) else { continue };
        let Ok(bytes) = std::fs::read(&resolved) else { continue };
        let asset_path = format!("assets/{file_name}");
        if !assets.iter().any(|(p, _): &(String, Vec<u8>)| p == &asset_path) {
            assets.push((asset_path, bytes));
        }
    }
    Ok(())
}

fn process_narrative_sheet(
    base_dir: &Path,
    name: &str,
    file_name: &str,
    rows: &[Vec<String>],
    kv: &BTreeMap<String, String>,
    mut card_title: String,
    md: &mut Vec<(String, Vec<u8>)>,
) -> (String, Option<String>) {
    let explicit_file = kv.get("file").or_else(|| kv.get("berkas")).cloned();
    let candidate_name = explicit_file
        .unwrap_or_else(|| if name.starts_with('U') || name.starts_with('u') { format!("{name}.rtf") } else { format!("{file_name}.rtf") });

    let stem = Path::new(&candidate_name).file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);
    let md_path = format!("md/{stem}.md");

    let mut resolved_md = None;
    if let Some((path, bytes)) = resolve_narrative_file(base_dir, &candidate_name, name).and_then(|p| std::fs::read(&p).ok().map(|b| (p, b))) {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase();
        if ext == "rtf" {
            if let Ok(converted) = rtf_to_md(&bytes) {
                resolved_md = Some(converted.into_bytes());
            }
        } else if ext == "md" || ext == "txt" {
            resolved_md = Some(bytes);
        }
    }

    let md_bytes = resolved_md.unwrap_or_else(|| {
        let mut content = String::new();
        for row in rows {
            let line = row.iter().map(String::as_str).filter(|s| !s.trim().is_empty()).collect::<Vec<_>>().join(" ");
            if !line.is_empty() {
                content.push_str(&line);
                content.push_str("\n\n");
            }
        }
        if content.trim().is_empty() {
            content = format!("# {card_title}\n\nNarasi untuk bagian {name}.\n");
        }
        content.into_bytes()
    });

    if !md.iter().any(|(p, _): &(String, Vec<u8>)| p == &md_path) {
        md.push((md_path.clone(), md_bytes));
    }

    if name == "U1" {
        card_title = "Ulasan Indikator Utama (U1)".to_string();
    } else if name.starts_with('U') || name.starts_with('u') {
        card_title = format!("Ulasan Narasi {name} ({stem}.md)");
    } else if card_title == *name {
        let upper = name.to_ascii_uppercase();
        if upper.starts_with('A') {
            card_title = format!("Penjelasan Teknis ({name})");
        } else if upper == "G00" {
            card_title = "Daftar Isi (G00)".to_string();
        } else if upper == "G01" {
            card_title = "Daftar Gambar (G01)".to_string();
        } else if upper == "G02" {
            card_title = "Daftar Tabel (G02)".to_string();
        } else if upper.starts_with('G') {
            card_title = format!("Daftar Lampiran ({name})");
        } else if upper.starts_with('R') {
            card_title = format!("Ulasan Bab ({name})");
        } else if upper.starts_with('B') {
            card_title = format!("Kata Pengantar ({name})");
        }
    }

    (card_title, Some(md_path))
}

fn process_image_or_divider_sheet(
    base_dir: &Path,
    name: &str,
    kv: &BTreeMap<String, String>,
    kind: CardKind,
    mut card_title: String,
    assets: &mut Vec<(String, Vec<u8>)>,
) -> (String, Option<String>) {
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

    if card_title == *name {
        if let Some(map) = map_val {
            card_title = format!("Peta Tematik {name} ({map})");
        } else {
            card_title = format!("Peta Tematik {name}");
        }
    }

    (card_title, card_source)
}

fn detect_profile(names: &[String]) -> &'static str {
    let upper: Vec<String> = names.iter().map(|n| n.to_ascii_uppercase()).collect();
    if upper.iter().any(|n| n == "POINT" || n == "RINGKASAN" || (n.starts_with('U') && n.len() <= 4)) {
        "bps_brs"
    } else if upper.iter().any(|n| {
        n.contains('.')
            || n.starts_with("BAB")
            || is_dda_table_sheet(n)
            || is_dda_paragraph_sheet(n)
            || is_dda_chart_or_image_sheet(n)
            || is_dda_map_sheet(n)
    }) {
        "bps_dda"
    } else {
        "generic_report"
    }
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
        metadata.extra.insert(raw_key.to_ascii_lowercase(), value.clone());
        match norm.as_str() {
            "title" | "title1" | "titleid" => {
                metadata.title = value.clone();
                metadata.extra.insert("title".into(), value.clone());
                metadata.extra.insert("title1".into(), value);
            }
            "titleen" | "title2" | "title_en" => {
                metadata.title_en = Some(value.clone());
                metadata.extra.insert("title_en".into(), value.clone());
                metadata.extra.insert("title2".into(), value);
            }
            "no" | "nomor" | "releaseno" | "volumeno" | "katalog" | "catalogue" | "nocatalogue" => {
                release_no = value.clone();
                metadata.extra.insert("release_no".into(), value.clone());
                metadata.extra.insert("no".into(), value);
            }
            "date" | "tanggal" | "tgl" | "releasedate" | "volumedate" => {
                let fmt = format_excel_date(&value);
                release_date = fmt.clone();
                metadata.extra.insert("release_date".into(), fmt.clone());
                metadata.extra.insert("date".into(), fmt);
            }
            "month" | "bulan" | "releasemonth" => {
                let fmt = format_excel_month_year(&value);
                release_month = fmt.clone();
                metadata.extra.insert("release_month".into(), fmt.clone());
                metadata.extra.insert("month".into(), fmt);
            }
            "agency" | "puboffice" | "officename" | "instansi" | "lembaga" => {
                agency = value.clone();
                metadata.extra.insert("agency".into(), value.clone());
                metadata.extra.insert("pub_office".into(), value);
            }
            "year" | "tahun" => {
                metadata.extra.insert("year".into(), value);
            }
            "volume" | "vol" => {
                metadata.extra.insert("volume".into(), value);
            }
            "code" | "issn" | "isbn" => {
                metadata.extra.insert("code".into(), value);
            }
            "picname" | "pic" | "narasumber" | "nama" => {
                metadata.extra.insert("pic_name".into(), value);
            }
            "picphoto" | "photo" | "foto" => {
                metadata.extra.insert("pic_photo".into(), value);
            }
            "picposition" | "position" | "jabatan" => {
                metadata.extra.insert("pic_position".into(), value);
            }
            "picoffice" | "office" | "kantor" => {
                metadata.extra.insert("pic_office".into(), value);
            }
            "picphone" | "phone" | "telepon" | "telp" => {
                metadata.extra.insert("pic_phone".into(), value);
            }
            "picemail" | "email" | "surel" => {
                metadata.extra.insert("pic_email".into(), value);
            }
            "style" | "coverstyle" | "template" => {
                metadata.extra.insert("style".into(), value.clone());
                metadata.extra.insert("template".into(), value);
            }
            "theme" | "tabletheme" | "colortheme" | "warna" => {
                metadata.extra.insert("theme".into(), value);
            }
            "palette" | "aipalette" | "colorpalette" | "palet" => {
                metadata.extra.insert("palette".into(), value);
            }
            "image1" | "imagehero" | "image" => {
                metadata.extra.insert("image_hero".into(), value.clone());
                metadata.extra.insert("imagehero".into(), value);
            }
            "imagelandscape" | "imagelanscape" | "landscape" => {
                metadata.extra.insert("image_landscape".into(), value);
            }
            "logo" | "officelogo" => {
                metadata.extra.insert("logo".into(), value);
            }
            "officeaddress" | "address" | "alamat" | "alamatkantor" => {
                metadata.extra.insert("office_address".into(), value);
            }
            "eventtheme" | "themeevent" | "backimage" | "backcover" => {
                metadata.extra.insert("event_theme".into(), value);
            }
            "point" | "poin" | "highlight" => {
                metadata.extra.insert("point".into(), value);
            }
            "qr" | "qrcode" => {
                metadata.extra.insert("qr".into(), value);
            }
            _ => {}
        }
    }

    if metadata.title.is_empty() {
        warnings.push("COVER sheet has no title field".into());
    }

    if metadata.title_en.is_none() && (!release_date.is_empty() || !agency.is_empty()) {
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

/// Convert RTF bytes into Markdown.
pub fn rtf_to_md(bytes: &[u8]) -> Result<String, IngestError> {
    let data = String::from_utf8_lossy(bytes);
    let style_map = rtf_parse_stylesheet(&data);

    let mut out = String::new();
    let mut para = String::new();
    let mut run = String::new();
    let mut run_bold = false;
    let mut run_italic = false;
    let mut bold = 0usize;
    let mut italic = 0usize;
    let mut skip_depth = 0usize;
    let mut para_prefix = String::new();

    let chars: Vec<char> = data.chars().collect();
    let mut i = 0usize;

    let skipped = [
        "fonttbl",
        "colortbl",
        "stylesheet",
        "info",
        "pict",
        "objdata",
        "header",
        "footer",
        "listtable",
        "listoverridetable",
        "generator",
        "pntext",
        "pn",
    ];

    fn push_run(para: &mut String, run: &mut String, bold: bool, italic: bool) {
        if run.is_empty() {
            return;
        }
        let text = std::mem::take(run);
        let wrap = bold || italic;
        let core_start = if wrap { text.len().saturating_sub(text.trim_start().len()) } else { 0 };
        let core_end = if wrap { text.trim_end().len() } else { text.len() };
        if let Some(slice) = text.get(..core_start) {
            para.push_str(slice);
        }
        let core_slice = if core_start < core_end { text.get(core_start..core_end) } else { None };
        if let Some(core) = core_slice {
            let wrapped = match (bold, italic) {
                (true, true) => format!("***{core}***"),
                (true, false) => format!("**{core}**"),
                (false, true) => format!("_{core}_"),
                (false, false) => core.to_owned(),
            };
            para.push_str(&wrapped);
        }
        if let Some(slice) = text.get(core_end..) {
            para.push_str(slice);
        }
    }

    fn transition(para: &mut String, run: &mut String, run_bold: &mut bool, run_italic: &mut bool, bold: bool, italic: bool) {
        if bold != *run_bold || italic != *run_italic {
            push_run(para, run, *run_bold, *run_italic);
            *run_bold = bold;
            *run_italic = italic;
        }
    }

    fn flush(para: &mut String, run: &mut String, run_bold: &mut bool, run_italic: &mut bool, out: &mut String, prefix: &mut String) {
        push_run(para, run, *run_bold, *run_italic);
        let line = para.trim().to_owned();
        if !line.is_empty() {
            if !prefix.is_empty() {
                out.push_str(prefix);
            }
            out.push_str(&line);
            out.push_str("\n\n");
        }
        para.clear();
        prefix.clear();
    }

    while i < chars.len() {
        if skip_depth > 0 {
            if let Some(&c) = chars.get(i) {
                match c {
                    '{' => skip_depth = skip_depth.saturating_add(1),
                    '}' => skip_depth = skip_depth.saturating_sub(1),
                    _ => {}
                }
            }
            i = i.saturating_add(1);
            continue;
        }
        let Some(&c) = chars.get(i) else { break };
        match c {
            '{' | '}' => i = i.saturating_add(1),
            '\\' => {
                i = i.saturating_add(1);
                let Some(&next_c) = chars.get(i) else { break };
                match next_c {
                    '\\' | '{' | '}' => {
                        transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                        para.push(next_c);
                        i = i.saturating_add(1);
                    }
                    '~' => {
                        para.push(' ');
                        i = i.saturating_add(1);
                    }
                    '\'' => {
                        i = i.saturating_add(1);
                        if let (Some(&c1), Some(&c2)) = (chars.get(i), chars.get(i + 1)) {
                            let hex_str: String = [c1, c2].iter().collect();
                            if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                                let decoded = [byte];
                                let decoded_str = String::from_utf8_lossy(&decoded);
                                transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                                run.push_str(&decoded_str);
                            }
                            i = i.saturating_add(2);
                        }
                    }
                    'a'..='z' | 'A'..='Z' => {
                        let start = i;
                        while i < chars.len() && chars.get(i).is_some_and(|ch| ch.is_ascii_alphabetic()) {
                            i = i.saturating_add(1);
                        }
                        let word: String = chars[start..i].iter().collect();
                        let mut param = String::new();
                        while i < chars.len() && chars.get(i).is_some_and(|ch| ch.is_ascii_digit() || *ch == '-') {
                            if let Some(&ch) = chars.get(i) {
                                param.push(ch);
                            }
                            i = i.saturating_add(1);
                        }
                        if i < chars.len() && chars.get(i) == Some(&' ') {
                            i = i.saturating_add(1);
                        }
                        match word.as_str() {
                            "par" | "sect" => flush(&mut para, &mut run, &mut run_bold, &mut run_italic, &mut out, &mut para_prefix),
                            "line" | "lbr" => {
                                transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                                para.push_str("  \n");
                            }
                            "tab" => para.push('\t'),
                            "pard" => para_prefix.clear(),
                            "s" if !param.is_empty() => {
                                if let Some(pfx) = param.parse::<u32>().ok().and_then(|idx| style_map.get(&idx)) {
                                    para_prefix = pfx.clone();
                                }
                            }
                            "outlinelevel" if !param.is_empty() => {
                                if let Ok(level) = param.parse::<u32>() {
                                    let count = (level + 1).clamp(1, 6) as usize;
                                    para_prefix = format!("{} ", "#".repeat(count));
                                }
                            }
                            "b" => {
                                if param == "0" {
                                    bold = bold.saturating_sub(1);
                                    if bold == 0 {
                                        transition(&mut para, &mut run, &mut run_bold, &mut run_italic, false, italic > 0);
                                    }
                                } else {
                                    if bold == 0 {
                                        transition(&mut para, &mut run, &mut run_bold, &mut run_italic, true, italic > 0);
                                    }
                                    bold = bold.saturating_add(1);
                                }
                            }
                            "i" => {
                                if param == "0" {
                                    italic = italic.saturating_sub(1);
                                    if italic == 0 {
                                        transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, false);
                                    }
                                } else {
                                    if italic == 0 {
                                        transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, true);
                                    }
                                    italic = italic.saturating_add(1);
                                }
                            }
                            "u" => {
                                let code: i32 = param.trim_start_matches('-').parse().unwrap_or(0);
                                transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                                if let Some(ch) = char::from_u32(code as u32) {
                                    run.push(ch);
                                }
                                if i < chars.len() {
                                    i = i.saturating_add(1);
                                }
                            }
                            w if skipped.contains(&w) => skip_depth = skip_depth.saturating_add(1),
                            _ => {}
                        }
                    }
                    _ => i = i.saturating_add(1),
                }
            }
            '\r' | '\n' => i = i.saturating_add(1),
            _ => {
                transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                run.push(c);
                i = i.saturating_add(1);
            }
        }
    }
    flush(&mut para, &mut run, &mut run_bold, &mut run_italic, &mut out, &mut para_prefix);
    Ok(out)
}

fn rtf_parse_stylesheet(data: &str) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    let Some(ss_off) = data.find("\\stylesheet") else { return map };

    let mut depth = 1i32;
    let mut entry: Vec<char> = Vec::new();
    let mut in_entry = false;

    for c in data[ss_off..].chars() {
        match c {
            '{' => {
                depth += 1;
                if depth == 2 {
                    entry.clear();
                    in_entry = true;
                } else if in_entry {
                    entry.push(c);
                }
            }
            '}' => {
                if depth == 2 && in_entry {
                    let mut style_idx: Option<u32> = None;
                    let mut outline_lvl: Option<u32> = None;
                    let mut segments: Vec<String> = Vec::new();
                    let mut seg = String::new();
                    let mut ei = 0usize;

                    while ei < entry.len() {
                        if entry.get(ei) == Some(&'\\') {
                            if !seg.trim().is_empty() {
                                segments.push(std::mem::take(&mut seg));
                            } else {
                                seg.clear();
                            }
                            ei = ei.saturating_add(1);
                            if ei < entry.len() && entry.get(ei).is_some_and(|ch| !ch.is_ascii_alphabetic()) {
                                ei = ei.saturating_add(1);
                                continue;
                            }
                            let w_start = ei;
                            while ei < entry.len() && entry.get(ei).is_some_and(|ch| ch.is_ascii_alphabetic()) {
                                ei = ei.saturating_add(1);
                            }
                            let word: String = entry[w_start..ei].iter().collect();
                            let mut param = String::new();
                            while ei < entry.len() && entry.get(ei).is_some_and(|ch| ch.is_ascii_digit() || *ch == '-') {
                                if let Some(&ch) = entry.get(ei) {
                                    param.push(ch);
                                }
                                ei = ei.saturating_add(1);
                            }
                            if ei < entry.len() && entry.get(ei) == Some(&' ') {
                                ei = ei.saturating_add(1);
                            }
                            if word == "s" && style_idx.is_none() {
                                style_idx = param.parse().ok().or(Some(0));
                            } else if word == "outlinelevel" {
                                outline_lvl = param.parse().ok();
                            }
                        } else if entry.get(ei) == Some(&'{') || entry.get(ei) == Some(&'}') {
                            ei = ei.saturating_add(1);
                        } else {
                            if let Some(&ch) = entry.get(ei) {
                                seg.push(ch);
                            }
                            ei = ei.saturating_add(1);
                        }
                    }
                    if !seg.trim().is_empty() {
                        segments.push(seg);
                    }

                    let name = segments.last().map(|s| s.trim().trim_end_matches(';').trim().to_ascii_lowercase()).unwrap_or_default();

                    let prefix = if name.contains("heading 1") || name.contains("judul 1") {
                        Some("# ".to_string())
                    } else if name.contains("heading 2") || name.contains("judul 2") {
                        Some("## ".to_string())
                    } else if name.contains("heading 3") || name.contains("judul 3") {
                        Some("### ".to_string())
                    } else {
                        outline_lvl.map(|lvl| format!("{} ", "#".repeat((lvl + 1).clamp(1, 6) as usize)))
                    };

                    if let (Some(idx), Some(pfx)) = (style_idx, prefix) {
                        map.insert(idx, pfx);
                    }
                    in_entry = false;
                } else if in_entry {
                    entry.push(c);
                }
                depth -= 1;
                if depth <= 0 {
                    break;
                }
            }
            _ if in_entry => entry.push(c),
            _ => {}
        }
    }

    map
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
    fn test_rtf_conversion() {
        let rtf = br#"{\rtf1{\fonttbl{\f0 Arial}}\b Bold \b0 plain \i ital \i0 end\par Second line}"#;
        let md = rtf_to_md(rtf).expect("rtf to md");
        assert!(md.contains("**Bold**"));
        assert!(md.contains("_ital_"));
        assert!(md.contains("Second line"));
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
            assert!(!report.md.is_empty(), "MD files extracted from linked RTF");
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
