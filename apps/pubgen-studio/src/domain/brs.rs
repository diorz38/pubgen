//! BPS BRS (Berita Resmi Statistik) bulletin content and table parser.

#[derive(Debug, Clone)]
pub struct ParsedNarrativePara {
    pub text: String,
    pub style: String,
    pub is_title: bool,
    pub is_h1: bool,
    pub is_h2: bool,
    pub is_list: bool,
}

/// Parses raw markdown text from BRS narrative sheets into structured paragraphs.
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
            let starts_abc = content.starts_with("A. ")
                || content.starts_with("B. ")
                || content.starts_with("C. ")
                || content.starts_with("A.")
                || content.starts_with("B.")
                || content.starts_with("C.");

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
                || content.starts_with("- ")
                || content.starts_with("* ");

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

            paras.push(ParsedNarrativePara { text: content.to_string(), style, is_title, is_h1, is_h2, is_list });
        }
    }

    (full_text, paras)
}

/// Parses a CSV table into (title, 2D table grid, optional footnote/source).
pub fn parse_brs_csv_table(raw: &str) -> (String, Vec<Vec<String>>, Option<String>) {
    let mut lines = raw.lines().map(str::trim).filter(|l| !l.is_empty());
    let title = lines.next().unwrap_or("Tabel").to_string();

    let mut grid = Vec::new();
    let mut notes = Vec::new();

    for line in lines {
        if line.starts_with("Sumber:") || line.starts_with("Catatan:") || line.starts_with("Source:") || line.starts_with("Note:") {
            notes.push(line.to_string());
            continue;
        }

        // Split CSV cells quote-aware
        let cells: Vec<String> = line.split(',').map(|c| c.trim().trim_matches('"').to_string()).collect();

        if !cells.is_empty() {
            grid.push(cells);
        }
    }

    let footnote = if notes.is_empty() { None } else { Some(notes.join("\n")) };
    (title, grid, footnote)
}

/// Estimates narrative text height in points to assist automatic flowing box placement.
pub fn estimate_brs_narrative_height(text: &str) -> f64 {
    let line_count = text.lines().count().max(1);
    let char_count = text.chars().count();
    let estimated_lines = (char_count as f64 / 65.0).ceil().max(line_count as f64);
    // Average line height for 9.5pt font with 12.5pt leading: ~13pt per line
    estimated_lines * 13.0 + 16.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brs_narrative_parser() {
        let raw = r#"
# Nilai Tukar Petani (NTP)
NTP Provinsi Jawa Timur bulan Januari 2026 sebesar 115.40.

## NTP Tanaman Pangan
Subsektor tanaman pangan mengalami kenaikan indeks sebesar 1.25 persen.

- Kenaikan terjadi pada komoditas padi
- Jagung juga mengalami penguatan harga
"#;
        let (full_text, paras) = parse_brs_narrative(raw);
        assert!(!full_text.is_empty());
        assert_eq!(paras.len(), 6);
        assert!(paras[0].is_h1);
        assert_eq!(paras[0].style, "Heading 1");
        assert!(paras[2].is_h2);
        assert_eq!(paras[2].style, "Heading 2");
        assert!(paras[4].is_list);
        assert_eq!(paras[4].style, "List Paragraph");
        assert!(paras[5].is_list);
        assert_eq!(paras[5].style, "List Paragraph");
    }

    #[test]
    fn test_brs_table_parser() {
        let raw = r#"
Tabel 1. Perkembangan Indeks Harga
Subsektor,Jan 2025,Jan 2026,Perubahan
Tanaman Pangan,110.2,115.4,4.72
Hortikultura,105.1,108.3,3.04
Sumber: BPS Provinsi Jawa Timur
Catatan: Angka sementara
"#;
        let (title, grid, note) = parse_brs_csv_table(raw);
        assert_eq!(title, "Tabel 1. Perkembangan Indeks Harga");
        assert_eq!(grid.len(), 3); // Header + 2 data rows
        assert_eq!(grid[0], vec!["Subsektor", "Jan 2025", "Jan 2026", "Perubahan"]);
        assert!(note.is_some());
        let n = note.unwrap();
        assert!(n.contains("Sumber: BPS"));
        assert!(n.contains("Catatan: Angka sementara"));
    }
}
