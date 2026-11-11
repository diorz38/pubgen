//! The header/footer card: template picking, preview and the apply/remove/save/delete engines.

use super::*;

/// Resolves title and release number lines for BRS publication running headers.
pub fn resolve_brs_header_texts(st: &PubgenState) -> (String, String) {
    let cov_opt = st.excel_cover.as_ref();
    let raw_title = cov_opt.map(|c| c.title1.as_str()).filter(|s| !s.trim().is_empty()).unwrap_or(if st.cover_title.trim().is_empty() {
        "Perkembangan Nilai Tukar Petani"
    } else {
        st.cover_title.trim()
    });
    let raw_month = cov_opt
        .map(|c| c.release_month.as_str())
        .filter(|s| !s.trim().is_empty())
        .or_else(|| cov_opt.map(|c| c.release_date.as_str()).filter(|s| !s.trim().is_empty()))
        .unwrap_or("");
    let fmt_month = super::covers::format_indonesian_month_year(raw_month);

    let title_line = if !fmt_month.is_empty() && !raw_title.to_lowercase().contains(&fmt_month.to_lowercase()) {
        let trimmed = raw_title.trim_end_matches(',').trim();
        format!("{trimmed}, {fmt_month}")
    } else {
        raw_title.to_string()
    };

    let release_no = cov_opt.map(|c| c.release_no.as_str()).unwrap_or("").trim();
    let release_date = cov_opt
        .map(|c| c.release_date.as_str())
        .filter(|s| !s.trim().is_empty())
        .or_else(|| cov_opt.map(|c| c.release_month.as_str()).filter(|s| !s.trim().is_empty()))
        .unwrap_or("")
        .trim();
    let release_info = if !release_no.is_empty() {
        let fmt_date = super::covers::format_indonesian_date(release_date);
        let prefix = if release_no.to_lowercase().starts_with("no") { "" } else { "No. " };
        if !fmt_date.is_empty() { format!("{prefix}{release_no}, {fmt_date}") } else { format!("{prefix}{release_no}") }
    } else {
        "No. 61/10/32/Th. XXVIII, 1 Oktober 2026".to_string()
    };

    let nopub_line = if release_info.to_uppercase().starts_with("BRS") { release_info } else { format!("BRS {release_info}") };

    (title_line, nopub_line)
}

/// Resolves the single-line running-footer text of a BPS General Report: the COVER/COV
/// "Title publication" row, then the cover's title, then the page-card title. No release number
/// and no month suffix (those belong to a BRS bulletin footer).
pub fn resolve_general_report_footer_text(st: &PubgenState) -> String {
    let from_cover = st
        .excel_cover
        .as_ref()
        .and_then(|cov| [cov.title_publication.trim(), cov.title1.trim()].into_iter().find(|s| !s.is_empty()).map(str::to_string));
    let text = from_cover.unwrap_or_else(|| st.cover_title.trim().to_string());
    // A publication title is one line: a multi-line cover cell contributes its first non-empty line.
    text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("").to_string()
}

/// Helper to extract a 4-digit year like 2026 from date or month strings.
fn extract_year_from_text(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    if bytes.len() < 4 {
        return None;
    }
    for i in 0..=bytes.len() - 4 {
        let chunk = &s[i..i + 4];
        if chunk.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(y) = chunk.parse::<u32>() {
                if (1900..=2100).contains(&y) {
                    return Some(chunk.to_string());
                }
            }
        }
    }
    None
}

/// Resolves the two lines for BPS DDA footer:
/// Line 1: Indonesian title (e.g. "KECAMATAN KARAWANG TIMUR DALAM ANGKA")
/// Line 2: English title with optional right-indent tab and year (e.g. "EAST KARAWANG DISTRICT IN FIGURES\u{e006}2026")
pub fn resolve_dda_footer_lines(st: &PubgenState) -> (String, String) {
    let (raw_title, raw_title2, raw_title_en, raw_year, raw_rel_date, raw_rel_month) = if let Some(ref cov) = st.excel_cover {
        (cov.title1.as_str(), cov.title2.as_str(), cov.title_en.as_str(), cov.year.as_str(), cov.release_date.as_str(), cov.release_month.as_str())
    } else {
        (st.cover_title.as_str(), st.cover_title2.as_str(), st.cover_title_en.as_str(), st.cover_year.as_str(), "", "")
    };

    let clean_single_line = |s: &str| -> String {
        let first = s.lines().next().unwrap_or("").trim();
        let stripped = first
            .strip_prefix("Cover Release: ")
            .or_else(|| first.strip_prefix("Sampul Rilis: "))
            .or_else(|| first.strip_prefix("Sampul Publikasi: "))
            .or_else(|| first.strip_prefix("Publication Cover: "))
            .or_else(|| first.strip_prefix("Sampul: "))
            .or_else(|| first.strip_prefix("Cover: "))
            .unwrap_or(first)
            .trim();
        stripped.to_string()
    };

    let title1 = clean_single_line(raw_title);
    let title2 =
        if raw_title2.contains('\n') || raw_title2.to_lowercase().contains("statistik") { String::new() } else { clean_single_line(raw_title2) };
    let title_en = clean_single_line(raw_title_en);

    // Line 1: Indonesian Title
    let line1 = if !title1.is_empty() {
        title1
    } else if !title2.is_empty() && title2 != title_en {
        title2.clone()
    } else {
        "Kabupaten Dalam Angka".to_string()
    };

    // Line 2: English Title and Year
    // In BPS DDA, title2 from sheet COVER holds the official English publication title.
    // Prefer title2, falling back to title_en if title2 is absent.
    let en = if !title2.is_empty() && title2 != line1 {
        title2
    } else if !title_en.is_empty() && title_en != line1 {
        title_en
    } else {
        String::new()
    };

    let mut year = raw_year.trim().to_string();
    if year.is_empty() {
        for s in [raw_rel_date, raw_rel_month] {
            if let Some(y) = extract_year_from_text(s) {
                year = y;
                break;
            }
        }
    }

    let line2 = if !en.is_empty() && !year.is_empty() {
        format!("{en}\u{e006}{year}")
    } else if !en.is_empty() {
        en
    } else if !year.is_empty() {
        format!("\u{e006}{year}")
    } else {
        String::new()
    };

    (line1, line2)
}

/// Resolves complete two-line footer text for BPS DDA.
pub fn resolve_dda_footer_text(st: &PubgenState) -> String {
    let (line1, line2) = resolve_dda_footer_lines(st);
    if !line2.is_empty() { format!("{line1}\n{line2}") } else { line1 }
}

pub fn resolve_dda_publication_title(st: &PubgenState) -> String {
    let (line1, line2) = resolve_dda_footer_lines(st);
    let en_part = line2.split('\u{e006}').next().unwrap_or("").trim();
    if !en_part.is_empty() && en_part != line1 { format!("{line1} / {en_part}") } else { line1 }
}

