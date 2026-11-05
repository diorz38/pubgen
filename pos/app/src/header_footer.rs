//! Header and footer line resolvers for BPS DDA and BPS BRS profiles.

/// Formats an Indonesian date string (e.g. "01-10-2026", "2026-10-01", "1 Oktober 2026").
pub fn format_indonesian_date(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    // If it already contains an Indonesian month name, return trimmed
    let indonesian_months = [
        "Januari", "Februari", "Maret", "April", "Mei", "Juni",
        "Juli", "Agustus", "September", "Oktober", "November", "Desember",
    ];
    for m in indonesian_months {
        if s.to_lowercase().contains(&m.to_lowercase()) {
            return s.to_string();
        }
    }
    // Attempt parsing "YYYY-MM-DD" or "DD-MM-YYYY"
    if let Some((d, m, y)) = parse_date_components(s)
        && (1..=12).contains(&m) {
        let month_name = indonesian_months[m - 1];
        return format!("{d} {month_name} {y}");
    }
    s.to_string()
}

/// Formats an Indonesian month-year string (e.g. "Oktober 2026").
pub fn format_indonesian_month_year(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    let indonesian_months = [
        "Januari", "Februari", "Maret", "April", "Mei", "Juni",
        "Juli", "Agustus", "September", "Oktober", "November", "Desember",
    ];
    for m in indonesian_months {
        if s.to_lowercase().contains(&m.to_lowercase()) {
            return s.to_string();
        }
    }
    if let Some((_, m, y)) = parse_date_components(s)
        && (1..=12).contains(&m) {
        return format!("{} {y}", indonesian_months[m - 1]);
    }
    s.to_string()
}

fn parse_date_components(s: &str) -> Option<(u32, usize, u32)> {
    let cleaned = s.replace('/', "-");
    let parts: Vec<&str> = cleaned.split('-').collect();
    if parts.len() == 3 {
        if parts[0].len() == 4 {
            // YYYY-MM-DD
            let y = parts[0].parse::<u32>().ok()?;
            let m = parts[1].parse::<usize>().ok()?;
            let d = parts[2].parse::<u32>().ok()?;
            return Some((d, m, y));
        } else if parts[2].len() == 4 {
            // DD-MM-YYYY
            let d = parts[0].parse::<u32>().ok()?;
            let m = parts[1].parse::<usize>().ok()?;
            let y = parts[2].parse::<u32>().ok()?;
            return Some((d, m, y));
        }
    }
    None
}

/// Extracts a 4-digit year (1900..=2100) from text.
pub fn extract_year_from_text(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    if bytes.len() < 4 {
        return None;
    }
    for i in 0..=bytes.len() - 4 {
        let chunk = &s[i..i + 4];
        if chunk.chars().all(|c| c.is_ascii_digit())
            && let Ok(y) = chunk.parse::<u32>()
            && (1900..=2100).contains(&y) {
            return Some(chunk.to_string());
        }
    }
    None
}

/// Resolves the two lines for BPS DDA footer:
/// - Line 1: Indonesian Title (e.g. "KECAMATAN KARAWANG TIMUR DALAM ANGKA")
/// - Line 2: English Title + right tab + year (e.g. "EAST KARAWANG DISTRICT IN FIGURES\t2026")
pub fn resolve_dda_footer_lines(
    title1: &str,
    title2: &str,
    title_en: &str,
    year: &str,
    rel_date: &str,
    rel_month: &str,
) -> (String, String) {
    let t1 = title1.trim();
    let t2 = title2.trim();
    let ten = title_en.trim();

    // Line 1: Indonesian Title
    let line1 = if !t1.is_empty() {
        if !t2.is_empty() && t2 != ten && ten.is_empty() {
            format!("{t1} {t2}")
        } else {
            t1.to_string()
        }
    } else if !t2.is_empty() && t2 != ten {
        t2.to_string()
    } else {
        "Kabupaten Dalam Angka".to_string()
    };

    // Line 2: English Title and Year
    let en = if !ten.is_empty() {
        ten
    } else if !t2.is_empty() && t2 != t1 {
        t2
    } else {
        ""
    };

    let mut y = year.trim().to_string();
    if y.is_empty() {
        for s in [rel_date, rel_month] {
            if let Some(found) = extract_year_from_text(s) {
                y = found;
                break;
            }
        }
    }
    if y.is_empty()
        && let Some(found) = extract_year_from_text(title1) {
        y = found;
    }
    if y.is_empty() {
        y = "2026".to_string();
    }

    let line2 = if !en.is_empty() {
        format!("{en}\t{y}")
    } else {
        y
    };

    (line1, line2)
}

/// Resolves the two lines for BRS publication running headers:
/// - Line 1: BRS No. [release_no], [release_date]
/// - Line 2: [Title], [release_month]
pub fn resolve_brs_header_texts(
    title: &str,
    release_no: &str,
    release_date: &str,
    release_month: &str,
) -> (String, String) {
    let raw_title = if title.trim().is_empty() {
        "Perkembangan Nilai Tukar Petani"
    } else {
        title.trim()
    };

    let raw_month = if !release_month.trim().is_empty() {
        release_month.trim()
    } else {
        release_date.trim()
    };
    let fmt_month = format_indonesian_month_year(raw_month);

    let title_line = if !fmt_month.is_empty() && !raw_title.to_lowercase().contains(&fmt_month.to_lowercase()) {
        let trimmed = raw_title.trim_end_matches(',').trim();
        format!("{trimmed}, {fmt_month}")
    } else {
        raw_title.to_string()
    };

    let rel_no = release_no.trim();
    let rel_date = if !release_date.trim().is_empty() {
        release_date.trim()
    } else {
        release_month.trim()
    };

    let release_info = if !rel_no.is_empty() {
        let fmt_date = format_indonesian_date(rel_date);
        let prefix = if rel_no.to_lowercase().starts_with("no") { "" } else { "No. " };
        if !fmt_date.is_empty() {
            format!("{prefix}{rel_no}, {fmt_date}")
        } else {
            format!("{prefix}{rel_no}")
        }
    } else {
        "No. 61/10/32/Th. XXVIII, 1 Oktober 2026".to_string()
    };

    let nopub_line = if release_info.to_uppercase().starts_with("BRS") {
        release_info
    } else {
        format!("BRS {release_info}")
    };

    (nopub_line, title_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_dda_footer() {
        let (l1, l2) = resolve_dda_footer_lines(
            "KECAMATAN KARAWANG TIMUR DALAM ANGKA",
            "",
            "EAST KARAWANG DISTRICT IN FIGURES",
            "2026",
            "",
            "",
        );
        assert_eq!(l1, "KECAMATAN KARAWANG TIMUR DALAM ANGKA");
        assert_eq!(l2, "EAST KARAWANG DISTRICT IN FIGURES\t2026");
    }

    #[test]
    fn test_resolve_brs_header() {
        let (l1, l2) = resolve_brs_header_texts(
            "Perkembangan Nilai Tukar Petani",
            "61/10/32/Th. XXVIII",
            "1 Oktober 2026",
            "September 2026",
        );
        assert_eq!(l1, "BRS No. 61/10/32/Th. XXVIII, 1 Oktober 2026");
        assert_eq!(l2, "Perkembangan Nilai Tukar Petani, September 2026");
    }
}
