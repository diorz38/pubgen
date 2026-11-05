use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::actions::resolve_asset_path;
use crate::metadata_form::{Metadata, PageKind};
use crate::project::Project;

#[allow(dead_code)]
pub struct ParsedNarrativePara {
    pub text: String,
    pub style: String,
    pub is_title: bool,
    pub is_h1: bool,
    pub is_h2: bool,
    pub is_list: bool,
}

pub fn parse_brs_narrative(raw: &str) -> (String, Vec<ParsedNarrativePara>) {
    let clean = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    let mut paras: Vec<ParsedNarrativePara> = Vec::new();
    let mut full_text = String::new();

    for block in clean.split("\n\n") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lines: Vec<&str> = trimmed.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        for line in lines {
            let is_h = line.starts_with('#');
            let content = if is_h {
                line.trim_start_matches('#').trim()
            } else if line.starts_with('>') {
                line.trim_start_matches('>').trim()
            } else {
                line
            };

            if content.is_empty() {
                continue;
            }

            let is_short = content.len() <= 70 && !content.ends_with('.');
            let starts_abc = content.starts_with("A. ") || content.starts_with("B. ") || content.starts_with("C. ")
                || content.starts_with("A.") || content.starts_with("B.") || content.starts_with("C.");

            let is_indicator_h1 = content == "Nilai Tukar Petani (NTP)"
                || content.starts_with("Indeks Harga yang Diterima")
                || content.starts_with("Indeks Harga yang Dibayar")
                || content.starts_with("NTP Menurut Subsektor")
                || content.starts_with("Perbandingan NTP")
                || content.starts_with("Nilai Tukar Usaha Petani");

            let is_subsector_h2 = content.starts_with("NTP Tanaman Pangan")
                || content.starts_with("NTP Tanaman Hortikultura")
                || content.starts_with("NTP Tanaman Perkebunan")
                || content.starts_with("NTP Peternakan")
                || content.starts_with("NTP Perikanan");

            let is_list = (content.len() >= 3 && content.chars().next().is_some_and(|c| c.is_ascii_digit()) && content.chars().nth(1) == Some('.'))
                || content.starts_with("- ") || content.starts_with("* ");

            let (style, is_title, is_h1, is_h2) = if starts_abc {
                ("Title".to_string(), true, false, false)
            } else if is_indicator_h1 || (is_h && !line.starts_with("##") && is_short) {
                ("Heading 1".to_string(), false, true, false)
            } else if is_subsector_h2 || (is_h && line.starts_with("##") && is_short) {
                ("Heading 2".to_string(), false, false, true)
            } else if is_list {
                ("List Paragraph".to_string(), false, false, false)
            } else {
                ("[Basic Paragraph]".to_string(), false, false, false)
            };

            if !full_text.is_empty() {
                full_text.push('\n');
            }
            full_text.push_str(content);

            paras.push(ParsedNarrativePara {
                text: content.to_string(),
                style,
                is_title,
                is_h1,
                is_h2,
                is_list,
            });
        }
    }

    (full_text, paras)
}

pub fn parse_brs_csv_table(raw: &str) -> (String, Vec<Vec<String>>, Option<String>) {
    let clean = raw.strip_prefix('\u{FEFF}').unwrap_or(raw).trim();
    if clean.is_empty() {
        return (String::new(), Vec::new(), None);
    }

    let first_line = clean.lines().next().unwrap_or("");
    let comma_count = first_line.matches(',').count();
    let semi_count = first_line.matches(';').count();
    let tab_count = first_line.matches('\t').count();
    let delim = if tab_count > comma_count && tab_count > semi_count {
        b'\t'
    } else if semi_count > comma_count {
        b';'
    } else {
        b','
    };

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .flexible(true)
        .has_headers(false)
        .from_reader(clean.as_bytes());

    let mut all_rows: Vec<Vec<String>> = Vec::new();
    for rec in rdr.records().flatten() {
        let cells: Vec<String> = rec.iter().map(|s| s.trim().to_string()).collect();
        all_rows.push(cells);
    }

    let mut table_title = None;
    if !all_rows.is_empty() {
        let first = all_rows[0].first().map(|s| s.trim()).unwrap_or("");
        let first_low = first.to_lowercase();
        let rest_empty = all_rows[0].iter().skip(1).all(|c| c.trim().is_empty());
        if (first_low.starts_with("tabel") || first_low.starts_with("table")) && rest_empty {
            table_title = Some(first.to_string());
            all_rows.remove(0);
        }
    }

    let mut lines = Vec::new();
    for row in &all_rows {
        lines.push(row.join("\t"));
    }

    (lines.join("\n"), all_rows, table_title)
}

/// Estimates the rendered vertical height of a narrative markdown text in points
/// assuming standard BRS typography (Lato 10pt, ~88 chars/line, 498.9 pt column width).
pub fn estimate_brs_narrative_height(text: &str) -> f64 {
    let mut h = 0.0;
    for para in text.split('\n') {
        let p = para.trim();
        if p.is_empty() {
            continue;
        }
        if p.starts_with("# ") {
            h += 32.0;
        } else if p.starts_with("## ") {
            h += 26.0;
        } else if p.starts_with("1.") || p.starts_with("2.") || p.starts_with("3.") || p.starts_with("4.") || p.starts_with("5.") {
            h += 24.0;
        } else {
            let lines = ((p.len() + 87) / 88).max(1);
            h += (lines as f64) * 13.5 + 6.0;
        }
    }
    h
}

fn parse_point_items_from_text(raw: &str) -> Vec<String> {
    let clean = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(clean.as_bytes());

    let mut items = Vec::new();
    for rec in rdr.records().flatten() {
        let text = rec.iter().find(|c| !c.trim().is_empty()).map(|s| s.trim()).unwrap_or("");
        if text.is_empty() {
            continue;
        }
        let lower = text.to_lowercase();
        if lower == "item"
            || lower == "point"
            || lower == "poin"
            || lower == "ringkasan"
            || lower == "ringkasan eksekutif"
            || lower == "no"
            || lower == "nomor"
        {
            continue;
        }
        items.push(text.to_string());
    }
    if items.is_empty() {
        for line in clean.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let lower = trimmed.to_lowercase();
            if lower == "item"
                || lower == "point"
                || lower == "poin"
                || lower == "ringkasan"
                || lower == "ringkasan eksekutif"
                || lower == "no"
                || lower == "nomor"
            {
                continue;
            }
            items.push(trimmed.to_string());
        }
    }
    items
}