fn load_hf_template_to_state(st: &mut PubgenState, tpl: &Value) {
    if let Some(name) = tpl.get("name").and_then(Value::as_str) {
        st.hf_style = name.to_string();
        if name.eq_ignore_ascii_case("hf-dda") || name.eq_ignore_ascii_case("dda") || name.eq_ignore_ascii_case("bps-dda") {
            st.hf_sided = 0;
            st.hf_header_offset = tpl.get("headerOffset").and_then(Value::as_f64).unwrap_or(24.0);
            st.hf_footer_offset = tpl.get("footerOffset").and_then(Value::as_f64).unwrap_or(24.0);
            st.hf_header_single = "{section}\u{e006}{page}".into();
            st.hf_header_left = "{section}\u{e006}{page}".into();
            st.hf_header_right = "{page}\u{e006}{section}".into();
            let dda_footer = resolve_dda_footer_text(st);
            st.hf_footer_single = dda_footer.clone();
            st.hf_footer_left = dda_footer.clone();
            st.hf_footer_right = dda_footer;
            st.hf_header_rule = false;
            st.hf_footer_rule = false;
            st.hf_top_accent_bar = false;
            return;
        }
        if name.eq_ignore_ascii_case("hf-bps-general") || name.eq_ignore_ascii_case("bps-general") {
            st.hf_sided = 0;
            st.hf_header_offset = tpl.get("headerOffset").and_then(Value::as_f64).unwrap_or(24.0);
            st.hf_footer_offset = tpl.get("footerOffset").and_then(Value::as_f64).unwrap_or(24.0);
            let footer = resolve_general_report_footer_text(st);
            let footer = if footer.is_empty() { "{title_publication}".to_string() } else { footer };
            st.hf_header_single.clear();
            st.hf_header_left.clear();
            st.hf_header_right.clear();
            st.hf_footer_single = footer.clone();
            st.hf_footer_left = footer.clone();
            st.hf_footer_right = footer;
            st.hf_header_rule = false;
            st.hf_footer_rule = false;
            st.hf_top_accent_bar = false;
            st.hf_color_primary = "#000000".to_string();
            st.hf_color_text = "#000000".to_string();
            return;
        }
        if name.eq_ignore_ascii_case("hf-brs") || name.eq_ignore_ascii_case("brs") {
            st.hf_sided = 0;
            st.hf_header_offset = tpl.get("headerOffset").and_then(Value::as_f64).unwrap_or(24.0);
            st.hf_footer_offset = tpl.get("footerOffset").and_then(Value::as_f64).unwrap_or(24.0);
            let (title, nopub) = resolve_brs_header_texts(st);
            let combined = format!("{title}\n{nopub}");
            st.hf_header_single.clear();
            st.hf_header_left.clear();
            st.hf_header_right.clear();
            st.hf_footer_single = combined.clone();
            st.hf_footer_left = combined.clone();
            st.hf_footer_right = combined;
            st.hf_header_rule = false;
            st.hf_footer_rule = false;
            st.hf_top_accent_bar = false;
            st.hf_color_primary = "#f68839".to_string();
            st.hf_color_text = "#f68839".to_string();
            return;
        }
    }
    if let Some(sided) = tpl.get("sided").and_then(Value::as_str) {
        st.hf_sided = if sided.eq_ignore_ascii_case("single") { 1 } else { 0 };
    }
    if let Some(ho) = tpl.get("headerOffset").and_then(Value::as_f64) {
        st.hf_header_offset = ho;
    }
    if let Some(fo) = tpl.get("footerOffset").and_then(Value::as_f64) {
        st.hf_footer_offset = fo;
    }
    if let Some(h) = tpl.get("header") {
        st.hf_header_single = h.get("single").and_then(Value::as_str).unwrap_or("").to_string();
        st.hf_header_left = h.get("left").and_then(Value::as_str).unwrap_or("").to_string();
        st.hf_header_right = h.get("right").and_then(Value::as_str).unwrap_or("").to_string();
    }
    if let Some(f) = tpl.get("footer") {
        st.hf_footer_single = f.get("single").and_then(Value::as_str).unwrap_or("").to_string();
        st.hf_footer_left = f.get("left").and_then(Value::as_str).unwrap_or("").to_string();
        st.hf_footer_right = f.get("right").and_then(Value::as_str).unwrap_or("").to_string();
    }
    if let Some(obj) = tpl.get("objects") {
        st.hf_header_rule = obj.get("headerRule").and_then(Value::as_bool).unwrap_or(false);
        st.hf_header_rule_weight = obj.get("headerRuleWeight").and_then(Value::as_f64).unwrap_or(0.5);
        st.hf_footer_rule = obj.get("footerRule").and_then(Value::as_bool).unwrap_or(false);
        st.hf_footer_rule_weight = obj.get("footerRuleWeight").and_then(Value::as_f64).unwrap_or(0.5);
        st.hf_top_accent_bar = obj.get("topAccentBar").and_then(Value::as_bool).unwrap_or(false);
        st.hf_top_accent_bar_height = obj.get("topAccentBarHeight").and_then(Value::as_f64).unwrap_or(4.0);
        st.hf_top_accent_bar_bleed = obj.get("topAccentBarBleed").and_then(Value::as_bool).unwrap_or(true);
    }
    if let Some(cols) = tpl.get("colors") {
        st.hf_color_primary = cols.get("primary").and_then(Value::as_str).unwrap_or("#0072bc").to_string();
        st.hf_color_accent = cols.get("accent").and_then(Value::as_str).unwrap_or("#f5ab19").to_string();
        st.hf_color_dark = cols.get("dark").and_then(Value::as_str).unwrap_or("#122647").to_string();
        st.hf_color_rule = cols.get("ruleColor").and_then(Value::as_str).unwrap_or("#cbd5e1").to_string();
        st.hf_color_text = cols.get("textColor").and_then(Value::as_str).unwrap_or("#334155").to_string();
    }
}

fn show_hf_preview_tooltip(ui: &mut Ui, st: &PubgenState, name: &str, t: &Tokens) {
    ui.vertical(|ui| {
        ui.label(RichText::new(name).size(12.0).strong().color(t.text_strong));
        ui.add_space(4.0);

        let (rect, _) = ui.allocate_exact_size(vec2(160.0, 110.0), Sense::hover());
        // Page background
        ui.painter().rect_filled(rect, 3.0, Color32::from_rgb(250, 250, 250));
        ui.painter().rect_stroke(rect, 3.0, egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)), egui::StrokeKind::Outside);

        if name.eq_ignore_ascii_case("hf-brs") || name.eq_ignore_ascii_case("brs") {
            let spine_x = rect.center().x;
            ui.painter().line_segment([pos2(spine_x, rect.min.y), pos2(spine_x, rect.max.y)], egui::Stroke::new(1.0, Color32::from_gray(210)));

            let brs_orange = Color32::from_rgb(245, 136, 56);
            let footer_y = rect.max.y - 20.0;

            // Faint body lines on both pages
            let top_body_y = rect.min.y + 14.0;
            for i in 0..5 {
                let by = top_body_y + (i as f32) * 8.0;
                ui.painter().line_segment([pos2(rect.min.x + 8.0, by), pos2(spine_x - 8.0, by)], egui::Stroke::new(1.0, Color32::from_gray(230)));
                ui.painter().line_segment([pos2(spine_x + 8.0, by), pos2(rect.max.x - 8.0, by)], egui::Stroke::new(1.0, Color32::from_gray(230)));
            }

            // Verso (Left page): 2-line text on left (right-aligned), folio on right in footer
            let left_text_x = spine_x - 32.0;
            ui.painter().text(
                pos2(left_text_x, footer_y),
                egui::Align2::RIGHT_TOP,
                "Title Line\nRelease No",
                egui::FontId::proportional(7.0),
                t.text_dim,
            );
            ui.painter().text(pos2(spine_x - 10.0, footer_y), egui::Align2::RIGHT_TOP, "9", egui::FontId::proportional(8.5), brs_orange);

            // Recto (Right page): folio on left, 2-line text on right (left-aligned) in footer
            ui.painter().text(pos2(spine_x + 10.0, footer_y), egui::Align2::LEFT_TOP, "10", egui::FontId::proportional(8.5), brs_orange);
            let right_text_x = spine_x + 32.0;
            ui.painter().text(
                pos2(right_text_x, footer_y),
                egui::Align2::LEFT_TOP,
                "Title Line\nRelease No",
                egui::FontId::proportional(7.0),
                t.text_dim,
            );

            ui.add_space(3.0);
            ui.label(
                RichText::new(
                    "BPS Berita Resmi Statistik running footer in Raleway Thin 10 with a Lato Regular 25 folio on the spine side (blank header).",
                )
                .size(10.0)
                .color(t.text_dim),
            );
            return;
        }

        if name.eq_ignore_ascii_case("hf-dda") || name.eq_ignore_ascii_case("dda") || name.eq_ignore_ascii_case("bps-dda") {
            let spine_x = rect.center().x;
            ui.painter().line_segment([pos2(spine_x, rect.min.y), pos2(spine_x, rect.max.y)], egui::Stroke::new(1.0, Color32::from_gray(210)));

            let top_y = rect.min.y + 10.0;
            let footer_y = rect.max.y - 12.0;

            let top_body_y = rect.min.y + 24.0;
            for i in 0..5 {
                let by = top_body_y + (i as f32) * 8.0;
                ui.painter().line_segment([pos2(rect.min.x + 8.0, by), pos2(spine_x - 8.0, by)], egui::Stroke::new(1.0, Color32::from_gray(230)));
                ui.painter().line_segment([pos2(spine_x + 8.0, by), pos2(rect.max.x - 8.0, by)], egui::Stroke::new(1.0, Color32::from_gray(230)));
            }

            // Verso: Running header ({section} ... {page}), footer: 2 lines
            ui.painter().text(
                pos2(rect.min.x + 8.0, top_y),
                egui::Align2::LEFT_TOP,
                "BAB 01 ...",
                egui::FontId::proportional(7.5),
                Color32::from_gray(100),
            );
            ui.painter().text(pos2(spine_x - 8.0, top_y), egui::Align2::RIGHT_TOP, "2", egui::FontId::proportional(7.5), Color32::from_gray(100));
            ui.painter().text(
                pos2(rect.min.x + 8.0, footer_y - 6.0),
                egui::Align2::LEFT_TOP,
                "Judul Indonesia\nEnglish Title",
                egui::FontId::proportional(6.0),
                Color32::from_gray(120),
            );
            ui.painter().text(
                pos2(spine_x - 8.0, footer_y),
                egui::Align2::RIGHT_BOTTOM,
                "2026",
                egui::FontId::proportional(6.5),
                Color32::from_gray(120),
            );

            // Recto: Running header ({page} ... {section}), footer: 2 lines
            ui.painter().text(pos2(spine_x + 8.0, top_y), egui::Align2::LEFT_TOP, "3", egui::FontId::proportional(7.5), Color32::from_gray(100));
            ui.painter().text(
                pos2(rect.max.x - 8.0, top_y),
                egui::Align2::RIGHT_TOP,
                "BAB 01 ...",
                egui::FontId::proportional(7.5),
                Color32::from_gray(100),
            );
            ui.painter().text(
                pos2(spine_x + 8.0, footer_y - 6.0),
                egui::Align2::LEFT_TOP,
                "Judul Indonesia\nEnglish Title",
                egui::FontId::proportional(6.0),
                Color32::from_gray(120),
            );
            ui.painter().text(
                pos2(rect.max.x - 8.0, footer_y),
                egui::Align2::RIGHT_BOTTOM,
                "2026",
                egui::FontId::proportional(6.5),
                Color32::from_gray(120),
            );

            ui.add_space(3.0);
            ui.label(
                RichText::new(
                    "BPS Daerah Dalam Angka running header (chapter title with page number) and 2-line publication title footer with year.",
                )
                .size(10.0)
                .color(t.text_dim),
            );
            return;
        }

        let parse_color = |s: &str, def: Color32| {
            if let Some(c) = designcraft_color::Color::from_hex(s) {
                let [r, g, b, _] = c.to_rgba8(1.0);
                Color32::from_rgb(r, g, b)
            } else {
                def
            }
        };

        let primary_color = parse_color(&st.hf_color_primary, Color32::from_rgb(0, 114, 188));
        let rule_color = parse_color(&st.hf_color_rule, Color32::from_rgb(200, 200, 200));

        // 1. Top accent bar
        if st.hf_top_accent_bar {
            let bar_rect = egui::Rect::from_min_size(rect.min, vec2(rect.width(), 4.0));
            ui.painter().rect_filled(bar_rect, 0.0, primary_color);
        }

        // 2. Header text & rule
        let header_y = rect.min.y + 14.0;
        let left_x = rect.min.x + 8.0;
        let right_x = rect.max.x - 8.0;

        if !st.hf_header_left.is_empty() || !st.hf_header_single.is_empty() {
            ui.painter().text(pos2(left_x, header_y), egui::Align2::LEFT_CENTER, "Headline", egui::FontId::proportional(8.0), t.text_dim);
        }
        if !st.hf_header_right.is_empty() {
            ui.painter().text(pos2(right_x, header_y), egui::Align2::RIGHT_CENTER, "Section", egui::FontId::proportional(8.0), t.text_dim);
        }
        if st.hf_header_rule {
            ui.painter().line_segment([pos2(left_x, header_y + 6.0), pos2(right_x, header_y + 6.0)], egui::Stroke::new(1.0, rule_color));
        }

        // Faint body lines
        let body_y0 = header_y + 14.0;
        for i in 0..4 {
            let by = body_y0 + (i as f32) * 8.0;
            let bw = if i == 3 { rect.width() * 0.45 } else { rect.width() - 24.0 };
            ui.painter().line_segment([pos2(left_x + 4.0, by), pos2(left_x + 4.0 + bw, by)], egui::Stroke::new(1.0, Color32::from_gray(230)));
        }

        // 3. Footer text & rule
        let footer_y = rect.max.y - 12.0;
        if st.hf_footer_rule {
            ui.painter().line_segment([pos2(left_x, footer_y - 6.0), pos2(right_x, footer_y - 6.0)], egui::Stroke::new(1.0, rule_color));
        }
        ui.painter().text(pos2(left_x, footer_y), egui::Align2::LEFT_CENTER, "12", egui::FontId::proportional(8.0), t.text_dim);
        ui.painter().text(pos2(right_x, footer_y), egui::Align2::RIGHT_CENTER, "Document", egui::FontId::proportional(8.0), t.text_dim);

        ui.add_space(3.0);
        let desc = match name {
            "Modern Minimal" => "Clean, understated page number with subtle footer divider.",
            "Editorial Book" => "Refined running headers with hairline rule, ideal for magazines and books.",
            "Corporate Report" => "Full-width top accent bar, running headers, and confidentiality footer.",
            "hf-dda" => "BPS Daerah Dalam Angka running header (chapter title) and footer with publication title and page number.",
            _ => "Custom user header & footer template.",
        };
        ui.label(RichText::new(desc).size(10.0).color(t.text_dim));
    });
}

