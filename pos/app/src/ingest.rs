use anyhow::{anyhow, bail, Context, Result};
use calamine::Reader as _;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use std::path::Path;

pub struct OutFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub fn ingest(path: &Path) -> Result<Vec<OutFile>> {
    ingest_with_profile(path, "")
}

/// Ingest with an active publication profile, so XLSX sheet routing
/// (e.g. BRS chart sheets) can influence converted CSV file names.
pub fn ingest_with_profile(path: &Path, profile_id: &str) -> Result<Vec<OutFile>> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("bad file name: {}", path.display()))?
        .to_owned();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "docx" => Ok(vec![OutFile { name: format!("md/{stem}.md"), bytes: docx_to_md(&bytes)?.into_bytes() }]),
        "rtf" => Ok(vec![OutFile { name: format!("md/{stem}.md"), bytes: rtf_to_md(&bytes)?.into_bytes() }]),
        "xlsx" | "xlsm" => xlsx_to_csv(path, profile_id),
        other => bail!("unsupported source type .{other} (use docx, rtf or xlsx)"),
    }
}

fn attr_bytes(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<Vec<u8>> {
    for a in e.attributes().flatten() {
        if a.key.as_ref() == key {
            return Some(a.value.to_vec());
        }
    }
    None
}

fn prop_on(e: &quick_xml::events::BytesStart) -> bool {
    !matches!(attr_bytes(e, b"w:val").as_deref(), Some(b"0") | Some(b"false"))
}

fn docx_to_md(bytes: &[u8]) -> Result<String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| anyhow!("not a valid docx (zip): {e}"))?;
    let mut xml_bytes = Vec::new();
    archive
        .by_name("word/document.xml")
        .map_err(|e| anyhow!("docx has no word/document.xml: {e}"))?
        .read_to_end(&mut xml_bytes)
        .context("read document.xml")?;
    let xml = String::from_utf8_lossy(&xml_bytes).into_owned();

    let mut reader = Reader::from_str(&xml);
    let mut out = String::new();
    let mut para = String::new();
    let mut prefix = String::new();
    let mut in_list = false;
    let mut bold = false;
    let mut italic = false;
    let mut in_text = false;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e) => match e.name().as_ref() {
                b"w:p" => {
                    para.clear();
                    prefix.clear();
                    in_list = false;
                    bold = false;
                    italic = false;
                    in_text = false;
                }
                b"w:pStyle" => {
                    if let Some(val) = attr_bytes(&e, b"w:val") {
                        let style = String::from_utf8_lossy(&val).into_owned();
                        if let Some(level) = style.strip_prefix("Heading").and_then(|n| n.parse::<u32>().ok()) {
                            prefix = format!("{} ", "#".repeat(level.clamp(1, 6) as usize));
                        } else if style == "Title" {
                            prefix = "# ".to_owned();
                        } else if style == "Subtitle" {
                            prefix = "### ".to_owned();
                        } else if style == "Quote" || style == "IntenseQuote" || style == "Block Text" {
                            prefix = "> ".to_owned();
                        } else if style == "ListParagraph" {
                            // Numbered/bulleted lists carry `w:numPr` too; the plain
                            // style alone still reads as a bullet row.
                            prefix = "- ".to_owned();
                        }
                    }
                }
                b"w:numPr" => in_list = true,
                b"w:b" => bold = prop_on(&e),
                b"w:i" => italic = prop_on(&e),
                b"w:t" => in_text = true,
                b"w:tab" => para.push(' '),
                _ => {}
            },
            Event::Text(t) => {
                if in_text {
                    let text = t.unescape()?.trim().to_owned();
                    if !text.is_empty() {
                        let wrapped = match (bold, italic) {
                            (true, true) => format!("***{text}***"),
                            (true, false) => format!("**{text}**"),
                            (false, true) => format!("_{text}_"),
                            (false, false) => text,
                        };
                        if !para.is_empty() {
                            para.push(' ');
                        }
                        para.push_str(&wrapped);
                    }
                }
            }
            Event::End(e) => match e.name().as_ref() {
                b"w:p" => {
                    let mut line = para.trim().to_owned();
                    if !line.is_empty() {
                        if in_list {
                            line.insert_str(0, "- ");
                        } else if !prefix.is_empty() {
                            line.insert_str(0, &prefix);
                        }
                        out.push_str(&line);
                        out.push_str("\n\n");
                    }
                }
                b"w:b" => bold = false,
                b"w:i" => italic = false,
                b"w:t" => in_text = false,
                b"w:r" => {
                    bold = false;
                    italic = false;
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(out)
}

/// Maps a normalised RTF style name to a Markdown paragraph prefix, or `None` for Normal/unstyled.
fn style_name_to_md_prefix(name: &str) -> Option<&'static str> {
    for part in name.split(',') {
        let n = part.trim().to_lowercase();
        let prefix = match n.as_str() {
            "title" | "titel" | "document title" | "judul dokumen" => Some("# "),
            "subtitle" | "sub title" | "sub-title" | "document subtitle" => Some("### "),
            s if s.starts_with("heading 1") || s == "h1" || s == "judul 1" || s == "head 1" => Some("# "),
            s if s.starts_with("heading 2") || s == "h2" || s == "judul 2" || s == "head 2" => Some("## "),
            s if s.starts_with("heading 3") || s == "h3" || s == "judul 3" || s == "head 3" => Some("### "),
            s if s.starts_with("heading 4") || s == "h4" || s == "judul 4" || s == "head 4" => Some("#### "),
            s if s.starts_with("heading 5") || s == "h5" || s == "judul 5" || s == "head 5" => Some("##### "),
            s if s.starts_with("heading 6") || s == "h6" || s == "judul 6" || s == "head 6" => Some("###### "),
            s if s.contains("quote") || s.contains("kutipan") || s == "block text" || s == "blockquote" => Some("> "),
            s if s.starts_with("list bullet") || s == "bullet" || s.starts_with("daftar berpoin") => Some("- "),
            _ => None,
        };
        if prefix.is_some() {
            return prefix;
        }
    }
    None
}

