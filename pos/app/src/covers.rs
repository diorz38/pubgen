use serde::{Deserialize, Serialize};

use crate::metadata_form::Metadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CoverStyle {
    #[default]
    Horizon,
    Modern,
    Tricolor,
    Minimal,
}

impl CoverStyle {
    pub const ALL: [CoverStyle; 4] = [
        CoverStyle::Horizon,
        CoverStyle::Modern,
        CoverStyle::Tricolor,
        CoverStyle::Minimal,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Horizon => "Horizon (Top Banner)",
            Self::Modern => "Modern (Asymmetric Split)",
            Self::Tricolor => "Tricolor (BPS 3-Band)",
            Self::Minimal => "Minimal (Editorial Clean)",
        }
    }
}

impl std::fmt::Display for CoverStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Generates a standalone SVG cover based on the publication metadata and chosen style.
pub fn render_cover_svg(meta: &Metadata, style: CoverStyle, width: f64, height: f64) -> String {
    let title_id = if meta.title.trim().is_empty() {
        "KABUPATEN DALAM ANGKA"
    } else {
        meta.title.trim()
    };
    let title_en = if meta.title_en.trim().is_empty() {
        "REGENCY IN FIGURES"
    } else {
        meta.title_en.trim()
    };
    let year = if meta.date.trim().is_empty() {
        "2026"
    } else {
        meta.date.trim()
    };
    let agency = if meta.agency.trim().is_empty() {
        "BADAN PUSAT STATISTIK"
    } else {
        meta.agency.trim()
    };
    let catalogue = &meta.catalogue;
    let code = &meta.code;

    let mut svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {width} {height}' width='{width}' height='{height}'>\n"
    );

    match style {
        CoverStyle::Horizon => {
            let banner_h = height * 0.45;
            let bar_y = banner_h - 6.0;
            let agency_y = banner_h + 36.0;
            let title1_y = banner_h + 84.0;
            let title2_y = banner_h + 114.0;
            let year_y = banner_h + 174.0;
            let line_y = height - 60.0;
            let line_w = width - 40.0;
            let meta_y = height - 40.0;

            svg.push_str(&format!(
                "  <rect width='100%' height='100%' fill='#F8FAFC' />\n  <rect x='0' y='0' width='{width}' height='{banner_h}' fill='#0072BC' />\n  <rect x='0' y='{bar_y}' width='{width}' height='6' fill='#F5871F' />\n  <text x='40' y='{agency_y}' font-family='sans-serif' font-size='12' font-weight='bold' fill='#0072BC'>{}</text>\n  <text x='40' y='{title1_y}' font-family='sans-serif' font-size='28' font-weight='900' fill='#0F172A'>{}</text>\n  <text x='40' y='{title2_y}' font-family='sans-serif' font-size='18' font-weight='600' fill='#475569' font-style='italic'>{}</text>\n  <text x='40' y='{year_y}' font-family='sans-serif' font-size='44' font-weight='bold' fill='#F5871F'>{}</text>\n  <line x1='40' y1='{line_y}' x2='{line_w}' y2='{line_y}' stroke='#E2E8F0' stroke-width='1.5' />\n  <text x='40' y='{meta_y}' font-family='sans-serif' font-size='10' fill='#64748B'>Katalog: {catalogue} | No. Publikasi: {code}</text>\n",
                html_escape(agency),
                html_escape(title_id),
                html_escape(title_en),
                html_escape(year)
            ));
        }
        CoverStyle::Modern => {
            let split_y = height * 0.55;
            let p1 = split_y - 40.0;
            let p2 = split_y + 40.0;
            let p3 = split_y - 30.0;
            let p4 = split_y + 50.0;
            let year_x = width - 40.0;
            let year_y = height - 90.0;
            let meta_y = height - 40.0;

            svg.push_str(&format!(
                "  <rect width='100%' height='100%' fill='#FFFFFF' />\n  <polygon points='0,0 {width},0 {width},{p1} 0,{p2}' fill='#0A2540' />\n  <polygon points='0,{p2} {width},{p1} {width},{p3} 0,{p4}' fill='#00A859' opacity='0.85' />\n  <text x='40' y='60' font-family='sans-serif' font-size='11' font-weight='bold' fill='#94A3B8' letter-spacing='1'>{}</text>\n  <text x='40' y='110' font-family='sans-serif' font-size='32' font-weight='900' fill='#FFFFFF'>{}</text>\n  <text x='40' y='145' font-family='sans-serif' font-size='19' font-weight='normal' fill='#E2E8F0' font-style='italic'>{}</text>\n  <text x='{year_x}' y='{year_y}' font-family='sans-serif' font-size='56' font-weight='900' fill='#0A2540' text-anchor='end'>{}</text>\n  <text x='40' y='{meta_y}' font-family='sans-serif' font-size='10' fill='#64748B'>Katalog: {catalogue} | No. Publikasi: {code}</text>\n",
                html_escape(agency),
                html_escape(title_id),
                html_escape(title_en),
                html_escape(year)
            ));
        }
        CoverStyle::Tricolor => {
            let h1 = height * 0.15;
            let h2 = height * 0.40;
            let h3 = height * 0.45;
            let band3_y = h1 + h2;
            let agency_y = h1 * 0.6;
            let t1_y = h1 + 60.0;
            let t2_y = h1 + 95.0;
            let yr_y = h1 + 145.0;
            let meta_y = height - 40.0;

            svg.push_str(&format!(
                "  <rect x='0' y='0' width='{width}' height='{h1}' fill='#0072BC' />\n  <rect x='0' y='{h1}' width='{width}' height='{h2}' fill='#F8FAFC' />\n  <rect x='0' y='{band3_y}' width='{width}' height='{h3}' fill='#0F172A' />\n  <text x='40' y='{agency_y}' font-family='sans-serif' font-size='12' font-weight='bold' fill='#FFFFFF'>{}</text>\n  <text x='40' y='{t1_y}' font-family='sans-serif' font-size='28' font-weight='900' fill='#0072BC'>{}</text>\n  <text x='40' y='{t2_y}' font-family='sans-serif' font-size='18' font-weight='600' fill='#F5871F' font-style='italic'>{}</text>\n  <text x='40' y='{yr_y}' font-family='sans-serif' font-size='36' font-weight='bold' fill='#00A859'>{}</text>\n  <text x='40' y='{meta_y}' font-family='sans-serif' font-size='11' fill='#94A3B8'>Katalog: {catalogue} | Publikasi: {code}</text>\n",
                html_escape(agency),
                html_escape(title_id),
                html_escape(title_en),
                html_escape(year)
            ));
        }
        CoverStyle::Minimal => {
            let border_inset = 28.0;
            let frame_w = width - border_inset * 2.0;
            let frame_h = height - border_inset * 2.0;
            let meta_y = height - 60.0;

            svg.push_str(&format!(
                "  <rect width='100%' height='100%' fill='#FFFFFF' />\n  <rect x='{border_inset}' y='{border_inset}' width='{frame_w}' height='{frame_h}' fill='none' stroke='#0072BC' stroke-width='2' />\n  <text x='60' y='70' font-family='sans-serif' font-size='11' font-weight='bold' fill='#64748B' letter-spacing='1.5'>{}</text>\n  <line x1='60' y1='85' x2='160' y2='85' stroke='#F5871F' stroke-width='3' />\n  <text x='60' y='160' font-family='serif' font-size='30' font-weight='bold' fill='#0F172A'>{}</text>\n  <text x='60' y='195' font-family='serif' font-size='18' font-weight='normal' fill='#475569' font-style='italic'>{}</text>\n  <text x='60' y='270' font-family='sans-serif' font-size='48' font-weight='bold' fill='#0072BC'>{}</text>\n  <text x='60' y='{meta_y}' font-family='sans-serif' font-size='10' fill='#94A3B8'>Katalog BPS: {catalogue} | ISSN/ISBN: {code}</text>\n",
                html_escape(agency),
                html_escape(title_id),
                html_escape(title_en),
                html_escape(year)
            ));
        }
    }

    svg.push_str("</svg>\n");
    svg
}