#[allow(dead_code)]
fn card_header_footer_ui(app: &mut DesignApp, ui: &mut Ui, st: &mut PubgenState, t: &Tokens) {
    section(ui, crate::i18n::tr(&app.ui.language, "Header & Footer Template"), true);
    card_header_footer_body_ui(app, ui, st, t);
}

pub(crate) fn card_header_footer_body_ui(app: &mut DesignApp, ui: &mut Ui, st: &mut PubgenState, t: &Tokens) {
    // Fetch all templates (built-in + user custom) from engine
    let templates = designcraft_engine::cmd::header_footer::all_templates(&app.session);

    // Lazy initialize from default or current doc if not yet initialized
    if !st.hf_initialized {
        st.hf_initialized = true;
        if let Some(doc_hf) = app.session.active().and_then(|s| s.doc.header_footer.as_ref()) {
            st.hf_style = doc_hf.preset.clone();
            st.hf_sided = if doc_hf.sided == designcraft_doc::Sided::Single { 1 } else { 0 };
            st.hf_header_offset = doc_hf.header_offset;
            st.hf_footer_offset = doc_hf.footer_offset;
            st.hf_header_single = doc_hf.header.single.clone();
            st.hf_header_left = doc_hf.header.left.clone();
            st.hf_header_right = doc_hf.header.right.clone();
            st.hf_footer_single = doc_hf.footer.single.clone();
            st.hf_footer_left = doc_hf.footer.left.clone();
            st.hf_footer_right = doc_hf.footer.right.clone();
            st.hf_header_rule = doc_hf.objects.header_rule;
            st.hf_header_rule_weight = doc_hf.objects.header_rule_weight;
            st.hf_footer_rule = doc_hf.objects.footer_rule;
            st.hf_footer_rule_weight = doc_hf.objects.footer_rule_weight;
            st.hf_top_accent_bar = doc_hf.objects.top_accent_bar;
            st.hf_top_accent_bar_height = doc_hf.objects.top_accent_bar_height;
            st.hf_top_accent_bar_bleed = doc_hf.objects.top_accent_bar_bleed;
            st.hf_color_primary = doc_hf.colors.primary.clone();
            st.hf_color_accent = doc_hf.colors.accent.clone();
            st.hf_color_dark = doc_hf.colors.dark.clone();
            st.hf_color_rule = doc_hf.colors.rule_color.clone();
            st.hf_color_text = doc_hf.colors.text_color.clone();
        } else if let Some(first_tpl) = templates.first() {
            load_hf_template_to_state(st, first_tpl);
        }
    }

    // 1. Template Style Dropdown with Hover Preview Tooltip & Eye Icon
    ui.horizontal(|ui| {
        ui.label(RichText::new("Style:").size(11.0).color(t.text));

        let cur_style = st.hf_style.clone();
        let combo_w = (ui.available_width() - 80.0).clamp(110.0, 200.0);
        let mut picked_template: Option<Value> = None;

        let combo = craft_ui::ComboBox::from_id_salt("pubgen_hf_style").selected_text(&cur_style).width(combo_w).show_ui(ui, |ui| {
            for tpl in &templates {
                let name = tpl.get("name").and_then(Value::as_str).unwrap_or("Template");
                let is_builtin = tpl.get("builtIn").and_then(Value::as_bool).unwrap_or(false);
                let label = if is_builtin { name.to_string() } else { format!("{name} ★") };
                let is_selected = cur_style == name;
                let resp = ui.selectable_label(is_selected, label);
                if resp.clicked() && !is_selected {
                    picked_template = Some(tpl.clone());
                }
                resp.on_hover_ui(|ui| {
                    show_hf_preview_tooltip(ui, st, name, t);
                });
            }
        });

        if let Some(tpl) = picked_template {
            load_hf_template_to_state(st, &tpl);
        }

        combo.response.on_hover_ui(|ui| {
            show_hf_preview_tooltip(ui, st, &cur_style, t);
        });

        // Eye preview button
        let (thumb_icon_rect, thumb_resp) = ui.allocate_exact_size(vec2(22.0, 20.0), Sense::click());
        let icon_hovered = thumb_resp.hovered();
        ui.painter().rect_filled(thumb_icon_rect, 3.0, if icon_hovered { t.accent } else { t.panel });
        ui.painter().rect_stroke(thumb_icon_rect, 3.0, egui::Stroke::new(1.0, t.border), egui::StrokeKind::Outside);
        ui.painter().text(
            thumb_icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            "👁",
            egui::FontId::proportional(11.0),
            if icon_hovered { Color32::WHITE } else { t.text_dim },
        );
        thumb_resp.on_hover_ui(|ui| {
            show_hf_preview_tooltip(ui, st, &cur_style, t);
        });

        // If current template is custom (not built-in), show a small delete button
        let cur_is_builtin = templates
            .iter()
            .any(|tpl| tpl.get("name").and_then(Value::as_str) == Some(&cur_style) && tpl.get("builtIn").and_then(Value::as_bool) == Some(true));
        if !cur_is_builtin && !cur_style.is_empty() {
            let del_btn = ui.button("🗑").on_hover_text("Delete this custom template");
            if del_btn.clicked() {
                match execute_delete_hf_template(app, st, &cur_style) {
                    Ok(msg) => st.hf_status = Some((msg, true)),
                    Err(err) => st.hf_status = Some((err, false)),
                }
            }
        }
    });

    // 2. Sidedness & Offsets
    ui.horizontal(|ui| {
        ui.label(RichText::new("Sided:").size(11.0).color(t.text));
        let sided_options = ["Double-sided", "Single-sided"];
        let cur_s = sided_options.get(st.hf_sided).copied().unwrap_or("Double-sided");
        craft_ui::ComboBox::from_id_salt("pubgen_hf_sided").selected_text(cur_s).width(95.0).show_ui(ui, |ui| {
            ui.selectable_value(&mut st.hf_sided, 0, "Double-sided");
            ui.selectable_value(&mut st.hf_sided, 1, "Single-sided");
        });

        ui.label(RichText::new("H.Off:").size(10.5).color(t.text_dim));
        ui.add(egui::DragValue::new(&mut st.hf_header_offset).range(0.0..=120.0).speed(0.5).suffix(" pt"))
            .on_hover_text("Header offset from top of page");

        ui.label(RichText::new("F.Off:").size(10.5).color(t.text_dim));
        ui.add(egui::DragValue::new(&mut st.hf_footer_offset).range(0.0..=120.0).speed(0.5).suffix(" pt"))
            .on_hover_text("Footer offset from bottom of page");
    });

    // 2b. Scope: which pages this template applies to (all, one section, a page range, this page only, or this page forward).
    ui.horizontal(|ui| {
        ui.label(RichText::new("Scope:").size(11.0).color(t.text));
        let (page_count, sections) = app
            .session
            .active()
            .map(|a| (a.doc.page_count(), a.doc.sections.iter().map(|s| (s.prefix.clone(), s.start)).collect::<Vec<_>>()))
            .unwrap_or((1, Vec::new()));
        let cur_scope = match st.hf_scope {
            1 => "Section",
            2 => "Pages",
            3 => "This Page Only",
            4 => "This Page Forward",
            _ => "All Pages",
        };
        craft_ui::ComboBox::from_id_salt("pubgen_hf_scope").selected_text(cur_scope).width(115.0).show_ui(ui, |ui| {
            ui.selectable_value(&mut st.hf_scope, 0, "All Pages");
            ui.selectable_value(&mut st.hf_scope, 3, "This Page Only");
            ui.selectable_value(&mut st.hf_scope, 4, "This Page Forward");
            ui.selectable_value(&mut st.hf_scope, 1, "Section");
            ui.selectable_value(&mut st.hf_scope, 2, "Pages");
        });
        match st.hf_scope {
            1 => {
                if st.hf_scope_section >= sections.len() {
                    st.hf_scope_section = 0;
                }
                let sec_label = |i: usize, s: &(String, usize)| {
                    if s.0.is_empty() { format!("Section {} (page {})", i + 1, s.1 + 1) } else { format!("{} (page {})", s.0, s.1 + 1) }
                };
                let cur_sec = sections
                    .get(st.hf_scope_section)
                    .map_or_else(|| format!("Section {}", st.hf_scope_section + 1), |s| sec_label(st.hf_scope_section, s));
                craft_ui::ComboBox::from_id_salt("pubgen_hf_scope_section").selected_text(cur_sec).width(130.0).show_ui(ui, |ui| {
                    for (i, s) in sections.iter().enumerate() {
                        if ui.selectable_label(st.hf_scope_section == i, sec_label(i, s)).clicked() {
                            st.hf_scope_section = i;
                        }
                    }
                });
            }
            2 => {
                let start = st.hf_scope_start.clamp(1, page_count.max(1));
                let end = st.hf_scope_end.clamp(start, page_count.max(1));
                (st.hf_scope_start, st.hf_scope_end) = (start, end);
                ui.add(egui::DragValue::new(&mut st.hf_scope_start).range(1..=page_count.max(1)).prefix("From: "));
                ui.add(egui::DragValue::new(&mut st.hf_scope_end).range(st.hf_scope_start..=page_count.max(1)).prefix("To: "));
            }
            3 => {
                let p = st.hf_scope_start.clamp(1, page_count.max(1));
                ui.label(RichText::new(format!("(Page {p})")).size(10.5).color(t.text_dim));
            }
            4 => {
                let p = st.hf_scope_start.clamp(1, page_count.max(1));
                ui.label(RichText::new(format!("(Page {p} to {page_count})")).size(10.5).color(t.text_dim));
            }
            _ => {}
        }
        if ui.button(RichText::new("Sections…").size(10.5).color(t.text_dim)).clicked() {
            crate::menus::activate(app, "layout.section", &Value::Null);
        }
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("Quick Scope:").size(10.5).color(t.text_dim));
        if ui.selectable_label(st.hf_scope == 3, "This Page Only").clicked() {
            st.hf_scope = 3;
        }
        if ui.selectable_label(st.hf_scope == 4, "This Page Forward").clicked() {
            st.hf_scope = 4;
        }
        if ui.selectable_label(st.hf_scope == 0, "All Pages").clicked() {
            st.hf_scope = 0;
        }
    });

    // 3. Header Text Fields
    if st.hf_sided == 0 {
        // Double-sided headers
        ui.horizontal(|ui| {
            ui.label(RichText::new("H.Left:").size(10.5).color(t.text));
            let avail = (ui.available_width() - 4.0).max(60.0);
            ui.add_sized([avail, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_header_left).hint_text("{page} | Title"))
                .on_hover_text("Left (verso) header text. Tokens: {page}, {section}, {filename}, {runningHeader}");
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("H.Right:").size(10.5).color(t.text));
            let avail = (ui.available_width() - 4.0).max(60.0);
            ui.add_sized([avail, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_header_right).hint_text("Title | {page}"))
                .on_hover_text("Right (recto) header text. Tokens: {page}, {section}, {filename}, {runningHeader}");
        });
    } else {
        // Single-sided header
        ui.horizontal(|ui| {
            ui.label(RichText::new("Header:").size(10.5).color(t.text));
            let avail = (ui.available_width() - 4.0).max(60.0);
            ui.add_sized([avail, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_header_single).hint_text("{page} | Document"))
                .on_hover_text("Single-sided header text. Tokens: {page}, {section}, {filename}, {runningHeader}");
        });
    }

    // 4. Footer Text Fields
    if st.hf_sided == 0 {
        // Double-sided footers
        ui.horizontal(|ui| {
            ui.label(RichText::new("F.Left:").size(10.5).color(t.text));
            let avail = (ui.available_width() - 4.0).max(60.0);
            ui.add_sized([avail, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_footer_left).hint_text("{page}"))
                .on_hover_text("Left (verso) footer text");
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("F.Right:").size(10.5).color(t.text));
            let avail = (ui.available_width() - 4.0).max(60.0);
            ui.add_sized([avail, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_footer_right).hint_text("Confidential"))
                .on_hover_text("Right (recto) footer text");
        });
    } else {
        // Single-sided footer
        ui.horizontal(|ui| {
            ui.label(RichText::new("Footer:").size(10.5).color(t.text));
            let avail = (ui.available_width() - 4.0).max(60.0);
            ui.add_sized([avail, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_footer_single).hint_text("{page}"))
                .on_hover_text("Single-sided footer text");
        });
    }

    // 5. Decorative Objects (Rules & Accent Bar)
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.checkbox(&mut st.hf_header_rule, RichText::new("H.Rule").size(10.5).color(t.text)).on_hover_text("Thin hairline rule below header text");
        if st.hf_header_rule {
            ui.add(egui::DragValue::new(&mut st.hf_header_rule_weight).range(0.25..=6.0).speed(0.25).suffix("pt"));
        }

        ui.checkbox(&mut st.hf_footer_rule, RichText::new("F.Rule").size(10.5).color(t.text)).on_hover_text("Thin hairline rule above footer text");
        if st.hf_footer_rule {
            ui.add(egui::DragValue::new(&mut st.hf_footer_rule_weight).range(0.25..=6.0).speed(0.25).suffix("pt"));
        }
    });

    ui.horizontal(|ui| {
        ui.checkbox(&mut st.hf_top_accent_bar, RichText::new("Top Accent Bar").size(10.5).color(t.text))
            .on_hover_text("Colored geometric accent bar across the top page edge");
        if st.hf_top_accent_bar {
            ui.add(egui::DragValue::new(&mut st.hf_top_accent_bar_height).range(1.0..=36.0).speed(0.5).suffix("pt"))
                .on_hover_text("Height of top accent bar");
            ui.checkbox(&mut st.hf_top_accent_bar_bleed, RichText::new("Bleed").size(10.0).color(t.text_dim))
                .on_hover_text("Extend accent bar into bleed area");
        }
    });

    // 6. Template Colors
    ui.horizontal(|ui| {
        ui.label(RichText::new("Colors:").size(10.5).color(t.text_dim));
        let field_w = ((ui.available_width() - 40.0) / 4.0 - 4.0).clamp(38.0, 60.0);
        ui.add_sized([field_w, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_color_primary))
            .on_hover_text("Primary / Accent Bar color hex (e.g. #0072bc)");
        ui.add_sized([field_w, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_color_accent))
            .on_hover_text("Accent color hex (e.g. #f5ab19)");
        ui.add_sized([field_w, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_color_rule))
            .on_hover_text("Hairline rule color hex (e.g. #cbd5e1)");
        ui.add_sized([field_w, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_color_text))
            .on_hover_text("Text color hex (e.g. #334155)");
    });

    // 7. Save As New Template (Collapsible / Toggleable)
    ui.add_space(3.0);
    ui.horizontal(|ui| {
        let toggle_text = if st.hf_show_save_row { "▾ Save As New Template…" } else { "▸ Save As New Template…" };
        if ui.button(RichText::new(toggle_text).size(10.5).color(t.text_dim)).clicked() {
            st.hf_show_save_row = !st.hf_show_save_row;
        }
    });

    if st.hf_show_save_row {
        ui.horizontal(|ui| {
            let avail = (ui.available_width() - 84.0).max(60.0);
            ui.add_sized([avail, crate::widgets::FIELD_H], egui::TextEdit::singleline(&mut st.hf_custom_name).hint_text("Template Name…"));
            let save_btn = ui.button(RichText::new("Save").size(11.0).color(t.text_strong));
            if save_btn.clicked() {
                match execute_save_hf_template(app, st) {
                    Ok(msg) => st.hf_status = Some((msg, true)),
                    Err(err) => st.hf_status = Some((err, false)),
                }
            }
        });
    }

    // 8. Action Buttons (Apply & Remove)
    ui.add_space(4.0);
    let fw = full_width(ui);
    let apply_btn =
        outline_button(ui, "Apply Header & Footer", fw).on_hover_text("Applies the current header & footer template configuration to the document");
    if apply_btn.clicked() {
        let res = execute_apply_header_footer(app, st);
        st.hf_status = Some(match res {
            Ok(msg) => (msg, true),
            Err(err) => (err, false),
        });
    }

    ui.horizontal(|ui| {
        if ui.button(RichText::new("Remove from Document").size(10.5).color(t.text_dim)).clicked() {
            match execute_remove_header_footer(app) {
                Ok(msg) => st.hf_status = Some((msg, true)),
                Err(err) => st.hf_status = Some((err, false)),
            }
        }
    });

    // Status message
    if let Some((msg, ok)) = &st.hf_status {
        let color = if *ok { Color32::from_rgb(80, 200, 120) } else { Color32::from_rgb(220, 80, 80) };
        ui.add(egui::Label::new(RichText::new(msg).size(11.0).color(color)).wrap());
    }
}