/// Pre-parses the RTF stylesheet block, returning a map of style-index → Markdown prefix.
///
/// Each style entry in the stylesheet looks like `{\sN \ctrl... Style Name;}`.
/// The style name is the trailing plain text segment before the closing `;`.
fn rtf_parse_stylesheet(data: &str) -> std::collections::HashMap<u32, String> {
    let mut map = std::collections::HashMap::new();
    let Some(ss_off) = data.find("\\stylesheet") else { return map };

    // At `\stylesheet`, we are already inside the stylesheet group `{\stylesheet...}`.
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
                        if entry[ei] == '\\' {
                            if !seg.trim().is_empty() {
                                segments.push(std::mem::take(&mut seg));
                            } else {
                                seg.clear();
                            }
                            ei += 1;
                            if ei < entry.len() && !entry[ei].is_ascii_alphabetic() {
                                ei += 1;
                                continue;
                            }
                            let w_start = ei;
                            while ei < entry.len() && entry[ei].is_ascii_alphabetic() { ei += 1; }
                            let word: String = entry[w_start..ei].iter().collect();
                            let mut param = String::new();
                            while ei < entry.len() && (entry[ei].is_ascii_digit() || entry[ei] == '-') {
                                param.push(entry[ei]);
                                ei += 1;
                            }
                            if ei < entry.len() && entry[ei] == ' ' { ei += 1; }
                            if word == "s" && style_idx.is_none() {
                                style_idx = param.parse().ok().or(Some(0));
                            } else if word == "outlinelevel" {
                                outline_lvl = param.parse().ok();
                            }
                        } else if entry[ei] == '{' || entry[ei] == '}' {
                            ei += 1;
                        } else {
                            seg.push(entry[ei]);
                            ei += 1;
                        }
                    }
                    if !seg.trim().is_empty() { segments.push(seg); }

                    let name = segments.last()
                        .map(|s| s.trim().trim_end_matches(';').trim().to_string())
                        .unwrap_or_default();

                    let prefix = style_name_to_md_prefix(&name).map(|s| s.to_string()).or_else(|| {
                        outline_lvl.map(|lvl| format!("{} ", "#".repeat((lvl + 1).clamp(1, 6) as usize)))
                    });

                    if let (Some(idx), Some(pfx)) = (style_idx, prefix) {
                        map.insert(idx, pfx);
                    }
                    in_entry = false;
                } else if in_entry {
                    entry.push(c);
                }
                depth -= 1;
                if depth <= 0 { break; }
            }
            _ if in_entry => entry.push(c),
            _ => {}
        }
    }

    map
}

fn rtf_to_md(bytes: &[u8]) -> Result<String> {
    let data = String::from_utf8_lossy(bytes);
    // Pre-parse stylesheet so we can map \sN control words to MD prefixes.
    let style_map = rtf_parse_stylesheet(&data);

    let mut out = String::new();
    let mut para = String::new();
    let mut run = String::new();
    let mut run_bold = false;
    let mut run_italic = false;
    let mut bold = 0usize;
    let mut italic = 0usize;
    let mut skip_depth = 0usize;
    // Markdown prefix for the current paragraph (e.g. "## ", "> ").
    // Set by \sN / \outlinelevelN, cleared by \pard and after each flush.
    let mut para_prefix = String::new();
    let chars: Vec<char> = data.chars().collect();
    let mut i = 0usize;

    // Groups we skip entirely during the main pass (stylesheet already pre-parsed above).
    let skipped = ["fonttbl", "colortbl", "stylesheet", "info", "pict", "objdata",
                   "header", "footer", "listtable", "listoverridetable", "generator",
                   "pntext", "pn"];

    fn push_run(para: &mut String, run: &mut String, bold: bool, italic: bool) {
        if run.is_empty() {
            return;
        }
        let text = std::mem::take(run);
        let wrap = bold || italic;
        let core_start = if wrap { text.len() - text.trim_start().len() } else { 0 };
        let core_end = if wrap { text.trim_end().len() } else { text.len() };
        para.push_str(&text[..core_start]);
        if core_start < core_end {
            let core = &text[core_start..core_end];
            let wrapped = match (bold, italic) {
                (true, true) => format!("***{core}***"),
                (true, false) => format!("**{core}**"),
                (false, true) => format!("_{core}_"),
                (false, false) => core.to_owned(),
            };
            para.push_str(&wrapped);
        }
        para.push_str(&text[core_end..]);
    }

    fn transition(para: &mut String, run: &mut String, run_bold: &mut bool, run_italic: &mut bool, bold: bool, italic: bool) {
        if bold != *run_bold || italic != *run_italic {
            push_run(para, run, *run_bold, *run_italic);
            *run_bold = bold;
            *run_italic = italic;
        }
    }

    fn flush(para: &mut String, run: &mut String, run_bold: &mut bool, run_italic: &mut bool,
             out: &mut String, prefix: &mut String) {
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
        prefix.clear(); // each paragraph resets; next \pard+\sN will re-establish it
    }

    while i < chars.len() {
        if skip_depth > 0 {
            match chars[i] {
                '{' => {
                    skip_depth += 1;
                }
                '}' => skip_depth -= 1,
                _ => {}
            }
            i += 1;
            continue;
        }
        let c = chars[i];
        match c {
            '{' => i += 1,
            '}' => i += 1,
            '\\' => {
                i += 1;
                if i >= chars.len() { break; }
                match chars[i] {
                    '\\' | '{' | '}' => {
                        transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                        para.push(chars[i]);
                        i += 1;
                    }
                    '~' => {
                        para.push(' ');
                        i += 1;
                    }
                    'a'..='z' | 'A'..='Z' => {
                        let start = i;
                        while i < chars.len() && chars[i].is_ascii_alphabetic() {
                            i += 1;
                        }
                        let word: String = chars[start..i].iter().collect();
                        let mut param = String::new();
                        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '-') {
                            param.push(chars[i]);
                            i += 1;
                        }
                        if i < chars.len() && chars[i] == ' ' {
                            i += 1;
                        }
                        match word.as_str() {
                            "par" | "sect" => flush(&mut para, &mut run, &mut run_bold, &mut run_italic, &mut out, &mut para_prefix),
                            "line" | "lbr" => {
                                transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                                para.push_str("  \n");
                            }
                            "tab" => para.push('\t'),
                            // \pard resets paragraph formatting; clear any active style prefix.
                            "pard" => para_prefix.clear(),
                            // \sN — named style from the pre-parsed stylesheet.
                            // outlinelevel (below) overrides heading level when both are present.
                            "s" if !param.is_empty() => {
                                if let Ok(idx) = param.parse::<u32>() {
                                    if let Some(pfx) = style_map.get(&idx) {
                                        para_prefix = pfx.clone();
                                    }
                                }
                            }
                            // \outlinelevelN is the most reliable heading signal in Word RTF.
                            // Level 0 → H1, 1 → H2, … up to H6.
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
                                    bold += 1;
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
                                    italic += 1;
                                }
                            }
                            "u" => {
                                let code: i32 = param.trim_start_matches('-').parse().unwrap_or(0);
                                transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                                if let Some(ch) = char::from_u32(code as u32) {
                                    run.push(ch);
                                }
                                if i < chars.len() {
                                    i += 1;
                                }
                            }
                            w if skipped.contains(&w) => skip_depth += 1,
                            _ => {}
                        }
                    }
                    '\'' => {
                        if i + 2 < chars.len() {
                            let hex: String = chars[i + 1..i + 3].iter().collect();
                            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                                transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                                run.push(b as char);
                            }
                            i += 3;
                        } else {
                            i += 1;
                        }
                    }
                    '*' => skip_depth += 1,
                    _ => i += 1,
                }
            }
            '\r' | '\n' => i += 1,
            _ => {
                transition(&mut para, &mut run, &mut run_bold, &mut run_italic, bold > 0, italic > 0);
                run.push(c);
                i += 1;
            }
        }
    }
    flush(&mut para, &mut run, &mut run_bold, &mut run_italic, &mut out, &mut para_prefix);
    Ok(out)
}

/// Converts a sheet name into a safe CSV file base name.
fn sanitize_sheet_name(name: &str) -> String {
    let clean: String = name
        .trim()
        .chars()
        .map(|c| if matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_whitespace() { '_' } else { c })
        .collect();
    // Collapse runs of underscores produced by replaced characters.
    let mut collapsed = String::new();
    for c in clean.chars() {
        if c == '_' && collapsed.ends_with('_') {
            continue;
        }
        collapsed.push(c);
    }
    let clean = collapsed.trim_matches('_').to_string();
    if clean.is_empty() { "sheet".into() } else { clean }
}