pub fn extract_brs_point_items(
    metadata: &Metadata,
    project: Option<&Project>,
    base_dirs: &[&Path],
) -> Vec<String> {
    // 1. Check metadata.pages for an entry matching "point", "poin", or "ringkasan"
    for page in &metadata.pages {
        let matches_src = page
            .source
            .as_ref()
            .map(|s| {
                let lower = s.to_lowercase();
                lower.contains("point") || lower.contains("poin") || lower.contains("ringkasan")
            })
            .unwrap_or(false);
        let matches_title = {
            let lower = page.title.to_lowercase();
            lower.contains("point") || lower.contains("poin") || lower.contains("ringkasan")
        };

        if matches_src || matches_title {
            if let Some(bytes) = project.and_then(|p| p.page_source(page)).map(|(_, b)| b) {
                if let Ok(text) = std::str::from_utf8(bytes) {
                    let items = parse_point_items_from_text(text);
                    if !items.is_empty() {
                        return items;
                    }
                }
            }
            if let Some(ref src) = page.source {
                let resolved = resolve_asset_path(src, base_dirs);
                if let Ok(text) = std::fs::read_to_string(&resolved) {
                    let items = parse_point_items_from_text(&text);
                    if !items.is_empty() {
                        return items;
                    }
                }
            }
        }
    }

    // 2. Check project.content directly
    if let Some(p) = project {
        for (name, bytes) in &p.content {
            let lower = name.to_lowercase();
            if lower.contains("point") || lower.contains("poin") || lower.contains("ringkasan") {
                if let Ok(text) = std::str::from_utf8(bytes) {
                    let items = parse_point_items_from_text(text);
                    if !items.is_empty() {
                        return items;
                    }
                }
            }
        }
    }

    // 3. Check base_dirs for standalone files
    let candidate_rel_paths = [
        "csv/POINT.csv",
        "csv/point.csv",
        "POINT.csv",
        "point.csv",
        "POINT.txt",
        "point.txt",
        "POINT.md",
        "point.md",
    ];
    for dir in base_dirs {
        for rel in &candidate_rel_paths {
            let p = dir.join(rel);
            if p.exists() {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    let items = parse_point_items_from_text(&text);
                    if !items.is_empty() {
                        return items;
                    }
                }
            }
        }
    }

    // 4. Scan any Excel files in base_dirs
    for dir in base_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                if ext == "xlsx" || ext == "xls" {
                    use calamine::Reader as _;
                    if let Ok(mut wb) = calamine::open_workbook_auto(&p) {
                        let sheet_names: Vec<String> = wb.sheet_names().to_vec();
                        for sname in sheet_names {
                            let slow = sname.to_lowercase();
                            if slow.contains("point") || slow.contains("poin") || slow.contains("ringkasan") {
                                if let Ok(range) = wb.worksheet_range(&sname) {
                                    let mut items = Vec::new();
                                    for row in range.rows() {
                                        for cell in row {
                                            let s = crate::ingest::cell_to_string(cell);
                                            let trimmed = s.trim();
                                            if !trimmed.is_empty() {
                                                let low = trimmed.to_lowercase();
                                                if low == "item"
                                                    || low == "point"
                                                    || low == "poin"
                                                    || low == "ringkasan"
                                                    || low == "ringkasan eksekutif"
                                                    || low == "no"
                                                    || low == "nomor"
                                                {
                                                    continue;
                                                }
                                                items.push(trimmed.to_string());
                                                break;
                                            }
                                        }
                                    }
                                    if !items.is_empty() {
                                        return items;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Vec::new()
}

pub fn build_batch_script_brs(
    metadata: &Metadata,
    project: Option<&Project>,
    base_dirs: &[&Path],
) -> String {
    let mut builder = crate::actions::BatchScriptBuilder::new();

    let brs_tpl = resolve_asset_path("assets/templates/covers/brs/brs.designcraft", base_dirs);

    // 1. Open template
    builder.push("file.open", json!({ "path": brs_tpl }));
    builder.push("layout.documentSetup", json!({
        "facingPages": true,
        "margins": { "top": 57.26, "bottom": 56.69, "inside": 53.86, "outside": 42.52 }
    }));

    // 2. Swatches
    builder.push("swatch.create", json!({
        "name": "#F68839",
        "color": [246, 136, 57]
    }));
    builder.push("swatch.create", json!({
        "name": "#F58838",
        "color": [245, 136, 56]
    }));
    builder.push("swatch.create", json!({
        "name": "#FEF7E8",
        "color": [254, 247, 232]
    }));
    builder.push("swatch.create", json!({
        "name": "#FEF0D2",
        "color": [254, 240, 210]
    }));

    // 3. Define paragraph styles
    builder.push("style.paragraph.create", json!({
        "name": "Title",
        "chars": {
            "fontFamily": "Lato",
            "fontStyle": "Bold",
            "size": 14.5,
            "fill": "#F68839"
        },
        "para": {
            "spaceBefore": 0.0,
            "spaceAfter": 6.0
        }
    }));
    builder.push("style.paragraph.create", json!({
        "name": "Heading 1",
        "chars": {
            "fontFamily": "Lato",
            "fontStyle": "Bold",
            "size": 13.0,
            "fill": "#F68839"
        },
        "para": {
            "spaceBefore": 12.0,
            "spaceAfter": 4.0,
            "listType": "numbers",
            "listName": "BRS",
            "numberExpression": "^#.^t",
            "listLevel": 1,
            "restartNumbers": true,
            "restartAfterLevel": 0
        }
    }));
    builder.push("style.paragraph.create", json!({
        "name": "Heading 2",
        "chars": {
            "fontFamily": "Lato",
            "fontStyle": "Bold",
            "size": 11.0,
            "fill": "#F68839"
        },
        "para": {
            "spaceBefore": 8.0,
            "spaceAfter": 3.0,
            "listType": "numbers",
            "listName": "BRS",
            "numberExpression": "^1.^#.^t",
            "listLevel": 2,
            "restartNumbers": true,
            "restartAfterLevel": 0
        }
    }));

    // 4. Populate Front Cover, Inside Cover, Back Cover
    let cov = metadata.pages.iter().find(|p| p.kind == PageKind::Cover).and_then(|p| p.cover.as_ref());

    let agency = if let Some(c) = cov && !c.pub_office.trim().is_empty() {
        c.pub_office.trim().to_string()
    } else if let Some(c) = cov && !c.agency.trim().is_empty() {
        c.agency.trim().to_string()
    } else if !metadata.publisher.trim().is_empty() {
        metadata.publisher.trim().to_string()
    } else {
        "BADAN PUSAT STATISTIK\nPROVINSI JAWA BARAT".to_string()
    };
    builder.push("story.setText", json!({ "story": 321, "text": agency }));

    let rel_no = cov
        .map(|c| c.release_no.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(metadata.release_no.as_str())
        .trim();
    let rel_date = cov
        .map(|c| c.release_date.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(metadata.release_date.as_str())
        .trim();

    let clean_no = if let Some(stripped) = rel_no.strip_prefix("No.") {
        stripped.trim()
    } else if let Some(stripped) = rel_no.strip_prefix("No ") {
        stripped.trim()
    } else {
        rel_no
    };

    let release_info = if !clean_no.is_empty() {
        if !rel_date.is_empty() {
            format!("No. {clean_no}, {rel_date}")
        } else {
            format!("No. {clean_no}")
        }
    } else {
        "No. 61/10/32/Th. XXVIII, 1 Oktober 2026".to_string()
    };
    builder.push("story.setText", json!({ "story": 323, "text": release_info }));

    let point = if let Some(c) = cov && !c.point.trim().is_empty() {
        c.point.trim().to_string()
    } else if !metadata.description.trim().is_empty() {
        metadata.description.trim().to_string()
    } else {
        "Nilai Tukar Petani Jawa Barat September 2026 sebesar 120,54 atau naik 0,99 persen dibandingkan Agustus 2026 (2018 = 100).".to_string()
    };
    builder.push("story.setText", json!({ "story": 868, "text": point }));

    let raw_title = if let Some(c) = cov && !c.title1.trim().is_empty() {
        c.title1.trim()
    } else if !metadata.title.trim().is_empty() {
        metadata.title.trim()
    } else {
        "Perkembangan Nilai Tukar Petani"
    };
    let month = cov
        .map(|c| c.release_month.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(metadata.release_month.as_str())
        .trim();
    let full_title = if !month.is_empty() && !raw_title.to_lowercase().contains(&month.to_lowercase()) {
        format!("{raw_title} {month}")
    } else {
        raw_title.to_string()
    };
    builder.push("story.setText", json!({ "story": 870, "text": full_title }));

    let hero_path = resolve_asset_path("assets/templates/covers/brs/hero.jpg", base_dirs);
    if Path::new(&hero_path).exists() {
        builder.push("file.place", json!({ "frame": 10, "path": hero_path }));
    }

    let point_items = extract_brs_point_items(metadata, project, base_dirs);
    let points_body = if !point_items.is_empty() {
        point_items.join("\n")
    } else if let Some(c) = cov && !c.point.trim().is_empty() {
        c.point.trim().to_string()
    } else {
        "Nilai Tukar Petani (NTP) adalah perbandingan indeks harga yang diterima petani (It) terhadap indeks harga yang dibayar petani (Ib).\n\nNTP merupakan salah satu indikator untuk melihat tingkat kemampuan/daya beli petani di perdesaan.".to_string()
    };
    builder.push("story.setText", json!({ "story": 884, "text": points_body }));

    let pic_name = cov.map(|c| c.pic_name.as_str()).filter(|s| !s.trim().is_empty()).unwrap_or("Yana Hendriana, S.ST");
    builder.push("story.setText", json!({ "story": 893, "text": pic_name }));

    let contact_details = if let Some(c) = cov {
        let mut lines = Vec::new();
        if !c.position.trim().is_empty() { lines.push(c.position.trim().to_string()); }
        if !c.office.trim().is_empty() { lines.push(c.office.trim().to_string()); }
        if !c.phone.trim().is_empty() { lines.push(format!("Telp: {}", c.phone.trim())); }
        if !c.email.trim().is_empty() { lines.push(format!("E-mail: {}", c.email.trim())); }
        if lines.is_empty() {
            "Ketua Tim Statistik Harga\nBPS Provinsi Jawa Barat\nTelp: (022) 7272595\nE-mail: yanah@bps.go.id".to_string()
        } else {
            lines.join("\n")
        }
    } else {
        "Ketua Tim Statistik Harga\nBPS Provinsi Jawa Barat\nTelp: (022) 7272595\nE-mail: yanah@bps.go.id".to_string()
    };
    builder.push("story.setText", json!({ "story": 895, "text": contact_details }));

    let agency_back = "BADAN PUSAT STATISTIK";
    builder.push("story.setText", json!({ "story": 901, "text": agency_back }));

    let office_address = cov.map(|c| c.office_address.as_str()).filter(|s| !s.trim().is_empty())
        .unwrap_or("Jl. dr. Sutomo No. 6-8 Jakarta 10710\nTelp : (021) 3841195, 3842508, 3810291-4, Fax : (021) 3857046\nHomepage : http://www.bps.go.id   E-mail : bpshq@bps.go.id");
    builder.push("story.setText", json!({ "story": 903, "text": office_address }));

    // 5. Insert 9 body pages before the Back Cover (index 2)
    builder.push("layout.pages.insert", json!({ "count": 1, "before": 2 }));
    for i in 2..10 {
        builder.push("layout.pages.insert", json!({ "count": 1, "after": i }));
    }

    // 6. Running Header/Footer preset hf-brs on pages 2-11
    let (h1, h2) = crate::header_footer::resolve_brs_header_texts(
        &full_title,
        clean_no,
        rel_date,
        month,
    );
    let footer_text = if h2.is_empty() { h1 } else { format!("{h1}\n{h2}") };
    builder.push("layout.headerFooter.apply", json!({
        "preset": "hf-brs",
        "sided": "double",
        "scope": {
            "range": [2, 11]
        },
        "footer": {
            "single": footer_text,
            "left": footer_text,
            "right": footer_text
        }
    }));

    // Find entries
    let find_entry_content = |pattern: &str| -> Option<String> {
        let pat_lower = pattern.to_lowercase();

        // 1. Check metadata.pages matching source or title
        for page in &metadata.pages {
            let matches_src = page.source.as_ref().is_some_and(|s| s.to_lowercase().contains(&pat_lower));
            let matches_title = page.title.to_lowercase().contains(&pat_lower);
            if matches_src || matches_title {
                if let Some(bytes) = project.and_then(|p| p.page_source(page)).map(|(_, b)| b) {
                    let s = String::from_utf8_lossy(bytes).into_owned();
                    if !s.trim().is_empty() {
                        return Some(s);
                    }
                }
                if let Some(ref src) = page.source {
                    let resolved = resolve_asset_path(src, base_dirs);
                    if let Ok(content) = std::fs::read_to_string(&resolved) {
                        if !content.trim().is_empty() {
                            return Some(content);
                        }
                    }
                }
            }
        }

        // 2. Search project.content directly by key / pattern
        if let Some(p) = project {
            if let Some((_, bytes)) = p.content.iter().find(|(name, _)| {
                let n_lower = name.to_lowercase();
                n_lower.contains(&pat_lower)
                    || n_lower.ends_with(&format!("/{pat_lower}.csv"))
                    || n_lower.ends_with(&format!("/{pat_lower}.md"))
            }) {
                let s = String::from_utf8_lossy(bytes).into_owned();
                if !s.trim().is_empty() {
                    return Some(s);
                }
            }
        }

        // 3. Search disk for csv/{pattern}.csv or md/{pattern}.md
        for ext in &["csv", "md"] {
            let rel = format!("{ext}/{pattern}.{ext}");
            let resolved = resolve_asset_path(&rel, base_dirs);
            if let Ok(content) = std::fs::read_to_string(&resolved) {
                if !content.trim().is_empty() {
                    return Some(content);
                }
            }
            let rel_upper = format!("{ext}/{pattern}.{ext}").to_uppercase();
            let resolved = resolve_asset_path(&rel_upper, base_dirs);
            if let Ok(content) = std::fs::read_to_string(&resolved) {
                if !content.trim().is_empty() {
                    return Some(content);
                }
            }
        }

        // 4. Fallback: extract directly from any .xlsx / .xlsm in base_dirs
        for base in base_dirs {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|e| e.eq_ignore_ascii_case("xlsx") || e.eq_ignore_ascii_case("xlsm"))
                    {
                        if let Ok(files) = crate::ingest::ingest_with_profile(&path, "bps_brs") {
                            if let Some(hit) = files.into_iter().find(|f| {
                                let n_lower = f.name.to_lowercase();
                                n_lower.contains(&pat_lower) || n_lower.ends_with(&format!("/{pat_lower}.csv"))
                            }) {
                                let s = String::from_utf8_lossy(&hit.bytes).into_owned();
                                if !s.trim().is_empty() {
                                    return Some(s);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    };

    let get_clean_narrative = |name: &str| -> String {
        let raw = find_entry_content(name).unwrap_or_default();
        let (clean, _) = parse_brs_narrative(&raw);
        clean
    };

    // 7. Emit items according to exact BRS architecture:
    // Page 3 (Spread 1 Right): U1 Narrative + C1 Chart
    let u1_text = get_clean_narrative("U1");
    let u1_step = builder.push("frame.create", json!({
        "spread": 1,
        "rect": [649.1, 57.3, 1148.0, 339.1],
        "content": "text",
        "text": u1_text
    }));
    builder.push("object.label", json!({ "ids": [format!("${u1_step}.id")], "label": "#slot:text:narrative_body_1" }));

    // C1 Chart frame
    let c1_step = builder.push("frame.create", json!({
        "spread": 1,
        "rect": [649.1, 350.0, 1148.0, 785.2],
        "content": "graphic"
    }));
    builder.push("object.label", json!({ "ids": [format!("${c1_step}.id")], "label": "#slot:chart:1" }));

    // Page 4 (Spread 2 Left): U2 Narrative
    let u2_text = get_clean_narrative("U2");
    let u2_step = builder.push("frame.create", json!({
        "spread": 2,
        "rect": [42.5, 57.3, 541.4, 750.9],
        "content": "text",
        "text": u2_text
    }));
    builder.push("object.label", json!({ "ids": [format!("${u2_step}.id")], "label": "#slot:text:narrative_body_2" }));

    // Page 5 (Spread 2 Right): Table 1 Title + Table 1 Frame + U4 Start
    let t1_raw = find_entry_content("T1").unwrap_or_default();
    let (t1_tab, t1_rows, t1_title) = parse_brs_csv_table(&t1_raw);
    let t1_title_text = t1_title.unwrap_or_else(|| "Tabel 1    Indeks Harga Diterima Petani, Indeks Harga Dibayar Petani per Subkelompok \nPengeluaran serta Perubahannya (2018=100)".to_string());
    
    let t1_title_step = builder.push("frame.create", json!({
        "spread": 2,
        "rect": [649.1, 57.3, 1148.0, 80.5],
        "content": "text",
        "text": t1_title_text
    }));
    builder.push("object.label", json!({ "ids": [format!("${t1_title_step}.id")], "label": "#slot:text:table_title_1" }));

    let t1_num_rows = t1_rows.len().max(1);
    let t1_calc_h = (t1_num_rows as f64 * 14.85 + 2.0).min(520.0);
    let t1_bottom_y = 82.5 + t1_calc_h;
    let u4_start_y = (t1_bottom_y + 14.0).min(730.0);

    let t1_step = builder.push("frame.create", json!({
        "spread": 2,
        "rect": [649.1, 82.5, 1148.0, t1_bottom_y],
        "content": "text",
        "text": t1_tab
    }));
    builder.push("object.label", json!({ "ids": [format!("${t1_step}.id")], "label": "#slot:table:1" }));
    builder.push("text.select", json!({ "story": format!("${t1_step}.story"), "anchor": 0, "focus": t1_tab.chars().count() }));
    builder.push("table.convertFromText", json!({ "columnSeparator": "tab" }));

    let u4_text = get_clean_narrative("U4.md").chars().take_while(|_| true).collect::<String>();
    let u4_full = if u4_text.is_empty() { get_clean_narrative("U4") } else { u4_text };
    
    let u4_start_step = builder.push("frame.create", json!({
        "spread": 2,
        "rect": [649.1, u4_start_y, 1148.0, 785.2],
        "content": "text",
        "text": u4_full
    }));
    builder.push("object.label", json!({ "ids": [format!("${u4_start_step}.id")], "label": "#slot:text:narrative_body_4" }));

    // Page 6 (Spread 3 Left): U4 Continuation
    let u4_cont_step = builder.push("frame.create", json!({
        "spread": 3,
        "rect": [42.5, 57.3, 541.4, 785.2],
        "content": "text"
    }));
    builder.push("object.label", json!({ "ids": [format!("${u4_cont_step}.id")], "label": "#slot:text:narrative_body_4_cont" }));
    builder.push("text.thread", json!({ "from": format!("${u4_start_step}.id"), "to": format!("${u4_cont_step}.id") }));

    // Page 7 (Spread 3 Right): Table 2 Title + Table 2 Frame + U4.5 Start
    let t2_raw = find_entry_content("T2").unwrap_or_default();
    let (t2_tab, t2_rows, t2_title) = parse_brs_csv_table(&t2_raw);
    let t2_title_text = t2_title.unwrap_or_else(|| "Tabel 2    Nilai Tukar Petani (NTP) Provinsi Jawa Barat per Subsektor Pertanian serta Perubahannya (2018=100)".to_string());

    let t2_title_step = builder.push("frame.create", json!({
        "spread": 3,
        "rect": [649.1, 57.3, 1148.0, 80.5],
        "content": "text",
        "text": t2_title_text
    }));
    builder.push("object.label", json!({ "ids": [format!("${t2_title_step}.id")], "label": "#slot:text:table_title_2" }));

    let t2_num_rows = t2_rows.len().max(1);
    let t2_calc_h = (t2_num_rows as f64 * 14.85 + 2.0).min(520.0);
    let t2_bottom_y = 82.5 + t2_calc_h;
    let u45_start_y = (t2_bottom_y + 14.0).min(730.0);

    let t2_step = builder.push("frame.create", json!({
        "spread": 3,
        "rect": [649.1, 82.5, 1148.0, t2_bottom_y],
        "content": "text",
        "text": t2_tab
    }));
    builder.push("object.label", json!({ "ids": [format!("${t2_step}.id")], "label": "#slot:table:2" }));
    builder.push("text.select", json!({ "story": format!("${t2_step}.story"), "anchor": 0, "focus": t2_tab.chars().count() }));
    builder.push("table.convertFromText", json!({ "columnSeparator": "tab" }));

    let u45_full = get_clean_narrative("U4.5");
    let u45_start_step = builder.push("frame.create", json!({
        "spread": 3,
        "rect": [649.1, u45_start_y, 1148.0, 785.2],
        "content": "text",
        "text": u45_full
    }));
    builder.push("object.label", json!({ "ids": [format!("${u45_start_step}.id")], "label": "#slot:text:narrative_body_45" }));

    // Page 8 (Spread 4 Left): U4.5 Cont + Table 3 Title + Table 3 Frame + U5 Start (dynamic flow)
    let u45_total_h = estimate_brs_narrative_height(&u45_full);
    let u45_p7_avail = 785.2 - u45_start_y;
    let u45_rem_h = (u45_total_h - u45_p7_avail).max(0.0);
    let (u45_p8_frame_h, t3_title_start_y) = if u45_rem_h > 10.0 {
        let end_y = (57.3 + u45_rem_h + 10.0).min(380.0);
        (end_y - 57.3, end_y + 14.0)
    } else {
        (0.0, 57.3)
    };

    let u45_cont_step = builder.push("frame.create", json!({
        "spread": 4,
        "rect": [42.5, 57.3, 541.4, 57.3 + u45_p8_frame_h.max(20.0)],
        "content": "text"
    }));
    builder.push("object.label", json!({ "ids": [format!("${u45_cont_step}.id")], "label": "#slot:text:narrative_body_45_cont" }));
    builder.push("text.thread", json!({ "from": format!("${u45_start_step}.id"), "to": format!("${u45_cont_step}.id") }));

    let t3_raw = find_entry_content("T3").unwrap_or_default();
    let (t3_tab, t3_rows, t3_title) = parse_brs_csv_table(&t3_raw);
    let t3_title_text = t3_title.unwrap_or_else(|| "Tabel 3    Perbandingan NTP Enam Provinsi di Pulau Jawa dan Nasional (2018=100)".to_string());

    let t3_title_end_y = t3_title_start_y + 23.0;
    let t3_table_start_y = t3_title_end_y + 2.0;
    let t3_num_rows = t3_rows.len().max(1);
    let t3_calc_h = (t3_num_rows as f64 * 14.85 + 2.0).min(350.0);
    let t3_table_end_y = t3_table_start_y + t3_calc_h;
    let u5_start_y = (t3_table_end_y + 14.0).min(730.0);

    let t3_title_step = builder.push("frame.create", json!({
        "spread": 4,
        "rect": [42.5, t3_title_start_y, 541.4, t3_title_end_y],
        "content": "text",
        "text": t3_title_text
    }));
    builder.push("object.label", json!({ "ids": [format!("${t3_title_step}.id")], "label": "#slot:text:table_title_3" }));

    let t3_step = builder.push("frame.create", json!({
        "spread": 4,
        "rect": [42.5, t3_table_start_y, 541.4, t3_table_end_y],
        "content": "text",
        "text": t3_tab
    }));
    builder.push("object.label", json!({ "ids": [format!("${t3_step}.id")], "label": "#slot:table:3" }));
    builder.push("text.select", json!({ "story": format!("${t3_step}.story"), "anchor": 0, "focus": t3_tab.chars().count() }));
    builder.push("table.convertFromText", json!({ "columnSeparator": "tab" }));

    let u5_full = get_clean_narrative("U5");
    let u5_start_step = builder.push("frame.create", json!({
        "spread": 4,
        "rect": [42.5, u5_start_y, 541.4, 785.2],
        "content": "text",
        "text": u5_full
    }));
    builder.push("object.label", json!({ "ids": [format!("${u5_start_step}.id")], "label": "#slot:text:narrative_body_5" }));

    // Page 9 (Spread 4 Right): U5 Cont + Table 4 Title + Table 4 Frame + U6 Narrative (dynamic flow)
    let u5_total_h = estimate_brs_narrative_height(&u5_full);
    let u5_p8_avail = 785.2 - u5_start_y;
    let u5_rem_h = (u5_total_h - u5_p8_avail).max(0.0);
    let (u5_p9_frame_h, t4_title_start_y) = if u5_rem_h > 10.0 {
        let end_y = (57.3 + u5_rem_h + 10.0).min(280.0);
        (end_y - 57.3, end_y + 14.0)
    } else {
        (0.0, 57.3)
    };

    let u5_cont_step = builder.push("frame.create", json!({
        "spread": 4,
        "rect": [649.1, 57.3, 1148.0, 57.3 + u5_p9_frame_h.max(20.0)],
        "content": "text"
    }));
    builder.push("object.label", json!({ "ids": [format!("${u5_cont_step}.id")], "label": "#slot:text:narrative_body_5_cont" }));
    builder.push("text.thread", json!({ "from": format!("${u5_start_step}.id"), "to": format!("${u5_cont_step}.id") }));

    let t4_raw = find_entry_content("T4").unwrap_or_default();
    let (t4_tab, t4_rows, t4_title) = parse_brs_csv_table(&t4_raw);
    let t4_title_text = t4_title.unwrap_or_else(|| "Tabel 4    Nilai Tukar Usaha Petani per Subsektor dan Persentase Perubahannya (2018=100)".to_string());

    let t4_title_end_y = t4_title_start_y + 23.0;
    let t4_table_start_y = t4_title_end_y + 2.0;
    let t4_num_rows = t4_rows.len().max(1);
    let t4_calc_h = (t4_num_rows as f64 * 14.85 + 2.0).min(350.0);
    let t4_table_end_y = t4_table_start_y + t4_calc_h;
    let u6_start_y = (t4_table_end_y + 14.0).min(730.0);

    let t4_title_step = builder.push("frame.create", json!({
        "spread": 4,
        "rect": [649.1, t4_title_start_y, 1148.0, t4_title_end_y],
        "content": "text",
        "text": t4_title_text
    }));
    builder.push("object.label", json!({ "ids": [format!("${t4_title_step}.id")], "label": "#slot:text:table_title_4" }));

    let t4_step = builder.push("frame.create", json!({
        "spread": 4,
        "rect": [649.1, t4_table_start_y, 1148.0, t4_table_end_y],
        "content": "text",
        "text": t4_tab
    }));
    builder.push("object.label", json!({ "ids": [format!("${t4_step}.id")], "label": "#slot:table:4" }));
    builder.push("text.select", json!({ "story": format!("${t4_step}.story"), "anchor": 0, "focus": t4_tab.chars().count() }));
    builder.push("table.convertFromText", json!({ "columnSeparator": "tab" }));

    let u6_full = get_clean_narrative("U6");
    let u6_step = builder.push("frame.create", json!({
        "spread": 4,
        "rect": [649.1, u6_start_y, 1148.0, 785.2],
        "content": "text",
        "text": u6_full
    }));
    builder.push("object.label", json!({ "ids": [format!("${u6_step}.id")], "label": "#slot:text:narrative_body_6" }));

    // Page 10 (Spread 5 Left): Infografis G1.png
    let g1_path = resolve_asset_path("G1.png", base_dirs);
    let g1_step = builder.push("frame.create", json!({
        "spread": 5,
        "rect": [42.5, 57.3, 541.4, 785.2],
        "content": "graphic"
    }));
    builder.push("object.label", json!({ "ids": [format!("${g1_step}.id")], "label": "#slot:image:1" }));
    if Path::new(&g1_path).exists() {
        builder.push("file.place", json!({ "frame": format!("${g1_step}.id"), "path": g1_path }));
        builder.push("object.fit", json!({ "mode": "fitProportionally", "ids": [format!("${g1_step}.id")] }));
        builder.push("object.fit", json!({ "mode": "centerContent", "ids": [format!("${g1_step}.id")] }));
    }

    builder.finish()
}

pub fn post_process_brs_document(doc_path: &Path, base_dirs: &[&Path]) -> Result<()> {
    if !doc_path.exists() {
        return Ok(());
    }

    let file = std::fs::File::open(doc_path)?;
    let mut zip = ZipArchive::new(file)?;

    let mut doc_json_bytes = Vec::new();
    let mut files_to_keep: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();

    for i in 0..zip.len() {
        if let Ok(mut entry) = zip.by_index(i) {
            let name = entry.name().to_string();
            let mut buf = Vec::new();
            if entry.read_to_end(&mut buf).is_ok() {
                if name == "document.json" {
                    doc_json_bytes = buf;
                } else if name != "mimetype" {
                    files_to_keep.insert(name, buf);
                }
            }
        }
    }

    if doc_json_bytes.is_empty() {
        return Ok(());
    }

    let mut doc: Value = serde_json::from_slice(&doc_json_bytes)?;

    // 0. Update document settings and page margins to match official BRS standard
    if let Some(settings) = doc.get_mut("settings").and_then(Value::as_object_mut) {
        settings.insert("horizontalUnits".to_string(), json!("millimeters"));
        settings.insert("verticalUnits".to_string(), json!("millimeters"));
        settings.insert("facingPages".to_string(), json!(true));
    }

    let brs_content_margins = json!({
        "top": 57.259842519685044,
        "bottom": 56.69291338582678,
        "inside": 53.85826771653544,
        "outside": 42.51968503937008
    });
    let brs_cover_margins = json!({
        "top": 36.0,
        "bottom": 36.0,
        "inside": 36.0,
        "outside": 36.0
    });

    if let Some(spreads) = doc.get_mut("spreads").and_then(Value::as_array_mut) {
        for (si, sp) in spreads.iter_mut().enumerate() {
            if let Some(pages) = sp.get_mut("pages").and_then(Value::as_array_mut) {
                for pg in pages {
                    if si == 0 {
                        pg["margins"] = brs_cover_margins.clone();
                    } else {
                        pg["margins"] = brs_content_margins.clone();
                    }
                }
            }
        }
    }

    if let Some(parents) = doc.get_mut("parents").and_then(Value::as_array_mut) {
        for (pi, parent) in parents.iter_mut().enumerate() {
            if let Some(pages) = parent.get_mut("pages").and_then(Value::as_array_mut) {
                for pg in pages {
                    if pi == 0 {
                        pg["margins"] = brs_cover_margins.clone();
                    } else {
                        pg["margins"] = brs_content_margins.clone();
                    }
                }
            }
        }
    }

    // 1. Swatches: inject #F68839, #F58838, #FEF7E8, #FEF0D2, #FBBF91
    if let Some(swatches) = doc.get_mut("swatches").and_then(Value::as_array_mut) {
        let existing: HashSet<String> = swatches
            .iter()
            .filter_map(|s| s.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();

        let make_rgb = |name: &str, r: f64, g: f64, b: f64| json!({
            "name": name,
            "value": { "kind": "color", "color": { "model": "rgb", "r": r, "g": g, "b": b }, "color_type": "process" },
            "locked": false,
            "named": true
        });

        if !existing.contains("#F68839") { swatches.push(make_rgb("#F68839", 0.9647, 0.5333, 0.2235)); }
        if !existing.contains("#F58838") { swatches.push(make_rgb("#F58838", 0.9608, 0.5333, 0.2196)); }
        if !existing.contains("#FEF0D2") { swatches.push(make_rgb("#FEF0D2", 0.9961, 0.9412, 0.8235)); }
        if !existing.contains("#FEF7E8") { swatches.push(make_rgb("#FEF7E8", 0.9961, 0.9686, 0.9098)); }
        if !existing.contains("#FBBF91") { swatches.push(make_rgb("#FBBF91", 0.9843, 0.7490, 0.5686)); }
    }

    // 1b. Document settings: cross-story numbered lists for BRS
    if let Some(settings) = doc.get_mut("settings").and_then(Value::as_object_mut) {
        settings.insert(
            "lists".to_string(),
            json!([{ "name": "BRS", "continueAcrossStories": true }]),
        );
    }

    // 2. Paragraph styles: inject Title, Heading 1, Heading 2, List Paragraph
    if let Some(styles) = doc.get_mut("styles").and_then(Value::as_object_mut) {
        let para_styles = styles.entry("paragraph").or_insert_with(|| json!([]));
        if let Some(arr) = para_styles.as_array_mut() {
            // Update or insert Title
            if let Some(h) = arr.iter_mut().find(|s| s.get("name") == Some(&json!("Title"))) {
                h["para"] = json!({ "spaceBefore": 0.0, "spaceAfter": 6.0 });
                h["chars"] = json!({ "fontFamily": "Lato", "fontStyle": "Bold", "size": 14.5, "fill": "#F68839" });
            } else {
                arr.push(json!({
                    "name": "Title",
                    "para": { "spaceBefore": 0.0, "spaceAfter": 6.0 },
                    "chars": { "fontFamily": "Lato", "fontStyle": "Bold", "size": 14.5, "fill": "#F68839" }
                }));
            }

            // Update or insert Heading 1 with numbered list style
            if let Some(h) = arr.iter_mut().find(|s| s.get("name") == Some(&json!("Heading 1"))) {
                h["para"] = json!({
                    "spaceBefore": 12.0,
                    "spaceAfter": 4.0,
                    "listType": "numbers",
                    "listName": "BRS",
                    "numberExpression": "^#.^t",
                    "listLevel": 1,
                    "restartNumbers": true,
                    "restartAfterLevel": 0
                });
                h["chars"] = json!({ "fontFamily": "Lato", "fontStyle": "Bold", "size": 13.0, "fill": "#F68839" });
            } else {
                arr.push(json!({
                    "name": "Heading 1",
                    "para": {
                        "spaceBefore": 12.0,
                        "spaceAfter": 4.0,
                        "listType": "numbers",
                        "listName": "BRS",
                        "numberExpression": "^#.^t",
                        "listLevel": 1,
                        "restartNumbers": true,
                        "restartAfterLevel": 0
                    },
                    "chars": { "fontFamily": "Lato", "fontStyle": "Bold", "size": 13.0, "fill": "#F68839" }
                }));
            }

            // Update or insert Heading 2 with numbered list style
            if let Some(h) = arr.iter_mut().find(|s| s.get("name") == Some(&json!("Heading 2"))) {
                h["para"] = json!({
                    "spaceBefore": 8.0,
                    "spaceAfter": 3.0,
                    "listType": "numbers",
                    "listName": "BRS",
                    "numberExpression": "^1.^#.^t",
                    "listLevel": 2,
                    "restartNumbers": true,
                    "restartAfterLevel": 0
                });
                h["chars"] = json!({ "fontFamily": "Lato", "fontStyle": "Bold", "size": 11.0, "fill": "#F68839" });
            } else {
                arr.push(json!({
                    "name": "Heading 2",
                    "para": {
                        "spaceBefore": 8.0,
                        "spaceAfter": 3.0,
                        "listType": "numbers",
                        "listName": "BRS",
                        "numberExpression": "^1.^#.^t",
                        "listLevel": 2,
                        "restartNumbers": true,
                        "restartAfterLevel": 0
                    },
                    "chars": { "fontFamily": "Lato", "fontStyle": "Bold", "size": 11.0, "fill": "#F68839" }
                }));
            }

            if !arr.iter().any(|s| s.get("name") == Some(&json!("List Paragraph"))) {
                arr.push(json!({
                    "name": "List Paragraph",
                    "para": { "align": "left", "leftIndent": 36.0, "rightIndent": 0.0, "spaceAfter": 8.0 },
                    "chars": { "size": 12.0 }
                }));
            }
        }

        let table_styles = styles.entry("table").or_insert_with(|| json!([]));
        if let Some(arr) = table_styles.as_array_mut() {
            let existing: HashSet<String> = arr
                .iter()
                .filter_map(|s| s.get("name").and_then(Value::as_str).map(str::to_string))
                .collect();
            if !existing.contains("Horizontal Rules") {
                arr.push(json!({
                    "name": "Horizontal Rules",
                    "border": { "weight": 0.0, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } }
                }));
            }
        }
    }

    // Find all narrative and table title story IDs from spreads items
    let mut narrative_story_ids = HashSet::new();
    let mut table_title_story_ids = HashSet::new();
    if let Some(spreads) = doc.get_mut("spreads").and_then(Value::as_array_mut) {
        for sp in spreads {
            if let Some(items) = sp.get_mut("items").and_then(Value::as_array_mut) {
                for it in items {
                    let lbl = it.get("label").and_then(Value::as_str).unwrap_or("");
                    if lbl.starts_with("#slot:text:narrative") {
                        if let Some(sid) = it.get("content").and_then(|c| c.get("story")).and_then(Value::as_u64) {
                            narrative_story_ids.insert(sid.to_string());
                        }
                    } else if lbl.starts_with("#slot:text:table_title") {
                        if let Some(cnt) = it.get_mut("content").and_then(Value::as_object_mut) {
                            let opts = cnt.entry("options").or_insert_with(|| json!({}));
                            opts["inset"] = json!([0.0, 0.0, 0.0, 0.0]);
                            opts["autoSize"] = json!("heightOnly");
                            if let Some(sid) = cnt.get("story").and_then(Value::as_u64) {
                                table_title_story_ids.insert(sid.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Format narrative stories and tables in `stories`
    if let Some(stories) = doc.get_mut("stories").and_then(Value::as_object_mut) {
        for (sid, st) in stories.iter_mut() {
            let text = st.get("text").and_then(Value::as_str).unwrap_or("").to_string();

            // Story 870: Front Cover Publication Title (Title + Month)
            if sid == "870" {
                let clen = text.chars().count();
                st["chars"] = json!([{
                    "len": clen,
                    "style": "[None]",
                    "over": {
                        "fill": "R=246 G=136 B=57",
                        "fontFamily": "Lato",
                        "fontStyle": "Bold",
                        "size": 24.0
                    }
                }]);
                continue;
            }

            // Story 323: Front Cover Release No. & Date
            if sid == "323" {
                let clen = text.chars().count();
                st["chars"] = json!([{
                    "len": clen,
                    "style": "[None]",
                    "over": {
                        "fontFamily": "Lato",
                        "size": 10.0
                    }
                }]);
                continue;
            }

            // Story 884: Executive Summary / Point Bullets on Inside Cover (Page 2)
            if sid == "884" {
                let mut lines: Vec<String> = text
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(String::from)
                    .collect();

                // If text only has 1 line or was default, try extracting dynamic items from base_dirs
                if lines.len() <= 1 {
                    let extracted = extract_brs_point_items(&Metadata::default(), None, base_dirs);
                    if !extracted.is_empty() {
                        lines = extracted;
                        st["text"] = Value::String(lines.join("\n"));
                    }
                }

                if !lines.is_empty() {
                    let mut paras = Vec::new();
                    for _ in &lines {
                        paras.push(json!({
                            "style": "[Basic Paragraph]",
                            "para": {
                                "align": "leftJustified",
                                "leftIndent": 18.0,
                                "firstLineIndent": -18.0,
                                "listType": "bullets",
                                "bulletChar": "\u{25a0}",
                                "bulletCharStyle": "Orange Bullet",
                                "listHang": true,
                                "spaceAfter": 8.0
                            }
                        }));
                    }
                    st["paras"] = Value::Array(paras);

                    let full_text = st.get("text").and_then(Value::as_str).unwrap_or("");
                    let full_len = full_text.chars().count();
                    st["chars"] = json!([{
                        "len": full_len,
                        "style": "[None]",
                        "over": {
                            "fontFamily": "Lato",
                            "fontStyle": "Regular",
                            "size": 12.0
                        }
                    }]);
                }
                continue;
            }

            // Case A: Table story
            if let Some(tables) = st.get_mut("tables").and_then(Value::as_object_mut) {
                for (_tid, tbl) in tables.iter_mut() {
                    tbl["style"] = json!("BRS6");
                    tbl["options"] = json!({
                        "direction": "leftToRight",
                        "border": { "weight": 0.0, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } },
                        "spaceBefore": 4.0,
                        "spaceAfter": -4.0,
                        "repeatHeader": true,
                        "repeatFooter": true
                    });

                    let nc = tbl.get("columns").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
                    let nr = tbl.get("rows").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);

                    let is_t1 = nc == 5
                        && tbl.get("cells").and_then(Value::as_array).is_some_and(|cells| {
                            cells.iter().any(|c| {
                                c.get("text")
                                    .and_then(|t| t.get("text"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .contains("Indeks Gabungan Subsektor")
                            })
                        });

                    // Distribute column widths
                    if nc == 5 {
                        if is_t1 {
                            tbl["columns"] = json!([
                                { "width": 40.0 }, { "width": 218.9 }, { "width": 80.0 }, { "width": 80.0 }, { "width": 80.0 }
                            ]);
                        } else {
                            tbl["columns"] = json!([
                                { "width": 120.0 }, { "width": 178.9 }, { "width": 66.67 }, { "width": 66.67 }, { "width": 66.67 }
                            ]);
                        }
                    } else if nc == 4 {
                        tbl["columns"] = json!([
                            { "width": 170.0 }, { "width": 109.63 }, { "width": 109.63 }, { "width": 109.63 }
                        ]);
                    }

                    // Format cells
                    if let Some(cells) = tbl.get_mut("cells").and_then(Value::as_array_mut) {
                        let header_row_count = if is_t1 { 2 } else { 1 };

                        for (i, cell) in cells.iter_mut().enumerate() {
                            let r = if nc > 0 { i / nc } else { 0 };
                            let c = if nc > 0 { i % nc } else { 0 };

                            cell["borderOverrides"] = json!([true, true, true, true]);
                            cell["strokeDefined"] = json!([true, true, true, true]);

                            if r < header_row_count {
                                // Header row
                                cell["fill"] = json!("#F58838");
                                cell["fillTint"] = json!(1.0);
                                cell["vj"] = json!("center");

                                // Strokes: top 1.0pt, bottom 0.5pt, left/right 0.0pt
                                cell["strokes"] = json!([
                                    { "weight": 1.0, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } },
                                    { "weight": 0.0, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } },
                                    { "weight": 0.5, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } },
                                    { "weight": 0.0, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } }
                                ]);

                                if let Some(txt_obj) = cell.get_mut("text").and_then(Value::as_object_mut) {
                                    txt_obj["paras"] = json!([{ "style": "[Basic Paragraph]", "para": { "align": "center" } }]);
                                    let clen = txt_obj.get("text").and_then(Value::as_str).map(|s| s.chars().count()).unwrap_or(0);
                                    txt_obj["chars"] = json!([{
                                        "len": clen,
                                        "style": "[None]",
                                        "over": { "fontFamily": "Lato", "fontStyle": "Bold", "size": 8.0, "fill": "[Paper]" }
                                    }]);
                                }
                            } else {
                                // Body row
                                let data_idx = r - header_row_count;
                                let is_even_data = data_idx % 2 == 0;
                                let is_last = r == nr - 1;

                                // Standard alternating zebra fill for BRS tables (even data row is #FEF0D2, odd is #FEF7E8)
                                let fill = if is_even_data {
                                    "#FEF0D2"
                                } else {
                                    "#FEF7E8"
                                };

                                let cell_txt = cell.get("text").and_then(|t| t.get("text")).and_then(Value::as_str).unwrap_or("").trim().to_string();
                                let bottom_w = if is_last { 1.0 } else { 0.0 };
                                let align = if c == 0 && (nc > 4 || !cell_txt.chars().all(|ch| ch.is_ascii_digit() || ch == '.' || ch == '-')) {
                                    "left"
                                } else if c == 1 && nc == 5 {
                                    "left"
                                } else {
                                    "center"
                                };

                                cell["fill"] = json!(fill);
                                cell["fillTint"] = json!(1.0);
                                cell["vj"] = json!("center");
                                cell["strokes"] = json!([
                                    { "weight": 0.0, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } },
                                    { "weight": 0.0, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } },
                                    { "weight": bottom_w, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } },
                                    { "weight": 0.0, "color": "[Black]", "tint": 1.0, "kind": { "kind": "solid" } }
                                ]);

                                if let Some(txt_obj) = cell.get_mut("text").and_then(Value::as_object_mut) {
                                    txt_obj["paras"] = json!([{ "style": "[Basic Paragraph]", "para": { "align": align } }]);
                                    let clen = txt_obj.get("text").and_then(Value::as_str).map(|s| s.chars().count()).unwrap_or(0);
                                    txt_obj["chars"] = json!([{
                                        "len": clen,
                                        "style": "[None]",
                                        "over": { "fontFamily": "Lato", "size": 8.0 }
                                    }]);
                                }
                            }
                        }

                        // Specific cell spans for Table 1
                        if is_t1 && cells.len() >= 5 {
                            cells[0]["rowSpan"] = json!(2);
                            cells[0]["colSpan"] = json!(1);
                            cells[1]["rowSpan"] = json!(2);
                            cells[1]["colSpan"] = json!(1);
                            cells[2]["rowSpan"] = json!(1);
                            cells[2]["colSpan"] = json!(3);
                        }
                    }
                }
                continue;
            }

            // Case B: Narrative story
            if narrative_story_ids.contains(sid) {
                let lines: Vec<&str> = text.split('\n').collect();
                let num_lines = lines.len();
                let mut new_paras = Vec::new();
                let mut new_chars = Vec::new();

                for (idx, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    let starts_abc = trimmed.starts_with("A. ") || trimmed.starts_with("B. ") || trimmed.starts_with("C. ");
                    let is_h1 = trimmed == "Nilai Tukar Petani (NTP)"
                        || trimmed.starts_with("Indeks Harga yang Diterima")
                        || trimmed.starts_with("Indeks Harga yang Dibayar")
                        || trimmed.starts_with("NTP Menurut Subsektor")
                        || trimmed.starts_with("Perbandingan NTP")
                        || trimmed.starts_with("Nilai Tukar Usaha Petani");
                    let is_h2 = trimmed.starts_with("NTP Tanaman Pangan")
                        || trimmed.starts_with("NTP Tanaman Hortikultura")
                        || trimmed.starts_with("NTP Tanaman Perkebunan")
                        || trimmed.starts_with("NTP Peternakan")
                        || trimmed.starts_with("NTP Perikanan");

                    let clen = line.len() + if idx + 1 < num_lines { 1 } else { 0 };

                    if starts_abc {
                        new_paras.push(json!({ "style": "Title", "para": { "spaceBefore": 0.0, "spaceAfter": 4.0 } }));
                        new_chars.push(json!({
                            "len": clen,
                            "style": "[None]",
                            "over": { "fontFamily": "Lato", "fontStyle": "Bold", "size": 14.5, "leading": { "kind": "points", "value": 15.0 }, "fill": "#F68839" }
                        }));
                    } else if is_h1 {
                        new_paras.push(json!({ "style": "Heading 1", "para": { "spaceBefore": 18.0, "spaceAfter": 4.0 } }));
                        new_chars.push(json!({
                            "len": clen,
                            "style": "[None]",
                            "over": { "fontFamily": "Lato", "fontStyle": "Bold", "size": 13.0, "leading": { "kind": "points", "value": 15.0 }, "fill": "#F68839" }
                        }));
                    } else if is_h2 {
                        new_paras.push(json!({ "style": "Heading 2", "para": { "spaceBefore": 8.0, "spaceAfter": 4.0 } }));
                        new_chars.push(json!({
                            "len": clen,
                            "style": "[None]",
                            "over": { "fontFamily": "Lato", "fontStyle": "Bold", "size": 11.0, "leading": { "kind": "points", "value": 15.0 }, "fill": "#F68839" }
                        }));
                    } else {
                        new_paras.push(json!({ "style": "[Basic Paragraph]", "para": { "align": "leftJustified", "spaceAfter": 8.0 } }));
                        new_chars.push(json!({
                            "len": clen,
                            "style": "[None]",
                            "over": { "fontFamily": "Lato", "size": 12.0, "leading": { "kind": "points", "value": 15.0 } }
                        }));
                    }
                }

                if !new_paras.is_empty() {
                    st["paras"] = json!(new_paras);
                    st["chars"] = json!(new_chars);
                }
                continue;
            }

            // Case C: Table title story
            if table_title_story_ids.contains(sid) {
                let clean_text = text.replace('\t', " ");
                let parts: Vec<&str> = clean_text.split_whitespace().collect();
                let new_text = if parts.len() >= 3 && (parts[0].to_lowercase().starts_with("tabel") || parts[0].to_lowercase().starts_with("table")) {
                    let prefix = format!("{} {}", parts[0], parts[1]);
                    let body = parts[2..].join(" ");
                    format!("{prefix}\t{body}")
                } else {
                    parts.join(" ")
                };
                st["text"] = Value::String(new_text.clone());
                st["paras"] = json!([{
                    "style": "[Basic Paragraph]",
                    "para": {
                        "leftIndent": 48.0,
                        "firstLineIndent": -48.0,
                        "spaceAfter": 4.0,
                        "tabs": [{ "position": 48.0, "alignment": "left" }]
                    }
                }]);
                let clen = new_text.chars().count();
                st["chars"] = json!([{
                    "len": clen,
                    "style": "[None]",
                    "over": { "fontFamily": "Lato", "fontStyle": "Bold", "size": 10.5, "fill": "#F58838" }
                }]);
                continue;
            }

            // Case D: Back cover agency
            if sid == "901" {
                st["chars"] = json!([{
                    "len": text.len(),
                    "style": "[None]",
                    "over": { "fontFamily": "Arial", "fontStyle": "Bold Italic", "size": 10.0, "fill": "C=100 M=0 Y=0 K=0" }
                }]);
                continue;
            }
        }
    }

    // 4. Ensure C1 vector chart (group 756), stories and swatches are copied into Spread 1
    let mut chart_group_opt: Option<Value> = None;
    let mut chart_stories_opt: Option<Value> = None;
    let mut chart_swatches_opt: Option<Vec<Value>> = None;

    // A. Check embedded or disk c1_chart.json
    let c1_json_bytes = crate::embedded_assets::get_embedded_asset("templates/covers/brs/c1_chart.json")
        .or_else(|| crate::embedded_assets::get_embedded_asset("assets/templates/covers/brs/c1_chart.json"))
        .or_else(|| {
            let p = resolve_asset_path("assets/templates/covers/brs/c1_chart.json", base_dirs);
            if Path::new(&p).exists() {
                std::fs::read(&p).ok()
            } else {
                None
            }
        });

    if let Some(bytes) = c1_json_bytes {
        if let Ok(data) = serde_json::from_slice::<Value>(&bytes) {
            chart_group_opt = data.get("chart_group").cloned();
            chart_stories_opt = data.get("stories").cloned();
            chart_swatches_opt = data.get("swatches").and_then(Value::as_array).cloned();
        }
    }

    // B. Fallback: check reference v1 designcraft
    let v1_candidate = resolve_asset_path("samples/brs/ntp202610_v1.designcraft", base_dirs);
    if chart_group_opt.is_none() && Path::new(&v1_candidate).exists() {
        if let Ok(v1_file) = std::fs::File::open(&v1_candidate) {
            if let Ok(mut v1_zip) = ZipArchive::new(v1_file) {
                let mut v1_bytes = Vec::new();
                if let Ok(mut e) = v1_zip.by_name("document.json") {
                    let _ = e.read_to_end(&mut v1_bytes);
                    if let Ok(v1_doc) = serde_json::from_slice::<Value>(&v1_bytes) {
                        if let Some(v1_sp1) = v1_doc.get("spreads").and_then(Value::as_array).and_then(|s| s.get(1)) {
                            if let Some(chart_group) = v1_sp1.get("items").and_then(Value::as_array).and_then(|items| items.iter().find(|it| it.get("id") == Some(&json!(756)))) {
                                chart_group_opt = Some(chart_group.clone());

                                fn collect_story_ids(item: &Value, set: &mut HashSet<u64>) {
                                    if let Some(cnt) = item.get("content").and_then(Value::as_object) {
                                        if let Some(sid) = cnt.get("story").and_then(Value::as_u64) {
                                            set.insert(sid);
                                        }
                                        if let Some(arr) = cnt.get("items").and_then(Value::as_array) {
                                            for sub in arr {
                                                collect_story_ids(sub, set);
                                            }
                                        }
                                    }
                                }

                                let mut chart_story_ids = HashSet::new();
                                collect_story_ids(chart_group, &mut chart_story_ids);

                                if let Some(v1_stories) = v1_doc.get("stories").and_then(Value::as_object) {
                                    let mut extracted_stories = serde_json::Map::new();
                                    for sid in chart_story_ids {
                                        let sid_str = sid.to_string();
                                        if let Some(story_val) = v1_stories.get(&sid_str) {
                                            extracted_stories.insert(sid_str, story_val.clone());
                                        }
                                    }
                                    chart_stories_opt = Some(Value::Object(extracted_stories));
                                }
                                if let Some(sws) = v1_doc.get("swatches").and_then(Value::as_array) {
                                    chart_swatches_opt = Some(sws.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(chart_group) = chart_group_opt {
        if let Some(sp1) = doc.get_mut("spreads").and_then(Value::as_array_mut).and_then(|s| s.get_mut(1)) {
            if let Some(items) = sp1.get_mut("items").and_then(Value::as_array_mut) {
                if !items.iter().any(|it| it.get("id") == Some(&json!(756))) {
                    items.retain(|it| it.get("label").and_then(Value::as_str) != Some("#slot:chart:1"));
                    items.push(chart_group);
                }
            }
        }
    }

    if let Some(stories_val) = chart_stories_opt {
        if let Some(st_map) = stories_val.as_object() {
            if let Some(our_stories) = doc.get_mut("stories").and_then(Value::as_object_mut) {
                for (sid_str, story_val) in st_map {
                    our_stories.insert(sid_str.clone(), story_val.clone());
                }
            }
        }
    }

    if let Some(sws) = chart_swatches_opt {
        if let Some(our_swatches) = doc.get_mut("swatches").and_then(Value::as_array_mut) {
            for sw in sws {
                if let Some(n) = sw.get("name").and_then(Value::as_str) {
                    if !our_swatches.iter().any(|s| s.get("name").and_then(Value::as_str) == Some(n)) {
                        our_swatches.push(sw);
                    }
                }
            }
        }
    }

    // Ensure all hex colors referenced anywhere in the document exist in doc["swatches"]
    ensure_all_hex_swatches(&mut doc);

    // 5. Ensure Spread 5 has G1.png graphic frame and asset properly embedded
    let g1_disk = resolve_asset_path("G1.png", base_dirs);
    let g1_bytes = if Path::new(&g1_disk).exists() {
        std::fs::read(&g1_disk).ok()
    } else {
        None
    };

    if let Some(sp5) = doc.get_mut("spreads").and_then(Value::as_array_mut).and_then(|s| s.get_mut(5)) {
        if let Some(items) = sp5.get_mut("items").and_then(Value::as_array_mut) {
            let mut found_g1 = false;
            for item in items.iter_mut() {
                if item.get("content").and_then(|c| c.get("type")).and_then(Value::as_str) == Some("graphic") {
                    item["content"]["asset"] = json!(838);
                    item["content"]["size"] = json!([2168.0, 2959.0]);
                    item["content"]["xf"] = json!([0.2301188366214371, 0.0, 0.0, 0.2301188366214371, 42.51968503937008, 80.76752767527682]);
                    item["content"]["autoFit"] = json!("none");
                    item["content"]["fitAlign"] = json!(4);
                    item["content"]["crop"] = json!([0.0, 0.0, 0.0, 0.0]);
                    item["path"] = json!({
                        "subpaths": [{
                            "anchors": [
                                { "p": [42.51968503937008, 57.259842519685044] },
                                { "p": [541.4173228346457, 57.259842519685044] },
                                { "p": [541.4173228346457, 785.196850393701] },
                                { "p": [42.51968503937008, 785.196850393701] }
                            ],
                            "closed": true
                        }]
                    });
                    found_g1 = true;
                    break;
                }
            }
            if !found_g1 {
                items.push(json!({
                    "id": 837,
                    "layer": 1,
                    "shape": "rectangle",
                    "path": {
                        "subpaths": [{
                            "anchors": [
                                { "p": [42.51968503937008, 57.259842519685044] },
                                { "p": [541.4173228346457, 57.259842519685044] },
                                { "p": [541.4173228346457, 785.196850393701] },
                                { "p": [42.51968503937008, 785.196850393701] }
                            ],
                            "closed": true
                        }]
                    },
                    "xf": [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    "fill": { "swatch": "[None]", "tint": 1.0, "overprint": false },
                    "stroke": { "swatch": "[None]", "tint": 1.0, "weight": 1.0, "type": { "kind": "solid" }, "align": "center", "cap": "butt", "join": "miter", "miterLimit": 4.0, "start": "none", "end": "none", "gapSwatch": "[None]", "gapTint": 1.0, "overprint": false },
                    "corners": { "corners": [{ "shape": "none", "size": 0.0 }, { "shape": "none", "size": 0.0 }, { "shape": "none", "size": 0.0 }, { "shape": "none", "size": 0.0 }] },
                    "opacity": 1.0,
                    "blend": "Normal",
                    "content": {
                        "type": "graphic",
                        "asset": 838,
                        "size": [2168.0, 2959.0],
                        "xf": [0.2301188366214371, 0.0, 0.0, 0.2301188366214371, 42.51968503937008, 80.76752767527682],
                        "autoFit": "none",
                        "fitAlign": 4,
                        "crop": [0.0, 0.0, 0.0, 0.0]
                    }
                }));
            }
        }
    }

    if let Some(assets) = doc.get_mut("assets").and_then(Value::as_object_mut) {
        assets.insert("838".to_string(), json!({
            "id": 838,
            "name": "G1.png",
            "mime": "image/png",
            "link": g1_disk,
            "pixels": [2168, 2959]
        }));
    }

    if let Some(b) = g1_bytes {
        files_to_keep.insert("assets/838.png".to_string(), b.clone());
        files_to_keep.insert("assets/1014.png".to_string(), b);
    }

    // 6. Repatriate jumped Back Cover decorative shape (item 907)
    // In brs.designcraft, item 907 belongs to the Back Cover (spread 2).
    // Because its path coordinates extend into negative X, designcraft-cli's layout.pages.insert
    // mistakenly migrates it into Spread 1 (Pages 2-3).
    // We remove it from body spreads and restore it to the final Back Cover spread with its original transform.
    if let Some(spreads) = doc.get_mut("spreads").and_then(Value::as_array_mut) {
        let total_spreads = spreads.len();
        if total_spreads > 2 {
            let mut extracted_907: Option<Value> = None;
            for sp in spreads.iter_mut().take(total_spreads - 1) {
                if let Some(items) = sp.get_mut("items").and_then(Value::as_array_mut) {
                    if let Some(pos) = items.iter().position(|it| it.get("id") == Some(&json!(907))) {
                        let mut it = items.remove(pos);
                        it["xf"] = json!([1.0, 0.0, 0.0, 1.0, -15.724409448818847, 174.0]);
                        extracted_907 = Some(it);
                        break;
                    }
                }
            }
            if let Some(it_907) = extracted_907 {
                if let Some(last_sp) = spreads.last_mut() {
                    if let Some(items) = last_sp.get_mut("items").and_then(Value::as_array_mut) {
                        if !items.iter().any(|it| it.get("id") == Some(&json!(907))) {
                            // In the original template z-order, item 907 sits in front of theme2 graphic frame 537,
                            // yellow circle 539, and decorative groups 857 & 856, immediately before item 845.
                            let insert_pos = items
                                .iter()
                                .position(|it| it.get("id") == Some(&json!(845)))
                                .unwrap_or_else(|| items.len().min(4));
                            items.insert(insert_pos, it_907);
                        }
                    }
                }
            }
        }
    }

    // Copy any missing assets and previews from v1 reference document
    if Path::new(&v1_candidate).exists() {
        if let Ok(v1_file) = std::fs::File::open(&v1_candidate) {
            if let Ok(mut v1_zip) = ZipArchive::new(v1_file) {
                for vi in 0..v1_zip.len() {
                    if let Ok(mut ve) = v1_zip.by_index(vi) {
                        let vname = ve.name().to_string();
                        if vname != "document.json" && vname != "mimetype" && !files_to_keep.contains_key(&vname) {
                            let mut vbuf = Vec::new();
                            if ve.read_to_end(&mut vbuf).is_ok() {
                                files_to_keep.insert(vname, vbuf);
                            }
                        }
                    }
                }
            }
        }
    }

    // 6. Sort stories in the order text frames appear on the spreads
    // DesignCraft's numbering engine resolves cross-story lists in document.json story map order.
    let mut spread_story_ids = Vec::new();
    if let Some(spreads) = doc.get("spreads").and_then(Value::as_array) {
        for sp in spreads {
            if let Some(items) = sp.get("items").and_then(Value::as_array) {
                for it in items {
                    if let Some(cnt) = it.get("content") {
                        if cnt.get("type").and_then(Value::as_str) == Some("text") {
                            if let Some(sid) = cnt.get("story").and_then(Value::as_u64) {
                                let sid_str = sid.to_string();
                                if !spread_story_ids.contains(&sid_str) {
                                    spread_story_ids.push(sid_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(stories_map) = doc.get_mut("stories").and_then(Value::as_object_mut) {
        let mut old_map = std::mem::take(stories_map);
        let mut sorted_stories = serde_json::Map::new();
        for sid in spread_story_ids {
            if let Some(sval) = old_map.remove(&sid) {
                sorted_stories.insert(sid, sval);
            }
        }
        for (sid, sval) in old_map {
            sorted_stories.insert(sid, sval);
        }
        *stories_map = sorted_stories;
    }

    // 7. Ensure all hex swatches are defined in doc.swatches, then write back to doc_path
    ensure_all_hex_swatches(&mut doc);

    let updated_json = serde_json::to_vec_pretty(&doc)?;
    let tmp_path = doc_path.with_extension("tmp.designcraft");
    {
        let out_file = std::fs::File::create(&tmp_path)?;
        let mut zip_out = ZipWriter::new(out_file);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip_out.start_file("mimetype", SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored))?;
        zip_out.write_all(b"application/vnd.designcraft.document")?;
        zip_out.start_file("document.json", opts)?;
        zip_out.write_all(&updated_json)?;

        for (name, data) in &files_to_keep {
            zip_out.start_file(name, opts)?;
            zip_out.write_all(data)?;
        }
        zip_out.finish()?;
    }

    std::fs::rename(&tmp_path, doc_path)?;
    Ok(())
}

pub fn ensure_all_hex_swatches(doc: &mut Value) {
    let mut needed: HashSet<String> = HashSet::new();

    fn collect_from_item(item: &Value, needed: &mut HashSet<String>) {
        if let Some(swatch) = item.get("fill").and_then(|f| f.get("swatch")).and_then(Value::as_str) {
            if swatch.starts_with('#') && swatch.len() == 7 {
                needed.insert(swatch.to_string());
            }
        }
        if let Some(swatch) = item.get("stroke").and_then(|s| s.get("swatch")).and_then(Value::as_str) {
            if swatch.starts_with('#') && swatch.len() == 7 {
                needed.insert(swatch.to_string());
            }
        }
        if let Some(items) = item.get("content").and_then(|c| c.get("items")).and_then(Value::as_array) {
            for sub in items {
                collect_from_item(sub, needed);
            }
        }
    }

    if let Some(spreads) = doc.get("spreads").and_then(Value::as_array) {
        for sp in spreads {
            if let Some(items) = sp.get("items").and_then(Value::as_array) {
                for it in items {
                    collect_from_item(it, &mut needed);
                }
            }
        }
    }

    if let Some(stories) = doc.get("stories").and_then(Value::as_object) {
        for (_sid, story) in stories {
            if let Some(chars) = story.get("chars").and_then(Value::as_array) {
                for ch in chars {
                    if let Some(fill) = ch.get("over").and_then(|o| o.get("fill")).and_then(Value::as_str) {
                        if fill.starts_with('#') && fill.len() == 7 {
                            needed.insert(fill.to_string());
                        }
                    }
                    if let Some(stroke) = ch.get("over").and_then(|o| o.get("stroke")).and_then(Value::as_str) {
                        if stroke.starts_with('#') && stroke.len() == 7 {
                            needed.insert(stroke.to_string());
                        }
                    }
                }
            }
        }
    }

    if doc.get("swatches").is_none() {
        doc["swatches"] = json!([]);
    }
    if let Some(swatches) = doc.get_mut("swatches").and_then(Value::as_array_mut) {
        for hex in needed {
            let exists = swatches.iter().any(|s| {
                s.get("name").and_then(Value::as_str) == Some(&hex)
            });
            if !exists {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[1..3], 16),
                    u8::from_str_radix(&hex[3..5], 16),
                    u8::from_str_radix(&hex[5..7], 16),
                ) {
                    swatches.push(json!({
                        "name": hex,
                        "value": {
                            "kind": "color",
                            "color": {
                                "model": "rgb",
                                "r": r as f64 / 255.0,
                                "g": g as f64 / 255.0,
                                "b": b as f64 / 255.0
                            },
                            "color_type": "process"
                        },
                        "locked": false,
                        "named": true
                    }));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_all_hex_swatches_synthesizes_missing_colors() {
        let mut doc = json!({
            "spreads": [{
                "items": [{
                    "id": 1,
                    "fill": { "swatch": "#418AB3" },
                    "stroke": { "swatch": "#002060" }
                }]
            }],
            "stories": {
                "10": {
                    "chars": [{
                        "len": 5,
                        "over": { "fill": "#D97706" }
                    }]
                }
            },
            "swatches": [
                { "name": "[None]", "value": { "kind": "none" } }
            ]
        });

        ensure_all_hex_swatches(&mut doc);

        let swatches = doc.get("swatches").and_then(Value::as_array).unwrap();
        assert!(swatches.iter().any(|s| s.get("name").and_then(Value::as_str) == Some("#418AB3")));
        assert!(swatches.iter().any(|s| s.get("name").and_then(Value::as_str) == Some("#002060")));
        assert!(swatches.iter().any(|s| s.get("name").and_then(Value::as_str) == Some("#D97706")));

        let b3 = swatches.iter().find(|s| s.get("name").and_then(Value::as_str) == Some("#418AB3")).unwrap();
        let color = b3.get("value").and_then(|v| v.get("color")).unwrap();
        assert_eq!(color.get("model").and_then(Value::as_str), Some("rgb"));
        let r = color.get("r").and_then(Value::as_f64).unwrap();
        assert!((r - (0x41 as f64 / 255.0)).abs() < 1e-4);
    }
}