pub fn execute_apply_header_footer(app: &mut DesignApp, st: &PubgenState) -> Result<String, String> {
    // If double-sided is requested and facingPages is currently false, ensure facingPages is true
    if st.hf_sided == 0 && app.session.active().is_some_and(|s| !s.doc.settings.facing_pages) {
        app.run("layout.documentSetup", json!({ "facingPages": true })).map_err(|e| format!("Failed to switch the document to facing pages: {e}"))?;
    }

    let is_brs = st.hf_style.eq_ignore_ascii_case("hf-brs") || st.hf_style.eq_ignore_ascii_case("brs");
    let is_general = st.hf_style.eq_ignore_ascii_case("hf-bps-general") || st.hf_style.eq_ignore_ascii_case("bps-general");
    let is_dda = st.hf_style.eq_ignore_ascii_case("hf-dda") || st.hf_style.eq_ignore_ascii_case("dda") || st.hf_style.eq_ignore_ascii_case("bps-dda");

    // Same shapes the Header/Footer… dialog sends: "all", {"section": 0-based index}, {"range": [first, last]}.
    let scope = match st.hf_scope {
        1 => json!({ "section": st.hf_scope_section }),
        2 => {
            let start = st.hf_scope_start.max(1) as u64;
            let end = (st.hf_scope_end as u64).max(start);
            json!({ "range": [start, end] })
        }
        3 => {
            // Apply for this page only
            let p = st.hf_scope_start.max(1) as u64;
            json!({ "range": [p, p] })
        }
        4 => {
            // Apply from this page forward
            let start = st.hf_scope_start.max(1) as u64;
            let total = app.session.active().map(|s| s.doc.page_count()).unwrap_or(start as usize);
            let mut last = (total as u64).max(start);
            if (is_brs || is_dda || is_general)
                && let Some(active) = app.session.active()
            {
                let (_, _, back) = super::covers::find_existing_covers(&active.doc);
                if let Some(b) = back
                    && b > 0
                {
                    last = (b as u64).min(last);
                }
            }
            if start <= last { json!({ "range": [start, last] }) } else { json!({ "range": [start, start] }) }
        }
        _ => {
            // For BRS, DDA and the general report, running headers should not land on cover
            // (page 1) or back cover (last page)
            if is_brs || is_dda || is_general {
                if let Some(active) = app.session.active() {
                    let total = active.doc.page_count();
                    if total > 1 {
                        let (front, inside, back) = super::covers::find_existing_covers(&active.doc);
                        let mut start = 1u64;
                        if is_dda && (front.is_some() || inside.is_some()) {
                            // DDA rule: page cover (1), 2nd cover (2), A02 (3) [none]
                            start = 4;
                        } else if inside.is_some() {
                            start = 3;
                        } else if front.is_some() || is_brs || is_general {
                            start = 2;
                        }
                        let mut last = total as u64;
                        if let Some(b) = back {
                            if b > 0 {
                                last = b as u64;
                            }
                        }
                        if start <= last { json!({ "range": [start, last] }) } else { json!("all") }
                    } else {
                        json!("all")
                    }
                } else {
                    json!("all")
                }
            } else {
                json!("all")
            }
        }
    };

    let (brs_title, brs_nopub) = resolve_brs_header_texts(st);
    let general_footer = resolve_general_report_footer_text(st);
    let (dda_title_line, dda_sub_line) = resolve_dda_footer_lines(st);
    let dda_footer = resolve_dda_footer_text(st);
    let dda_title = resolve_dda_publication_title(st);
    let (dda_en, dda_year) = {
        let mut parts = dda_sub_line.split('\u{e006}');
        let en = parts.next().unwrap_or("").trim().to_string();
        let yr = parts.next().unwrap_or("").trim().to_string();
        (en, yr)
    };
    if is_brs {
        let _ = app.session.edit(|doc, _| {
            if let Some(pos) = doc.text_variables.iter().position(|v| v.name == "Title") {
                doc.text_variables[pos].kind = designcraft_doc::vars::VarKind::Custom { text: brs_title.clone() };
            } else {
                doc.text_variables
                    .push(designcraft_doc::vars::TextVariable::new("Title", designcraft_doc::vars::VarKind::Custom { text: brs_title.clone() }));
            }
            if let Some(pos) = doc.text_variables.iter().position(|v| v.name == "Release No") {
                doc.text_variables[pos].kind = designcraft_doc::vars::VarKind::Custom { text: brs_nopub.clone() };
            } else {
                doc.text_variables
                    .push(designcraft_doc::vars::TextVariable::new("Release No", designcraft_doc::vars::VarKind::Custom { text: brs_nopub.clone() }));
            }
            Ok(serde_json::Value::Null)
        });
    } else if is_dda {
        let _ = app.session.edit(|doc, _| {
            if let Some(pos) = doc.text_variables.iter().position(|v| v.name == "Title") {
                doc.text_variables[pos].kind = designcraft_doc::vars::VarKind::Custom { text: dda_title.clone() };
            } else {
                doc.text_variables
                    .push(designcraft_doc::vars::TextVariable::new("Title", designcraft_doc::vars::VarKind::Custom { text: dda_title.clone() }));
            }
            Ok(serde_json::Value::Null)
        });
    }

    let expand_vars = |text: &str, is_footer: bool| -> String {
        let mut s = text.to_string();
        if (s.is_empty() || s == "{page}") && is_general && is_footer {
            return general_footer.clone();
        }
        if (s.is_empty() || s == "{page}") && is_brs && is_footer {
            return format!("{brs_title}\n{brs_nopub}");
        }
        if s.contains("{title_publication}") {
            s = s.replace("{title_publication}", &general_footer);
        }
        if s.contains("{title}") {
            let t = if is_general {
                &general_footer
            } else if is_brs {
                &brs_title
            } else {
                &dda_title_line
            };
            s = s.replace("{title}", t);
        }
        if s.contains("{title_en}") {
            s = s.replace("{title_en}", &dda_en);
        }
        if s.contains("{year}") {
            s = s.replace("{year}", &dda_year);
        }
        if s.contains("{static_text}") {
            s = s.replace("{static_text}", &dda_footer);
        }
        if s.contains("{tab}") {
            s = s.replace("{tab}", "\u{e006}");
        }
        if s.contains("{running_header}") {
            s = s.replace("{running_header}", "{section}");
        }
        if s.contains("{nopub}") {
            s = s.replace("{nopub}", &brs_nopub);
        }
        if s.contains("{releaseNo}") {
            s = s.replace("{releaseNo}", &brs_nopub);
        }
        s
    };

    let header_single = if (is_brs || is_general) && (st.hf_header_single == "{title}\n{nopub}" || st.hf_header_single.is_empty()) {
        String::new()
    } else if is_dda && (st.hf_header_single.is_empty() || st.hf_header_single == "{running_header}" || st.hf_header_single == "{section}") {
        "{section}\u{e006}{page}".to_string()
    } else {
        expand_vars(&st.hf_header_single, false)
    };
    let header_left = if (is_brs || is_general) && (st.hf_header_left == "{title}\n{nopub}" || st.hf_header_left.is_empty()) {
        String::new()
    } else if is_dda && (st.hf_header_left.is_empty() || st.hf_header_left == "{running_header}" || st.hf_header_left == "{section}") {
        "{section}\u{e006}{page}".to_string()
    } else {
        expand_vars(&st.hf_header_left, false)
    };
    let header_right = if (is_brs || is_general) && (st.hf_header_right == "{title}\n{nopub}" || st.hf_header_right.is_empty()) {
        String::new()
    } else if is_dda
        && (st.hf_header_right.is_empty()
            || st.hf_header_right == "{tab}{running_header}"
            || st.hf_header_right == "\u{e006}{running_header}"
            || st.hf_header_right == "\u{e006}{section}"
            || st.hf_header_right == "{running_header}")
    {
        "{page}\u{e006}{section}".to_string()
    } else {
        expand_vars(&st.hf_header_right, false)
    };

    let footer_single = if is_dda && (st.hf_footer_single.is_empty() || st.hf_footer_single == "{page}") {
        dda_footer.clone()
    } else {
        expand_vars(&st.hf_footer_single, true)
    };
    let footer_left = if is_dda && (st.hf_footer_left.is_empty() || st.hf_footer_left == "{page}") {
        dda_footer.clone()
    } else {
        expand_vars(&st.hf_footer_left, true)
    };
    let footer_right =
        if is_dda && (st.hf_footer_right.is_empty() || st.hf_footer_right == "{page}") { dda_footer } else { expand_vars(&st.hf_footer_right, true) };

    let mut params = json!({
        "preset": st.hf_style,
        "sided": if st.hf_sided == 1 { "single" } else { "double" },
        "scope": scope,
        "headerOffset": st.hf_header_offset,
        "footerOffset": st.hf_footer_offset,
        "header": {
            "single": header_single,
            "left": header_left,
            "right": header_right,
        },
        "footer": {
            "single": footer_single,
            "left": footer_left,
            "right": footer_right,
        },
        "objects": {
            "headerRule": st.hf_header_rule,
            "headerRuleWeight": st.hf_header_rule_weight,
            "footerRule": st.hf_footer_rule,
            "footerRuleWeight": st.hf_footer_rule_weight,
            "topAccentBar": st.hf_top_accent_bar,
            "topAccentBarHeight": st.hf_top_accent_bar_height,
            "topAccentBarBleed": st.hf_top_accent_bar_bleed,
        },
        "colors": {
            "primary": st.hf_color_primary,
            "accent": st.hf_color_accent,
            "dark": st.hf_color_dark,
            "ruleColor": st.hf_color_rule,
            "textColor": st.hf_color_text,
        }
    });

    let doc_margins = app.session.active().map(|s| detect_document_margins(&s.doc));
    if let Some(m) = doc_margins
        && let Some(obj) = params.as_object_mut()
    {
        obj.insert(
            "margins".to_string(),
            json!({
                "top": m.top,
                "bottom": m.bottom,
                "inside": m.inside,
                "outside": m.outside,
            }),
        );
    }

    let mut applied = app.run("layout.headerFooter.apply", params.clone());
    if applied.is_err() && is_general {
        // The engine may not ship the `hf-bps-general` built-in preset yet: apply the same
        // configuration through the `custom` base, which the explicit header/footer, geometry and
        // color params above fully override anyway.
        if let Some(obj) = params.as_object_mut() {
            obj.insert("preset".to_string(), json!("custom"));
        }
        applied = app.run("layout.headerFooter.apply", params);
    }
    applied?;

    if is_dda {
        let _ = app.session.edit(|doc, _| {
            let total = doc.page_count();
            // Pages 0, 1, 2 (Cover, 2nd Cover, A02) always have [none]
            for p in 0..3.min(total) {
                if let Some((si, pi)) = doc.page_loc(p) {
                    let sp = std::sync::Arc::make_mut(&mut doc.spreads[si]);
                    sp.pages[pi].parent = None;
                }
            }
            // Back cover -> [none]
            if let Some(b) = super::covers::find_back_cover_page(doc) {
                if let Some((si, pi)) = doc.page_loc(b) {
                    let sp = std::sync::Arc::make_mut(&mut doc.spreads[si]);
                    sp.pages[pi].parent = None;
                }
            }
            // Inserted blank pages (0 margins and no items) -> [none]
            for p in 0..total {
                if let Some(pg) = doc.page(p) {
                    if pg.margins.top == 0.0 && pg.margins.bottom == 0.0 && !doc.page_has_items(p) {
                        if let Some((si, pi)) = doc.page_loc(p) {
                            let sp = std::sync::Arc::make_mut(&mut doc.spreads[si]);
                            sp.pages[pi].parent = None;
                        }
                    }
                }
            }
            Ok(serde_json::Value::Null)
        });
    }

    Ok(format!("Header & footer template '{}' applied successfully", st.hf_style))
}