/// Reads the chart type from a BRS chart sheet header (`type` / `jenis` label in
/// the first column, value in the second), e.g. "line" -> `C1_line.csv`.
fn sheet_chart_type<'a, I: Iterator<Item = &'a [calamine::Data]>>(rows: I) -> Option<String> {
    for row in rows.take(5) {
        let r: Vec<String> = row.iter().map(cell_to_string).collect();
        if r.len() > 1 {
            let k = r[0].trim().to_lowercase();
            if (k == "type" || k == "jenis") && !r[1].trim().is_empty() {
                let v: String = r[1].trim().to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
                if !v.is_empty() {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn parse_chart_type_str(s: Option<&str>) -> Option<crate::vector::ChartType> {
    let t = s?.trim().to_lowercase();
    if t.contains("line") {
        Some(crate::vector::ChartType::Line)
    } else if t.contains("horizontal") {
        Some(crate::vector::ChartType::HorizontalBar)
    } else if t.contains("donut") || t.contains("pie") {
        Some(crate::vector::ChartType::Donut)
    } else if t.contains("map") || t.contains("choropleth") {
        Some(crate::vector::ChartType::Map)
    } else {
        Some(crate::vector::ChartType::BarClustered)
    }
}

/// CSV base name for a sheet: sanitized sheet name; BRS chart sheets (CXX)
/// append the chart type, e.g. `C1_line`. Shared by the converter and the
/// content-entry extractor so card sources bind the exact converted file.
fn sheet_csv_base(sheet_name: &str, is_brs: bool, type_val: Option<&str>) -> String {
    let mut base = sanitize_sheet_name(sheet_name);
    if is_brs && base.to_lowercase().starts_with('c')
        && let Some(t) = type_val.filter(|t| !t.is_empty())
    {
        base = format!("{base}_{t}");
    }
    base
}

fn xlsx_to_csv(path: &Path, profile_id: &str) -> Result<Vec<OutFile>> {
    let mut wb = calamine::open_workbook_auto(path)
        .map_err(|e| anyhow!("open xlsx {}: {e}", path.display()))?;
    let names = wb.sheet_names().to_vec();
    let is_brs = profile_id.trim().to_lowercase().contains("brs");
    let mut files = Vec::new();
    let mut used: Vec<String> = Vec::new();
    for sheet_name in &names {
        let range = wb
            .worksheet_range(sheet_name)
            .map_err(|e| anyhow!("read sheet {sheet_name}: {e}"))?;
        let lower = sheet_name.trim().to_lowercase();
        let base = sheet_csv_base(sheet_name, is_brs, sheet_chart_type(range.rows()).as_deref());

        // BRS point/summary sheets: single-column CSV with an `item` header;
        // each non-empty row becomes one numbered bullet paragraph downstream.
        if is_brs && matches!(lower.as_str(), "point" | "ringkasan") {
            let mut w = csv::WriterBuilder::new().from_writer(Vec::new());
            let _ = w.write_record(["item"]);
            for row in range.rows() {
                let r: Vec<String> = row.iter().map(cell_to_string).collect();
                let joined = r.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
                if !joined.is_empty() {
                    let _ = w.write_record([&joined]);
                }
            }
            files.push(OutFile { name: format!("csv/{base}.csv"), bytes: w.into_inner().unwrap_or_default() });
            used.push(base);
            continue;
        }

        let bytes = rows_to_csv(range.rows());

        // Deduplicate colliding sheet names.
        // ponytail: the extractor is not dedupe-aware; revisit if sheets collide.
        let mut final_name = base.clone();
        let mut n = 2;
        while used.contains(&final_name) {
            final_name = format!("{base}-{n}");
            n += 1;
        }
        used.push(final_name.clone());
        files.push(OutFile { name: format!("csv/{final_name}.csv"), bytes });
    }
    Ok(files)
}

/// Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Formats an Excel date serial as a human-readable date (or date+time).
/// ponytail: ignores the rare 1904-epoch workbooks; add is_1904 plumbing if needed.
fn excel_serial_to_string(serial: f64) -> String {
    let mut v = serial;
    if v < 60.0 {
        v += 1.0; // Excel 1900 leap-year bug
    }
    let days = v.floor() as i64 - 25_569; // to days since 1970-01-01
    let frac = v - v.floor();
    let (y, m, d) = civil_from_days(days);
    if frac < 1e-6 {
        format!("{y:04}-{m:02}-{d:02}")
    } else {
        let secs = (frac * 86_400.0).round() as i64;
        format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}", secs / 3600, (secs / 60) % 60, secs % 60)
    }
}

pub(crate) fn cell_to_string(cell: &calamine::Data) -> String {
    use calamine::Data;
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 9e15 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => format!("{i}"),
        Data::Bool(b) => if *b { "TRUE".to_owned() } else { "FALSE".to_owned() },
        Data::DateTime(dt) => excel_serial_to_string(dt.as_f64()),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{e}"),
    }
}

fn rows_to_csv<'a, I: Iterator<Item = &'a [calamine::Data]>>(rows: I) -> Vec<u8> {
    let mut w = csv::WriterBuilder::new().from_writer(Vec::new());
    for row in rows {
        let record: Vec<String> = row.iter().map(cell_to_string).collect();
        let _ = w.write_record(&record);
    }
    w.into_inner().unwrap_or_default()
}

fn parse_date_components(s: &str) -> Option<(i32, u32, u32)> {
    let s = s.trim();
    if let Some((y_str, rest)) = s.split_once('-').or_else(|| s.split_once('/'))
        && let Some((m_str, d_str)) = rest.split_once('-').or_else(|| rest.split_once('/'))
    {
        if let (Ok(y), Ok(m), Ok(d)) = (y_str.parse::<i32>(), m_str.parse::<u32>(), d_str.parse::<u32>()) {
            return Some((y, m, d));
        }
    }
    if let Ok(f) = s.parse::<f64>() {
        if (20000.0..100000.0).contains(&f) {
            let s_iso = excel_serial_to_string(f);
            return parse_date_components(&s_iso);
        }
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 3 {
        if let (Ok(d), Ok(y)) = (parts[0].parse::<u32>(), parts[2].parse::<i32>()) {
            let m_low = parts[1].to_lowercase();
            let month_pairs = [
                ("jan", 1), ("feb", 2), ("mar", 3), ("apr", 4),
                ("mei", 5), ("may", 5), ("jun", 6), ("jul", 7),
                ("agu", 8), ("aug", 8), ("sep", 9), ("okt", 10),
                ("oct", 10), ("nov", 11), ("des", 12), ("dec", 12),
            ];
            for (prefix, m) in month_pairs {
                if m_low.starts_with(prefix) {
                    return Some((y, m, d));
                }
            }
        }
    }
    None
}

pub fn format_indonesian_date(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Some((y, m, d)) = parse_date_components(s) {
        let months = [
            "Januari", "Februari", "Maret", "April", "Mei", "Juni",
            "Juli", "Agustus", "September", "Oktober", "November", "Desember",
        ];
        if (1..=12).contains(&m) {
            return format!("{d} {} {y}", months[(m - 1) as usize]);
        }
    }
    s.to_string()
}

pub fn format_indonesian_month_year(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Some((y, m, _)) = parse_date_components(s) {
        let months = [
            "Januari", "Februari", "Maret", "April", "Mei", "Juni",
            "Juli", "Agustus", "September", "Oktober", "November", "Desember",
        ];
        if (1..=12).contains(&m) {
            return format!("{} {y}", months[(m - 1) as usize]);
        }
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 2 {
        if let Ok(y) = parts[1].parse::<i32>() {
            let m_low = parts[0].to_lowercase();
            let month_pairs = [
                ("jan", "Januari"), ("feb", "Februari"), ("mar", "Maret"), ("apr", "April"),
                ("mei", "Mei"), ("may", "Mei"), ("jun", "Juni"), ("jul", "Juli"),
                ("agu", "Agustus"), ("aug", "Agustus"), ("sep", "September"), ("okt", "Oktober"),
                ("oct", "Oktober"), ("nov", "November"), ("des", "Desember"), ("dec", "Desember"),
            ];
            for (prefix, name) in month_pairs {
                if m_low.starts_with(prefix) {
                    return format!("{name} {y}");
                }
            }
        }
    }
    s.to_string()
}

/// Returns true if the document content begins with a paragraph formatted
/// with style "Title" (or "Judul", "Titel", "Document Title").
pub fn content_starts_with_title_style(name: &str, bytes: &[u8]) -> bool {
    let lower_name = name.to_lowercase();
    if lower_name.ends_with(".rtf") || bytes.starts_with(b"{\\rtf") {
        rtf_first_paragraph_is_title(bytes)
    } else if lower_name.ends_with(".docx") || bytes.starts_with(b"PK") {
        docx_first_paragraph_is_title(bytes)
    } else if lower_name.ends_with(".md") {
        if let Ok(s) = std::str::from_utf8(bytes) {
            for line in s.lines() {
                let t = line.trim();
                if !t.is_empty() {
                    return t.starts_with("# ") && !t.starts_with("##");
                }
            }
        }
        false
    } else {
        false
    }
}

fn rtf_first_paragraph_is_title(bytes: &[u8]) -> bool {
    let s = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    };
    let Some(ss_off) = s.find("\\stylesheet") else { return false };
    let ss_start = s[..ss_off].rfind('{').unwrap_or(ss_off);

    let mut depth = 0i32;
    let mut ss_end = ss_start;
    for (i, c) in s[ss_start..].char_indices() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth <= 0 {
                ss_end = ss_start + i;
                break;
            }
        }
    }
    let stylesheet_text = &s[ss_start..ss_end];

    let mut title_style_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut i = 0;
    while let Some(pos) = stylesheet_text[i..].find("{\\s") {
        let entry_start = i + pos;
        if let Some(entry_end) = stylesheet_text[entry_start..].find('}') {
            let entry = &stylesheet_text[entry_start..entry_start + entry_end];
            let mut style_id: Option<u32> = None;
            if let Some(s_pos) = entry.find("\\s") {
                let rest = &entry[s_pos + 2..];
                let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                style_id = digits.parse().ok();
            }
            if let Some(semi) = entry.rfind(';') {
                let before_semi = &entry[..semi];
                let name = before_semi.rsplit('\\').next().unwrap_or(before_semi);
                let name_direct = name.trim().to_lowercase();
                if (name_direct == "title" || name_direct.contains("title") || name_direct.contains("judul") || name_direct.contains("titel"))
                    && !name_direct.contains("sub")
                {
                    if let Some(sid) = style_id {
                        title_style_ids.insert(sid);
                    }
                }
            }
            i = entry_start + entry_end + 1;
        } else {
            break;
        }
    }

    let body = &s[ss_end..];
    for par in body.split("\\pard").skip(1) {
        let mut clean = String::new();
        let mut in_ctrl = false;
        let mut chars = par.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                in_ctrl = true;
            } else if in_ctrl {
                if c.is_ascii_whitespace() {
                    in_ctrl = false;
                } else if !c.is_ascii_alphanumeric() && c != '-' && c != '*' {
                    in_ctrl = false;
                }
            } else if c != '{' && c != '}' && !c.is_ascii_whitespace() {
                clean.push(c);
            }
        }

        if clean.len() >= 2 {
            let mut sid = None;
            if let Some(s_pos) = par.find("\\s") {
                let rest = &par[s_pos + 2..];
                let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                sid = digits.parse().ok();
            }
            return sid.map(|id| title_style_ids.contains(&id)).unwrap_or(false);
        }
    }

    false
}