/// Generates a structured prompt suitable for text-to-image AI generators (e.g. Cloudflare AI Flux / SDXL).
pub fn generate_ai_cover_prompt(meta: &Metadata, theme: &str) -> String {
    let region = if !meta.title.trim().is_empty() {
        meta.title.trim()
    } else {
        "Indonesian regional development"
    };
    let year = if !meta.date.trim().is_empty() {
        meta.date.trim()
    } else {
        "2026"
    };

    let theme_desc = match theme.to_lowercase().as_str() {
        "agriculture" | "pertanian" => {
            "lush vibrant terraced rice fields in Indonesia with golden hour morning mist, farmers harvesting crops, modern agricultural machinery, crystal clear atmosphere"
        }
        "industry" | "industri" => {
            "modern smart manufacturing factory in Indonesia, high-tech robotics, clean industrial automation, dynamic dramatic architectural perspective"
        }
        "maritime" | "maritim" | "perikanan" => {
            "bustling Indonesian modern seaport, vibrant fishing vessels, azure tropical ocean waters, cargo container ships under clear tropical skies"
        }
        "urban" | "perkotaan" => {
            "futuristic smart city skyline in Indonesia, sustainable green skyscrapers, monorail public transit, aerial drone perspective, clean aesthetic"
        }
        _ => {
            "majestic panoramic landscape of Indonesian countryside and thriving modern regional infrastructure, high vantage point, vibrant natural colors"
        }
    };

    format!(
        "High quality editorial publication cover photograph for '{region} ({year})': {theme_desc}. Professional photography, 8k resolution, Hasselblad medium format look, rich color grading, cinematic natural lighting, negative space at the top and bottom for book typography, no text, no letters, masterpiece."
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_cover_svg_horizon() {
        let mut meta = Metadata::default();
        meta.title = "Kecamatan Karawang Timur Dalam Angka".into();
        meta.title_en = "East Karawang District in Figures".into();
        meta.date = "2026".into();
        meta.agency = "BPS Kabupaten Karawang".into();

        let svg = render_cover_svg(&meta, CoverStyle::Horizon, 600.0, 850.0);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Kecamatan Karawang Timur"));
        assert!(svg.contains("East Karawang District"));
        assert!(svg.contains("2026"));
        assert!(svg.contains("BPS Kabupaten Karawang"));
        assert!(svg.ends_with("</svg>\n"));
    }

    #[test]
    fn test_render_cover_svg_modern() {
        let mut meta = Metadata::default();
        meta.title = "Berita Resmi Statistik".into();
        meta.date = "2026".into();

        let svg = render_cover_svg(&meta, CoverStyle::Modern, 600.0, 850.0);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Berita Resmi Statistik"));
        assert!(svg.contains("2026"));
    }

    #[test]
    fn test_generate_ai_prompt() {
        let mut meta = Metadata::default();
        meta.title = "Kabupaten Karawang Dalam Angka".into();
        meta.date = "2026".into();

        let prompt = generate_ai_cover_prompt(&meta, "agriculture");
        assert!(prompt.contains("Kabupaten Karawang"));
        assert!(prompt.contains("terraced rice fields"));
        assert!(prompt.contains("no text"));
    }
}