pub fn execute_remove_header_footer(app: &mut DesignApp) -> Result<String, String> {
    app.run("layout.headerFooter.remove", json!({}))?;
    Ok("Header and footer removed from document".to_string())
}

pub fn execute_save_hf_template(app: &mut DesignApp, st: &mut PubgenState) -> Result<String, String> {
    let name = st.hf_custom_name.trim().to_string();
    if name.is_empty() {
        return Err("Please enter a template name".to_string());
    }
    let template = json!({
        "name": name,
        "description": "User custom template",
        "sided": if st.hf_sided == 1 { "single" } else { "double" },
        "headerOffset": st.hf_header_offset,
        "footerOffset": st.hf_footer_offset,
        "header": {
            "single": st.hf_header_single,
            "left": st.hf_header_left,
            "right": st.hf_header_right,
        },
        "footer": {
            "single": st.hf_footer_single,
            "left": st.hf_footer_left,
            "right": st.hf_footer_right,
        },
        "objects": {
            "headerRule": st.hf_header_rule,
            "headerRuleWeight": st.hf_header_rule_weight,
            "footerRule": st.hf_footer_rule,
            "footerRuleWeight": st.hf_footer_rule_weight,
            "topAccentBar": st.hf_top_accent_bar,
            "topAccentBarHeight": st.hf_top_accent_bar_height,
            "topAccentBarBleed": st.hf_top_accent_bar_bleed,
        },
        "colors": {
            "primary": st.hf_color_primary,
            "accent": st.hf_color_accent,
            "dark": st.hf_color_dark,
            "ruleColor": st.hf_color_rule,
            "textColor": st.hf_color_text,
        }
    });

    app.run("layout.headerFooter.saveTemplate", json!({ "template": template }))?;
    st.hf_style = name.to_string();
    st.hf_custom_name.clear();
    Ok(format!("Template '{name}' saved successfully"))
}