fn docx_first_paragraph_is_title(bytes: &[u8]) -> bool {
    let cursor = std::io::Cursor::new(bytes);
    let Ok(mut archive) = zip::ZipArchive::new(cursor) else { return false };
    let Ok(file) = archive.by_name("word/document.xml") else { return false };
    let reader = std::io::BufReader::new(file);
    let mut xml = Reader::from_reader(reader);
    xml.config_mut().trim_text(true);

    let mut in_p = false;
    let mut current_style = String::new();
    let mut p_text = String::new();
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match xml.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"w:p" => {
                    in_p = true;
                    current_style.clear();
                    p_text.clear();
                }
                b"w:pStyle" => {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"w:val" {
                            current_style = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(e)) if e.name().as_ref() == b"w:pStyle" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"w:val" {
                        current_style = String::from_utf8_lossy(&attr.value).to_string();
                    }
                }
            }
            Ok(Event::Text(t)) if in_p => {
                if let Ok(txt) = t.unescape() {
                    p_text.push_str(&txt);
                }
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"w:p" => {
                in_p = false;
                if p_text.trim().len() >= 2 {
                    let s = current_style.to_lowercase();
                    return s.contains("title") || s.contains("judul") || s.contains("titel");
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    false
}

/// Parses an Excel file to extract metadata from a `COVER` sheet if present.
pub fn parse_excel_cover(path: &Path) -> Result<Option<crate::metadata_form::ExcelCoverData>> {
    let mut wb = calamine::open_workbook_auto(path)
        .map_err(|e| anyhow!("open xlsx {}: {e}", path.display()))?;
    let names = wb.sheet_names().to_vec();
    for name in &names {
        let lower = name.trim().to_lowercase();
        if lower == "cover" || lower == "cov" || lower == "sampul" {
            let range = wb
                .worksheet_range(name)
                .map_err(|e| anyhow!("read sheet {name}: {e}"))?;
            let mut grid: Vec<Vec<String>> = Vec::new();
            for row in range.rows() {
                let r: Vec<String> = row.iter().map(cell_to_string).collect();
                grid.push(r);
            }
            return Ok(Some(parse_cover_grid(&grid)));
        }
    }
    Ok(None)
}

/// Parses a 2D matrix of strings (from a COVER sheet) into ExcelCoverData.
pub fn parse_cover_grid(grid: &[Vec<String>]) -> crate::metadata_form::ExcelCoverData {
    let mut cover = crate::metadata_form::ExcelCoverData::default();
    for (_row_idx, row) in grid.iter().enumerate() {
        let Some(key) = row.first().map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()) else {
            if _row_idx == 2 && row.len() > 1 && !row[1].trim().is_empty() && cover.title_en.is_empty() {
                let v = row[1..].iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
                cover.title_en = v.clone();
                if cover.title2.is_empty() {
                    cover.title2 = v;
                }
            }
            continue;
        };
        let value = row[1..].iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
        if value.is_empty() {
            continue;
        }
        let norm: String = key.chars().filter(|c| !matches!(c, '_' | ' ' | '-')).collect();
        match norm.as_str() {
            "style" => cover.style = value,
            "level" => cover.level = value,
            "titleid" | "title1" | "title" => cover.title1 = value,
            "title2" => {
                cover.title2 = value.clone();
                if cover.title_en.is_empty() {
                    cover.title_en = value;
                }
            }
            "titleen" => cover.title_en = value,
            "year" | "tahun" => cover.year = value,
            "volume" | "vol" => cover.volume = value,
            "nocatalogue" | "catalogue" | "catalog" | "katalog" => cover.catalogue = value,
            "issn" | "isbn" | "code" => cover.code = value,
            "officename" | "agency" | "lembaga" | "instansi" => cover.agency = value,
            "puboffice" | "kantorpublikasi" => cover.pub_office = value,
            "image1" | "imagehero" | "image" | "imagetheme1" | "theme1" | "hero" => cover.image_hero = value,
            "imagelandscape" | "landscape" | "imagetheme2" | "theme2" => cover.image_landscape = value,
            "officelogo" | "logo" => cover.logo = value,
            "officeaddress" | "address" | "alamatkantor" | "alamat" => cover.office_address = value,
            "tabletheme" | "theme" | "color" => cover.table_theme = value,
            "palette" => cover.palette = value,
            "no" | "nomor" | "releaseno" | "no_release" | "norelease" | "volumeno" => cover.release_no = value,
            "date" | "tanggal" | "tgl" | "volumedate" | "releasedate" => cover.release_date = format_indonesian_date(&value),
            "month" | "bulan" | "releasemonth" => cover.release_month = format_indonesian_month_year(&value),
            "point" | "points" | "ringkasan" | "poin" | "highlight" => cover.point = value,
            "qr" | "qrcode" | "qr_code" => cover.qr = value,
            "picname" | "namapic" | "nama_pic" | "pic" | "narasumber" | "nama" => cover.pic_name = value,
            "picphoto" | "fotopic" | "foto_pic" | "photo" | "foto" => cover.pic_photo = value,
            "picposition" | "position" | "posisi" | "jabatan" => cover.position = value,
            "picoffice" | "office" | "kantor" => cover.office = value,
            "picphone" | "phone" | "telp" | "telepon" | "telpon" => cover.phone = value,
            "picemail" | "email" | "surel" => cover.email = value,
            _ => {}
        }
    }
    cover
}

/// Detects suitable publication profile from an Excel workbook.
pub fn detect_workbook_profile(path: &Path) -> Result<&'static str> {
    let wb = calamine::open_workbook_auto(path)
        .map_err(|e| anyhow!("open xlsx {}: {e}", path.display()))?;
    let names = wb.sheet_names().to_vec();
    Ok(crate::profiles::detect_profile_from_sheets(&names))
}

/// Extracts publication content items (pages / flowing objects) from workbook sheets.
///
/// Follows BRS vs DDA publication rules:
/// - In BRS:
///   1. `COV` -> NewPage (Cover)
///   2. `POINT` -> NewPage (Executive Summary / Bullets)
///   3. `U1` -> NewPage (Main narrative chapter)
///   4. `UX` (except U1), `CX`, `TX`, `GX` -> NewObject (Flowing objects sequentially inserted)
/// - In DDA:
///   Every sheet is generated as a `NewPage`.
pub fn extract_workbook_content_entries(
    path: &Path,
    profile_id: &str,
) -> Result<Vec<crate::metadata_form::PageEntry>> {
    let mut wb = calamine::open_workbook_auto(path)
        .map_err(|e| anyhow!("open xlsx {}: {e}", path.display()))?;
    let sheet_names = wb.sheet_names().to_vec();
    let is_brs = profile_id.trim().to_lowercase().contains("brs");
    let parsed_cover_opt = parse_excel_cover(path).ok().flatten();
    let mut entries = Vec::new();

    for sheet_name in sheet_names {
        let trimmed_name = sheet_name.trim();
        let lower = trimmed_name.to_lowercase();

        let range_opt = wb.worksheet_range(&sheet_name).ok();
        let mut first_cell_text = String::new();
        let mut title_val = String::new();
        let mut title_en_val = String::new();
        let mut file_val = String::new();
        let mut chart_type_val: Option<String> = None;

        if let Some(ref range) = range_opt {
            chart_type_val = sheet_chart_type(range.rows());
            for row in range.rows().take(5) {
                let r: Vec<String> = row.iter().map(cell_to_string).collect();
                if r.is_empty() {
                    continue;
                }
                if first_cell_text.is_empty() && !r[0].trim().is_empty() {
                    first_cell_text = r[0].trim().to_string();
                }
                let k = r[0].trim().to_lowercase();
                if (k == "title" || k == "judul") && r.len() > 1 && !r[1].trim().is_empty() {
                    title_val = r[1].trim().to_string();
                } else if (k == "title_en" || k == "titleen" || k == "judulen" || k == "judul_en") && r.len() > 1 && !r[1].trim().is_empty() {
                    title_en_val = r[1].trim().to_string();
                } else if (k == "file" || k == "berkas") && r.len() > 1 && !r[1].trim().is_empty() {
                    file_val = r[1].trim().to_string();
                }
            }
        }

        let chapter_id_val = if let Some((prefix, _)) = trimmed_name.split_once('_')
            && prefix.chars().all(|c| c.is_ascii_digit())
        {
            prefix.to_string()
        } else if let Some(digits) = trimmed_name.strip_prefix(|c: char| c.is_ascii_alphabetic())
            && !digits.is_empty()
            && digits.chars().all(|c| c.is_ascii_digit())
        {
            let trimmed_digits = digits.trim_start_matches('0');
            if trimmed_digits.is_empty() { "0".to_string() } else { trimmed_digits.to_string() }
        } else {
            String::new()
        };

        // Compute the CSV file name this sheet will be converted to, so that
        // cards whose source comes from a CSV sheet (Charts, Tables, POINT, etc.)
        // are automatically bound to the correct `csv/…` file.
        let csv_source = {
            let base = sheet_csv_base(trimmed_name, is_brs, chart_type_val.as_deref());
            format!("csv/{base}.csv")
        };

        if is_brs {
            if lower == "cov" || lower == "cover" || lower == "sampul" {
                entries.push(crate::metadata_form::PageEntry {
                    kind: crate::metadata_form::PageKind::Cover,
                    scope: crate::metadata_form::PageItemScope::NewPage,
                    title: if !title_val.is_empty() {
                        format!("Sampul Rilis: {title_val}")
                    } else {
                        "Sampul Berita Resmi Statistik (COV)".to_string()
                    },
                    source: None,
                    cover: parsed_cover_opt.clone(),
                    ..Default::default()
                });
            } else if lower == "point" || lower == "ringkasan" {
                let display_title = if !first_cell_text.is_empty() {
                    let mut s: String = first_cell_text.chars().take(60).collect();
                    if first_cell_text.chars().count() > 60 {
                        s.push_str("...");
                    }
                    format!("Ringkasan Poin: {s}")
                } else {
                    "Ringkasan Eksekutif & Poin Utama (POINT)".to_string()
                };
                entries.push(crate::metadata_form::PageEntry {
                    kind: crate::metadata_form::PageKind::Paragraph,
                    scope: crate::metadata_form::PageItemScope::NewPage,
                    title: display_title,
                    source: Some(csv_source.clone()),
                    ..Default::default()
                });
            } else if lower == "u1" {
                let parent_dir = path.parent().unwrap_or(Path::new(""));
                let rtf_or_docx = if !file_val.is_empty() { file_val.clone() } else { "U1.rtf".to_string() };
                let candidate = parent_dir.join(&rtf_or_docx);
                let has_title = if candidate.exists() {
                    std::fs::read(&candidate).map(|b| content_starts_with_title_style(&rtf_or_docx, &b)).unwrap_or(true)
                } else {
                    true
                };
                let stem = Path::new(&rtf_or_docx).file_stem().and_then(|s| s.to_str()).unwrap_or("U1");
                let md_source = format!("md/{stem}.md");
                entries.push(crate::metadata_form::PageEntry {
                    kind: crate::metadata_form::PageKind::Paragraph,
                    scope: crate::metadata_form::PageItemScope::NewPage,
                    title: "Ulasan Indikator Utama (U1)".to_string(),
                    has_title,
                    source: Some(md_source),
                    ..Default::default()
                });
            } else if lower.starts_with('u') {
                let stem = if !file_val.is_empty() {
                    Path::new(&file_val).file_stem().and_then(|s| s.to_str()).unwrap_or(trimmed_name)
                } else {
                    trimmed_name
                };
                let title = format!("Ulasan Narasi {trimmed_name} ({stem}.md)");
                let md_source = format!("md/{stem}.md");
                let parent_dir = path.parent().unwrap_or(Path::new(""));
                let rtf_or_docx = if !file_val.is_empty() { file_val.clone() } else { format!("{trimmed_name}.rtf") };
                let candidate = parent_dir.join(&rtf_or_docx);
                let has_title = if candidate.exists() {
                    std::fs::read(&candidate).map(|b| content_starts_with_title_style(&rtf_or_docx, &b)).unwrap_or(false)
                } else {
                    false
                };
                entries.push(crate::metadata_form::PageEntry {
                    kind: crate::metadata_form::PageKind::Paragraph,
                    scope: crate::metadata_form::PageItemScope::NewObject,
                    title,
                    has_title,
                    source: Some(md_source),
                    ..Default::default()
                });
            } else if lower.starts_with('c') || lower.starts_with("chart") || lower.starts_with("grafik") {
                let title = if !title_val.is_empty() {
                    title_val.replace('\n', " ")
                } else if !first_cell_text.is_empty() && first_cell_text.to_lowercase().starts_with("gambar") {
                    first_cell_text.replace('\n', " ")
                } else {
                    format!("Grafik {trimmed_name}")
                };
                entries.push(crate::metadata_form::PageEntry {
                    kind: crate::metadata_form::PageKind::Chart,
                    scope: crate::metadata_form::PageItemScope::NewObject,
                    title,
                    title_en: title_en_val.clone(),
                    chapter_id: chapter_id_val.clone(),
                    chart_type: parse_chart_type_str(chart_type_val.as_deref()),
                    source: Some(csv_source.clone()),
                    ..Default::default()
                });
            } else if lower.starts_with('t') || lower.starts_with("table") || lower.starts_with("tabel") {
                let title = if !first_cell_text.is_empty() && first_cell_text.to_lowercase().starts_with("tabel") {
                    first_cell_text.replace('\n', " ")
                } else if !title_val.is_empty() {
                    title_val.replace('\n', " ")
                } else {
                    format!("Tabel {trimmed_name}")
                };
                entries.push(crate::metadata_form::PageEntry {
                    kind: crate::metadata_form::PageKind::Table,
                    scope: crate::metadata_form::PageItemScope::NewObject,
                    title,
                    source: Some(csv_source.clone()),
                    ..Default::default()
                });
            } else if lower.starts_with('g') || lower.starts_with("gambar") || lower.starts_with("image") {
                let title = if !file_val.is_empty() {
                    format!("Infografis / Gambar {trimmed_name} ({file_val})")
                } else {
                    format!("Gambar {trimmed_name}")
                };
                entries.push(crate::metadata_form::PageEntry {
                    kind: crate::metadata_form::PageKind::Image,
                    scope: crate::metadata_form::PageItemScope::NewObject,
                    title,
                    image_area: Some(crate::metadata_form::ImageArea::MarginArea),
                    source: if !file_val.is_empty() { Some(file_val) } else { None },
                    ..Default::default()
                });
            } else {
                entries.push(crate::metadata_form::PageEntry {
                    kind: crate::metadata_form::PageKind::Paragraph,
                    scope: crate::metadata_form::PageItemScope::NewObject,
                    title: format!("Konten {trimmed_name}"),
                    source: Some(csv_source),
                    ..Default::default()
                });
            }
        } else {
            let (kind, default_title) = if lower == "cover" || lower == "cov" || lower == "sampul" {
                (crate::metadata_form::PageKind::Cover, "Halaman Sampul (Cover)".to_string())
            } else if lower.starts_with("div") || lower.starts_with("pembatas") {
                (crate::metadata_form::PageKind::Paragraph, format!("Pembatas Bab ({trimmed_name})"))
            } else if lower.starts_with('c') || lower.starts_with("chart") || lower.starts_with("grafik") {
                (crate::metadata_form::PageKind::Chart, format!("Grafik ({trimmed_name})"))
            } else if lower.starts_with('g') || lower.starts_with("gambar") || lower.starts_with("image") {
                (crate::metadata_form::PageKind::Image, format!("Gambar / Foto ({trimmed_name})"))
            } else if lower.contains('_') || lower.starts_with('t') || lower.starts_with("tabel") || lower.starts_with("table") {
                (crate::metadata_form::PageKind::Table, format!("Tabel {trimmed_name}"))
            } else {
                (crate::metadata_form::PageKind::Paragraph, format!("Ulasan / Bab ({trimmed_name})"))
            };

            let title = if !title_val.is_empty() {
                title_val.replace('\n', " ")
            } else if !first_cell_text.is_empty() && (first_cell_text.to_lowercase().starts_with("tabel") || first_cell_text.to_lowercase().starts_with("bab")) {
                first_cell_text.replace('\n', " ")
            } else {
                default_title
            };

            let effective_source = if !file_val.is_empty() {
                let low = file_val.to_lowercase();
                if low.ends_with(".rtf") || low.ends_with(".docx") {
                    let stem = Path::new(&file_val).file_stem().and_then(|s| s.to_str()).unwrap_or(&file_val);
                    Some(format!("md/{stem}.md"))
                } else {
                    Some(file_val)
                }
            } else {
                Some(csv_source)
            };

            entries.push(crate::metadata_form::PageEntry {
                kind,
                scope: crate::metadata_form::PageItemScope::NewPage,
                title,
                title_en: title_en_val.clone(),
                chapter_id: chapter_id_val.clone(),
                chart_type: if kind == crate::metadata_form::PageKind::Chart { parse_chart_type_str(chart_type_val.as_deref()) } else { None },
                image_area: if kind == crate::metadata_form::PageKind::Image { Some(crate::metadata_form::ImageArea::MarginArea) } else { None },
                cover: if kind == crate::metadata_form::PageKind::Cover { parsed_cover_opt.clone() } else { None },
                source: effective_source,
                ..Default::default()
            });
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_docx(document_xml: &str) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut w = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            w.start_file("word/document.xml", opts).unwrap();
            std::io::Write::write_all(&mut w, document_xml.as_bytes()).unwrap();
            w.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn docx_headings_lists_and_runs() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
            <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title Here</w:t></w:r></w:p>
            <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>bold</w:t></w:r><w:r><w:t> plain </w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>ital</w:t></w:r></w:p>
            <w:p><w:pPr><w:numPr/></w:pPr><w:r><w:t>item one</w:t></w:r></w:p>
            <w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Section</w:t></w:r></w:p>
            <w:p><w:pPr><w:pStyle w:val="Quote"/></w:pPr><w:r><w:t>quoted line</w:t></w:r></w:p>
            <w:p><w:pPr><w:pStyle w:val="ListParagraph"/></w:pPr><w:r><w:t>bullet row</w:t></w:r></w:p>
        </w:body></w:document>"#;
        let md = docx_to_md(&make_docx(xml)).unwrap();
        assert!(md.contains("# Title Here"), "got: {md}");
        assert!(md.contains("**bold** plain _ital_"), "got: {md}");
        assert!(md.contains("- item one"), "got: {md}");
        assert!(md.contains("## Section"), "got: {md}");
        assert!(md.contains("> quoted line"), "got: {md}");
        assert!(md.contains("- bullet row"), "got: {md}");
    }

    #[test]
    fn dual_lang_sample_narratives_convert_with_heading_layout() {
        // The DUAL_LANG workbooks reference RTF narratives by bare name; their
        // stylesheets carry Word heading styles used in the body. The converted
        // md must keep the paragraph layout (headings become md headings).
        let rtf = std::path::Path::new("samples/DUAL_LANG/rtf/R01.rtf");
        if rtf.exists() {
            let files = ingest(rtf).expect("R01.rtf must convert");
            assert_eq!(files.len(), 1);
            assert_eq!(files[0].name, "md/R01.md");
            let md = String::from_utf8_lossy(&files[0].bytes).into_owned();
            assert!(!md.trim().is_empty(), "converted md must not be empty");
            assert!(
                md.lines().any(|l| l.starts_with("## ")),
                "R01 body uses \\s2 (heading 2); md must carry the heading layout:\n{md}"
            );
        }
    }

    #[test]
    fn kcda_sample_docx_converts_to_nonempty_md() {
        let docx = std::path::Path::new("samples/DUAL_LANG/KCDA_Kartim 2025.docx");
        if docx.exists() {
            let files = ingest(docx).expect("KCDA docx must convert");
            assert_eq!(files[0].name, "md/KCDA_Kartim 2025.md");
            let md = String::from_utf8_lossy(&files[0].bytes).into_owned();
            assert!(!md.trim().is_empty(), "converted md must not be empty");
        }
    }

    #[test]
    fn rtf_groups_and_formatting() {
        let rtf = br#"{\rtf1{\fonttbl{\f0 Arial}}\b Bold \b0 plain \i ital \i0 end\par Second line\'e9\u8212? tail}"#;
        let md = rtf_to_md(rtf).unwrap();
        assert!(md.contains("**Bold** plain _ital_ end"), "got: {md}");
        assert!(md.contains("Second line"), "got: {md}");
        assert!(md.contains('\u{e9}'), "expected é, got: {md}");
        assert!(md.contains('\u{2014}'), "expected em-dash, got: {md}");
        assert!(!md.contains("Arial"), "got: {md}");
    }

    #[test]
    fn rtf_paragraph_styles_headings_and_quote() {
        // Stylesheet with Heading 1, Heading 2, Title, Subtitle, and Block Text (quote) styles.
        let rtf = br#"{\rtf1
{\stylesheet
{\s0 Normal;}
{\s1\outlinelevel0 Heading 1;}
{\s2\outlinelevel1 Heading 2;}
{\s15 Title;}
{\s16 Subtitle;}
{\s17 Block Text;}
}
\pard\s15 My Title\par
\pard\s1\outlinelevel0 Chapter One\par
\pard\s2\outlinelevel1 Section Two\par
\pard\s17 A quoted passage.\par
\pard\s0 Normal paragraph.\par
}"#;
        let md = rtf_to_md(rtf).unwrap();
        assert!(md.contains("# My Title"), "Title should become H1, got: {md}");
        assert!(md.contains("# Chapter One"), "Heading 1 should become H1, got: {md}");
        assert!(md.contains("## Section Two"), "Heading 2 should become H2, got: {md}");
        assert!(md.contains("> A quoted passage."), "Block Text should become blockquote, got: {md}");
        assert!(md.contains("Normal paragraph."), "Normal text should pass through, got: {md}");
        assert!(!md.contains("# Normal"), "Normal must not get a heading prefix, got: {md}");
    }

    #[test]
    fn rtf_outlinelevel_without_explicit_style() {
        // \outlinelevel alone (no stylesheet) should still produce headings.
        let rtf = br#"{\rtf1\pard\outlinelevel0 Heading via outlinelevel\par\pard Normal\par}"#;
        let md = rtf_to_md(rtf).unwrap();
        assert!(md.contains("# Heading via outlinelevel"), "outlinelevel0 must become H1, got: {md}");
        assert!(md.contains("Normal"), "got: {md}");
        assert!(!md.contains("# Normal"), "got: {md}");
    }

    #[test]
    fn rtf_parse_stylesheet_maps_known_styles() {
        let rtf = r#"{\rtf1{\stylesheet{\s0 Normal;}{\s1\outlinelevel0 Heading 1;}{\s3 Title;}{\s9 Block Text;}}}"#;
        let map = rtf_parse_stylesheet(rtf);
        assert_eq!(map.get(&1).map(|s| s.as_str()), Some("# "), "Heading 1 → H1");
        assert_eq!(map.get(&3).map(|s| s.as_str()), Some("# "), "Title → H1");
        assert_eq!(map.get(&9).map(|s| s.as_str()), Some("> "), "Block Text → blockquote");
        assert!(map.get(&0).is_none(), "Normal must not be in map");
    }
    #[test]
    fn csv_cells_typed() {
        let rows = vec![
            vec![calamine::Data::String("name".into()), calamine::Data::String("qty".into())],
            vec![calamine::Data::String("widget".into()), calamine::Data::Float(4.0)],
        ];
        let bytes = rows_to_csv(rows.iter().map(|r| r.as_slice()));
        let text = String::from_utf8(bytes).unwrap();
        assert_eq!(text, "name,qty\nwidget,4\n");
    }

    #[test]
    fn excel_serial_dates_human_readable() {
        let dt = |serial| {
            cell_to_string(&calamine::Data::DateTime(calamine::ExcelDateTime::new(
                serial,
                calamine::ExcelDateTimeType::DateTime,
                false,
            )))
        };
        assert_eq!(dt(46296.0), "2026-10-01");
        assert_eq!(dt(46266.0), "2026-09-01");
        assert_eq!(dt(25569.0), "1970-01-01");
        assert_eq!(dt(46296.5416666667), "2026-10-01 13:00:00");
    }

    #[test]
    fn csv_sheet_name_and_chart_type() {
        assert_eq!(sanitize_sheet_name(" C1 Grafik "), "C1_Grafik");
        assert_eq!(sanitize_sheet_name("T1: Luas?"), "T1_Luas");

        let rows = vec![
            vec![calamine::Data::String("Judul".into()), calamine::Data::String("x".into())],
            vec![calamine::Data::String("type".into()), calamine::Data::String("Line ".into())],
        ];
        let slices: Vec<&[calamine::Data]> = rows.iter().map(|r| r.as_slice()).collect();
        assert_eq!(sheet_chart_type(slices.into_iter()), Some("line".into()));

        let none_rows = vec![vec![calamine::Data::String("data".into()), calamine::Data::Float(1.0)]];
        let none_slices: Vec<&[calamine::Data]> = none_rows.iter().map(|r| r.as_slice()).collect();
        assert_eq!(sheet_chart_type(none_slices.into_iter()), None);
    }

    #[test]
    fn test_brs_workbook_csv_conversion() {
        let sample = std::path::Path::new("../../plan/pubgen/samples/brs/ntp202610.xlsx");
        if !sample.exists() {
            return;
        }
        let files = ingest_with_profile(sample, "bps_brs").unwrap();
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();

        // POINT sheet converts to single-column CSV with `item` header.
        let point = files.iter().find(|f| f.name == "csv/POINT.csv").expect("csv/POINT.csv missing");
        let text = String::from_utf8(point.bytes.clone()).unwrap();
        assert!(text.starts_with("item\n"), "POINT header wrong: {text}");
        assert!(text.lines().count() > 2, "POINT rows missing");
        assert!(!text.contains(",,"), "POINT must be single column");

        // Chart sheet name carries its type.
        assert!(names.contains(&"csv/C1_line.csv"), "names: {names:?}");
    }

    #[test]
    fn test_extract_brs_content_entries_from_sample() {
        let sample_path = std::path::Path::new("../../plan/pubgen/samples/brs/ntp202610.xlsx");
        if !sample_path.exists() {
            return;
        }
        let entries = extract_workbook_content_entries(sample_path, "bps_brs").unwrap();
        assert!(!entries.is_empty(), "must extract entries");

        // 1. COV must be NewPage
        assert_eq!(entries[0].scope, crate::metadata_form::PageItemScope::NewPage);
        // 2. POINT must be NewPage
        assert_eq!(entries[1].scope, crate::metadata_form::PageItemScope::NewPage);
        // 3. U1 must be NewPage
        assert_eq!(entries[2].scope, crate::metadata_form::PageItemScope::NewPage);

        // 4. Other sheets (C1, U2, T1, U4, etc.) must be NewObject
        let rest = &entries[3..];
        for item in rest {
            assert_eq!(
                item.scope,
                crate::metadata_form::PageItemScope::NewObject,
                "sheet {} should be NewObject",
                item.title
            );
        }
    }

    #[test]
    fn test_brs_cover_populated_and_image_area_default() {
        let sample = std::path::Path::new("samples/brs/ntp202610.xlsx");
        let alt_sample = std::path::Path::new("../samples/brs/ntp202610.xlsx");
        let path = if sample.exists() {
            sample
        } else if alt_sample.exists() {
            alt_sample
        } else {
            return;
        };

        // Test 1: parse_excel_cover extracts all 14 fields with Indonesian date formatting
        let cover = parse_excel_cover(path).unwrap().expect("COVER sheet should yield data");
        assert_eq!(cover.title1, "Perkembangan Nilai Tukar Petani");
        assert_eq!(cover.release_date, "1 Oktober 2026");
        assert_eq!(cover.release_month, "September 2026");
        assert_eq!(cover.release_no, "61/10/32/Th. XXVIII");
        assert_eq!(cover.pub_office, "BADAN PUSAT STATISTIK PROVINSI JAWA BARAT");
        assert_eq!(cover.image_hero, "theme1.png");
        assert_eq!(cover.image_landscape, "theme2.png");
        assert_eq!(cover.pic_name, "Yana Hendriana, S.ST");
        assert_eq!(cover.pic_photo, "yana.jpg");
        assert_eq!(cover.position, "Ketua Tim Statistik Harga");
        assert_eq!(cover.office, "BPS Provinsi Jawa Barat");
        assert_eq!(cover.phone, "(022) 7272595");
        assert_eq!(cover.email, "yanah@bps.go.id");
        assert!(cover.point.contains("Nilai Tukar Petani"));

        // Test 2: extract_workbook_content_entries attaches cover and sets default image_area
        let entries = extract_workbook_content_entries(path, "bps_brs").unwrap();
        let cov_entry = entries.iter().find(|e| e.kind == crate::metadata_form::PageKind::Cover).expect("Cover card");
        assert!(cov_entry.cover.is_some(), "Cover entry must have populated cover struct");
        let cov_data = cov_entry.cover.as_ref().unwrap();
        assert_eq!(cov_data.release_date, "1 Oktober 2026");
        assert_eq!(cov_data.image_hero, "theme1.png");

        // Test 3: G1 image item must have ImageArea::MarginArea
        let img_entry = entries.iter().find(|e| e.kind == crate::metadata_form::PageKind::Image).expect("G1 image card");
        assert_eq!(img_entry.image_area, Some(crate::metadata_form::ImageArea::MarginArea));

        // Test 4: U1 has Title style and md source; U2 does not have title style and has md source
        let u1_entry = entries.iter().find(|e| e.title.contains("U1")).expect("U1 entry");
        assert!(u1_entry.has_title, "U1 must have has_title = true");
        assert_eq!(u1_entry.source.as_deref(), Some("md/U1.md"), "U1 source must be md/U1.md");

        let u2_entry = entries.iter().find(|e| e.title.contains("U2")).expect("U2 entry");
        assert!(!u2_entry.has_title, "U2 must have has_title = false");
        assert_eq!(u2_entry.source.as_deref(), Some("md/U2.md"), "U2 source must be md/U2.md");
        assert_eq!(u2_entry.title, "Ulasan Narasi U2 (U2.md)", "U2 title must refer to U2.md");
    }

    #[test]
    fn test_rtf_title_style_detection() {
        let u1_path = if std::path::Path::new("samples/brs/U1.rtf").exists() {
            std::path::PathBuf::from("samples/brs/U1.rtf")
        } else {
            std::path::PathBuf::from("../samples/brs/U1.rtf")
        };
        let u2_path = if std::path::Path::new("samples/brs/U2.rtf").exists() {
            std::path::PathBuf::from("samples/brs/U2.rtf")
        } else {
            std::path::PathBuf::from("../samples/brs/U2.rtf")
        };
        if u1_path.exists() && u2_path.exists() {
            let u1_bytes = std::fs::read(&u1_path).unwrap();
            let u2_bytes = std::fs::read(&u2_path).unwrap();
            assert!(content_starts_with_title_style("U1.rtf", &u1_bytes), "U1 starts with Title style");
            assert!(!content_starts_with_title_style("U2.rtf", &u2_bytes), "U2 starts with Heading 1, not Title");
        }
    }
}