pub fn execute_delete_hf_template(app: &mut DesignApp, st: &mut PubgenState, name: &str) -> Result<String, String> {
    app.run("layout.headerFooter.deleteTemplate", json!({ "name": name }))?;
    st.hf_style = "Modern Minimal".to_string();
    Ok(format!("Template '{name}' deleted"))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_execute_apply_and_remove_header_footer() {
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({ "facingPages": true, "pages": 4 })).unwrap();

        let st = PubgenState {
            hf_style: "Corporate Report".into(),
            hf_sided: 0,
            hf_header_left: "{title}".into(),
            hf_header_right: "{section}".into(),
            hf_footer_left: "{page}".into(),
            hf_footer_right: "Confidential".into(),
            hf_top_accent_bar: true,
            hf_top_accent_bar_height: 5.0,
            hf_top_accent_bar_bleed: true,
            hf_footer_rule: true,
            hf_color_primary: "#0072bc".into(),
            ..Default::default()
        };

        let res = execute_apply_header_footer(&mut app, &st);
        assert!(res.is_ok(), "apply should succeed: {res:?}");

        let doc = &app.session.active().unwrap().doc;
        let hf = doc.header_footer.as_ref().expect("header_footer present");
        assert_eq!(hf.preset, "Corporate Report");
        assert!(hf.objects.top_accent_bar);
        assert!(hf.accent_bar_items.left.is_some());
        assert!(hf.accent_bar_items.right.is_some());

        // Now set custom page margins and verify header_footer parent spread is synchronized
        let set_m_res = execute_set_page_margins(&mut app);
        assert!(set_m_res.is_ok());

        let doc_with_m = &app.session.active().unwrap().doc;
        let hf_with_m = doc_with_m.header_footer.as_ref().unwrap();
        let parent_sp = doc_with_m.parents.iter().find(|p| Some(p.id) == hf_with_m.parent).unwrap();
        assert!((parent_sp.pages[0].margins.inside - 19.0 * 72.0 / 25.4).abs() < 1e-3);
        assert!((parent_sp.pages[0].margins.outside - 15.0 * 72.0 / 25.4).abs() < 1e-3);

        let rem_res = execute_remove_header_footer(&mut app);
        assert!(rem_res.is_ok());
        let doc_after = &app.session.active().unwrap().doc;
        assert!(doc_after.header_footer.is_none());
    }

    #[test]
    fn test_execute_apply_header_footer_scopes_to_range_and_section() {
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({ "pages": 11 })).unwrap();
        // A section starting at page 5: section 0 covers pages 1-4, section 1 pages 5-11.
        app.run("layout.section", json!({ "page": 5, "startNumber": 1, "style": "arabic" })).unwrap();

        // Range scope: the template's parent lands on pages 5-8 only.
        let st = PubgenState { hf_scope: 2, hf_scope_start: 5, hf_scope_end: 8, ..Default::default() };
        execute_apply_header_footer(&mut app, &st).unwrap_or_else(|e| panic!("range apply: {e}"));
        {
            let doc = &app.session.active().unwrap().doc;
            let hf = doc.header_footer.as_ref().expect("header_footer present");
            assert_eq!(hf.scope, designcraft_doc::HfScope::Range { first: 5, last: 8 });
            let parent = hf.parent;
            let on_page = |i: usize| doc.page(i).is_some_and(|p| p.parent == parent);
            assert!((4..=7).all(on_page), "pages 5-8 carry the header/footer parent");
            assert!((0..4).all(|i| !on_page(i)) && (8..11).all(|i| !on_page(i)), "pages outside the range do not");
        }

        // Section scope: re-apply scoped to section index 1 (pages 5-11).
        let st = PubgenState { hf_scope: 1, hf_scope_section: 1, ..Default::default() };
        execute_apply_header_footer(&mut app, &st).unwrap_or_else(|e| panic!("section apply: {e}"));
        {
            let doc = &app.session.active().unwrap().doc;
            let hf = doc.header_footer.as_ref().expect("header_footer present");
            assert_eq!(hf.scope, designcraft_doc::HfScope::Section(1));
            let parent = hf.parent;
            let on_page = |i: usize| doc.page(i).is_some_and(|p| p.parent == parent);
            assert!((4..11).all(on_page), "section pages carry the parent");
            assert!((0..4).all(|i| !on_page(i)), "pages before the section do not");
        }

        // Scope 3: This Page Only (page 6 only).
        let st = PubgenState { hf_scope: 3, hf_scope_start: 6, ..Default::default() };
        execute_apply_header_footer(&mut app, &st).unwrap_or_else(|e| panic!("this page only apply: {e}"));
        {
            let doc = &app.session.active().unwrap().doc;
            let hf = doc.header_footer.as_ref().expect("header_footer present");
            assert_eq!(hf.scope, designcraft_doc::HfScope::Range { first: 6, last: 6 });
            let parent = hf.parent;
            let on_page = |i: usize| doc.page(i).is_some_and(|p| p.parent == parent);
            assert!(on_page(5), "page 6 carries the parent");
            assert!((0..5).all(|i| !on_page(i)) && (6..11).all(|i| !on_page(i)), "other pages do not carry parent");
        }

        // Scope 4: This Page Forward (page 7 through end of doc).
        let st = PubgenState { hf_scope: 4, hf_scope_start: 7, ..Default::default() };
        execute_apply_header_footer(&mut app, &st).unwrap_or_else(|e| panic!("this page forward apply: {e}"));
        {
            let doc = &app.session.active().unwrap().doc;
            let hf = doc.header_footer.as_ref().expect("header_footer present");
            assert_eq!(hf.scope, designcraft_doc::HfScope::Range { first: 7, last: 11 });
            let parent = hf.parent;
            let on_page = |i: usize| doc.page(i).is_some_and(|p| p.parent == parent);
            assert!((6..11).all(on_page), "pages 7-11 carry the parent");
            assert!((0..6).all(|i| !on_page(i)), "pages before page 7 do not");
        }
    }

    #[test]
    fn test_execute_save_and_delete_hf_template() {
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({})).unwrap();

        let mut st = PubgenState {
            hf_custom_name: "Annual Audit 2026".into(),
            hf_sided: 1,
            hf_header_single: "Audit Report | {page}".into(),
            hf_footer_single: "Internal Use Only".into(),
            hf_header_rule: true,
            hf_color_primary: "#990000".into(),
            ..Default::default()
        };

        let save_res = execute_save_hf_template(&mut app, &mut st);
        assert!(save_res.is_ok(), "{:?}", save_res.err());
        assert_eq!(st.hf_style, "Annual Audit 2026");
        assert!(st.hf_custom_name.is_empty());

        let templates = designcraft_engine::cmd::header_footer::all_templates(&app.session);
        assert!(templates.iter().any(|t| t.get("name").and_then(Value::as_str) == Some("Annual Audit 2026")));

        let del_res = execute_delete_hf_template(&mut app, &mut st, "Annual Audit 2026");
        assert!(del_res.is_ok());
        assert_eq!(st.hf_style, "Modern Minimal");
    }

    #[test]
    fn test_load_hf_template_to_state() {
        let mut st = PubgenState::default();
        let tpl = json!({
            "name": "Custom Editorial",
            "sided": "single",
            "headerOffset": 30.0,
            "footerOffset": 32.0,
            "header": { "single": "Chapter | {page}" },
            "footer": { "single": "Footer Notes" },
            "objects": {
                "headerRule": true,
                "headerRuleWeight": 1.25,
                "footerRule": false,
                "topAccentBar": true,
                "topAccentBarHeight": 8.0,
                "topAccentBarBleed": false
            },
            "colors": {
                "primary": "#334455",
                "accent": "#ffaa00",
                "dark": "#112233",
                "ruleColor": "#888888",
                "textColor": "#222222"
            }
        });

        load_hf_template_to_state(&mut st, &tpl);
        assert_eq!(st.hf_style, "Custom Editorial");
        assert_eq!(st.hf_sided, 1);
        assert_eq!(st.hf_header_offset, 30.0);
        assert_eq!(st.hf_footer_offset, 32.0);
        assert_eq!(st.hf_header_single, "Chapter | {page}");
        assert_eq!(st.hf_footer_single, "Footer Notes");
        assert!(st.hf_header_rule);
        assert_eq!(st.hf_header_rule_weight, 1.25);
        assert!(!st.hf_footer_rule);
        assert!(st.hf_top_accent_bar);
        assert_eq!(st.hf_top_accent_bar_height, 8.0);
        assert!(!st.hf_top_accent_bar_bleed);
        assert_eq!(st.hf_color_primary, "#334455");
        assert_eq!(st.hf_color_accent, "#ffaa00");
    }

    #[test]
    fn test_resolve_brs_header_texts() {
        let mut st = PubgenState::default();
        st.cover_title.clear();
        let (def_title, def_nopub) = resolve_brs_header_texts(&st);
        assert_eq!(def_title, "Perkembangan Nilai Tukar Petani");
        assert_eq!(def_nopub, "BRS No. 61/10/32/Th. XXVIII, 1 Oktober 2026");

        st.excel_cover = Some(super::excel::ExcelCoverData {
            title1: "Perkembangan Nilai Tukar Petani Jawa Barat".into(),
            release_month: "September 2026".into(),
            release_no: "61/10/32/Th. XXVIII".into(),
            release_date: "1 Oktober 2026".into(),
            ..Default::default()
        });

        let (title, nopub) = resolve_brs_header_texts(&st);
        assert_eq!(title, "Perkembangan Nilai Tukar Petani Jawa Barat, September 2026");
        assert_eq!(nopub, "BRS No. 61/10/32/Th. XXVIII, 1 Oktober 2026");
    }

    #[test]
    fn test_resolve_general_report_footer_text() {
        let mut st = PubgenState::default();
        st.cover_title.clear();
        assert_eq!(resolve_general_report_footer_text(&st), "");

        st.excel_cover = Some(super::excel::ExcelCoverData { title1: "TITLE ONE".into(), ..Default::default() });
        assert_eq!(resolve_general_report_footer_text(&st), "TITLE ONE");

        st.excel_cover = Some(super::excel::ExcelCoverData {
            title_publication: "Statistik Kesejahteraan Rakyat 2026".into(),
            title1: "OTHER TITLE".into(),
            ..Default::default()
        });
        assert_eq!(resolve_general_report_footer_text(&st), "Statistik Kesejahteraan Rakyat 2026");

        st.excel_cover = None;
        st.cover_title = "  Cover Title\nSecond line  ".into();
        assert_eq!(resolve_general_report_footer_text(&st), "Cover Title");
    }

    #[test]
    fn test_load_hf_template_to_state_general_report() {
        let mut st = PubgenState::default();
        st.excel_cover = Some(super::excel::ExcelCoverData { title_publication: "Statistik Kesejahteraan Rakyat 2026".into(), ..Default::default() });
        load_hf_template_to_state(&mut st, &json!({ "name": "hf-bps-general" }));

        assert_eq!(st.hf_style, "hf-bps-general");
        assert_eq!(st.hf_sided, 0);
        assert_eq!(st.hf_header_offset, 24.0);
        assert_eq!(st.hf_footer_offset, 24.0);
        assert!(st.hf_header_single.is_empty() && st.hf_header_left.is_empty() && st.hf_header_right.is_empty(), "no running header");
        assert_eq!(st.hf_footer_single, "Statistik Kesejahteraan Rakyat 2026");
        assert_eq!(st.hf_footer_left, "Statistik Kesejahteraan Rakyat 2026");
        assert_eq!(st.hf_footer_right, "Statistik Kesejahteraan Rakyat 2026");
        assert!(!st.hf_header_rule && !st.hf_footer_rule && !st.hf_top_accent_bar);
        assert_eq!(st.hf_color_primary, "#000000");
        assert_eq!(st.hf_color_text, "#000000");

        // The short alias is accepted too.
        let mut alias = PubgenState::default();
        alias.cover_title.clear();
        load_hf_template_to_state(&mut alias, &json!({ "name": "bps-general" }));
        assert_eq!(alias.hf_style, "bps-general");
        assert_eq!(alias.hf_footer_single, "{title_publication}", "no cover title yet: keep the token");
    }

    #[test]
    fn test_execute_apply_header_footer_general_report_black_footer() {
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({ "facingPages": false, "pages": 4 })).unwrap();

        let mut st = PubgenState {
            hf_style: "hf-bps-general".into(),
            hf_sided: 0,
            hf_scope: 0, // All pages -> body pages only, cover and back cover stay clean
            hf_color_primary: "#000000".into(),
            hf_color_text: "#000000".into(),
            ..Default::default()
        };
        st.excel_cover = Some(super::excel::ExcelCoverData { title_publication: "Statistik Kesejahteraan Rakyat 2026".into(), ..Default::default() });
        load_hf_template_to_state(&mut st, &json!({ "name": "hf-bps-general" }));

        let res = execute_apply_header_footer(&mut app, &st);
        assert!(res.is_ok(), "apply hf-bps-general should succeed: {res:?}");

        let doc = &app.session.active().unwrap().doc;
        assert!(doc.settings.facing_pages, "double-sided footers need facing pages");
        let hf = doc.header_footer.as_ref().expect("header_footer present");
        assert_eq!(hf.scope, designcraft_doc::HfScope::Range { first: 2, last: 4 }, "cover and back cover are skipped");
        assert_eq!(hf.colors.text_color, "#000000", "footer text is black");
        assert!(hf.header_items.left.is_none() && hf.header_items.right.is_none(), "no running header");

        for id in [hf.footer_items.left, hf.footer_items.right].into_iter().flatten() {
            let text = doc.item(id).and_then(|it| it.text_frame()).and_then(|tf| doc.story(tf.story)).map(|s| s.text.clone());
            assert_eq!(text.as_deref(), Some("Statistik Kesejahteraan Rakyat 2026"), "the footer prints the COVER/COV publication title");
        }
    }

    #[test]
    fn test_execute_apply_hf_brs_facing_pages_and_scope() {
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        // Initially single pages (facingPages: false) with 4 pages
        app.run("file.new", json!({ "facingPages": false, "pages": 4 })).unwrap();
        assert!(!app.session.active().unwrap().doc.settings.facing_pages);

        let mut st = PubgenState {
            hf_style: "hf-brs".into(),
            hf_sided: 0,
            hf_scope: 0, // All Pages -> should auto-scope to body pages [2, 4]
            ..Default::default()
        };
        st.excel_cover = Some(super::excel::ExcelCoverData {
            title1: "Perkembangan Nilai Tukar Petani Jawa Barat".into(),
            release_month: "September 2026".into(),
            release_no: "61/10/32/Th. XXVIII".into(),
            release_date: "1 Oktober 2026".into(),
            ..Default::default()
        });

        let res = execute_apply_header_footer(&mut app, &st);
        assert!(res.is_ok(), "apply hf-brs should succeed: {res:?}");

        let doc = &app.session.active().unwrap().doc;
        // Verify facing_pages was automatically enabled
        assert!(doc.settings.facing_pages, "facing pages should be enabled for double-sided BRS");

        let hf = doc.header_footer.as_ref().expect("header_footer present");
        assert_eq!(hf.preset, "hf-brs");
        assert_eq!(hf.sided, designcraft_doc::Sided::Double);
        assert_eq!(hf.scope, designcraft_doc::HfScope::Range { first: 2, last: 4 });

        // Header is blank
        assert!(hf.header_items.left.is_none(), "left header is blank");
        assert!(hf.header_items.right.is_none(), "right header is blank");

        // Footer text and folio items are generated for verso and recto
        assert!(hf.footer_items.left.is_some(), "left footer text item exists");
        assert!(hf.footer_items.right.is_some(), "right footer text item exists");
        assert!(hf.folio_items.left.is_some(), "left folio item exists");
        assert!(hf.folio_items.right.is_some(), "right folio item exists");
    }

    #[test]
    fn test_resolve_dda_footer_lines_no_duplication() {
        let mut st = PubgenState::default();
        st.excel_cover = Some(super::excel::ExcelCoverData {
            title1: "KECAMATAN KARAWANG TIMUR DALAM ANGKA".into(),
            title2: "EAST KARAWANG DISTRICT IN FIGURES".into(),
            title_en: "EAST KARAWANG DISTRICT IN FIGURES".into(),
            year: "2026".into(),
            ..Default::default()
        });

        let (line1, line2) = resolve_dda_footer_lines(&st);
        assert_eq!(line1, "KECAMATAN KARAWANG TIMUR DALAM ANGKA");
        assert_eq!(line2, "EAST KARAWANG DISTRICT IN FIGURES\u{e006}2026");

        let pub_title = resolve_dda_publication_title(&st);
        assert_eq!(pub_title, "KECAMATAN KARAWANG TIMUR DALAM ANGKA / EAST KARAWANG DISTRICT IN FIGURES");

        let full_footer = resolve_dda_footer_text(&st);
        assert_eq!(full_footer, "KECAMATAN KARAWANG TIMUR DALAM ANGKA\nEAST KARAWANG DISTRICT IN FIGURES\u{e006}2026");
    }

    #[test]
    fn test_execute_apply_header_footer_hf_dda_preset() {
        let mut app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({ "pages": 6, "facingPages": true })).unwrap();
        app.run("layout.section", json!({ "page": 1, "marker": "BAB 01 Geografi / Chapter 01 Geography" })).unwrap();

        let mut st = PubgenState {
            hf_style: "hf-dda".into(),
            hf_sided: 0,
            hf_scope: 0,
            cover_title: "Kabupaten Pasuruan Dalam Angka 2026".into(),
            cover_title2: String::new(),
            cover_title_en: "Pasuruan Regency in Figures 2026".into(),
            cover_year: "2026".into(),
            ..Default::default()
        };
        load_hf_template_to_state(&mut st, &json!({ "name": "hf-dda" }));

        let res = execute_apply_header_footer(&mut app, &st);
        assert!(res.is_ok(), "apply hf-dda should succeed: {res:?}");

        let doc = &app.session.active().unwrap().doc;
        let hf = doc.header_footer.as_ref().expect("header_footer present");
        assert_eq!(hf.preset, "hf-dda");
        assert_eq!(hf.sided, designcraft_doc::Sided::Double);

        // Verify "Title" variable was populated with bilingual title
        let title_var = doc.text_variables.iter().find(|v| v.name.eq_ignore_ascii_case("Title"));
        assert!(title_var.is_some(), "Title variable should be created/updated");
        match &title_var.unwrap().kind {
            designcraft_doc::vars::VarKind::Custom { text } => {
                assert_eq!(text, "Kabupaten Pasuruan Dalam Angka 2026 / Pasuruan Regency in Figures 2026");
            }
            other => panic!("expected Custom VarKind, got {other:?}"),
        }

        // Verify parent spread was created and has 2 pages
        let parent_sp = doc.parents.iter().find(|p| Some(p.id) == hf.parent).expect("parent spread exists");
        assert_eq!(parent_sp.pages.len(), 2, "facing parent spread has 2 pages");

        // Verify pages 0, 1, 2 (Cover, 2nd Cover, A02) have parent == None
        for p in 0..3 {
            assert_eq!(doc.page(p).unwrap().parent, None, "Page {p} must have parent [none]");
        }
        // Verify pages 3..6 have the header/footer parent
        for p in 3..6 {
            assert_eq!(doc.page(p).unwrap().parent, hf.parent, "Page {p} must have header/footer parent");
        }
    }

    #[test]
    fn test_all_templates_includes_hf_brs_and_hf_dda_regardless_of_profile() {
        let app = DesignApp::new(designcraft_engine::Session::new(), crate::Services::default());
        let templates = designcraft_engine::cmd::header_footer::all_templates(&app.session);
        assert!(templates.iter().any(|t| t.get("name").and_then(Value::as_str) == Some("hf-brs")), "hf-brs template must be available");
        assert!(templates.iter().any(|t| t.get("name").and_then(Value::as_str) == Some("hf-dda")), "hf-dda template must be available");
    }
}
