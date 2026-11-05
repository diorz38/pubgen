use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::i18n::UiLang;

/// Standard BPS publication palette for statistical charts.
pub const BPS_CHART_PALETTE: &[&str] = &[
    "#0072BC", // BPS Primary Blue
    "#F5871F", // BPS Accent Orange
    "#00A859", // Leaf Green
    "#0088D2", // Sky Blue
    "#E53935", // Crimson Red
    "#8E24AA", // Purple
    "#3949AB", // Indigo
    "#FDD835", // Amber
    "#00897B", // Teal
    "#6D4C41", // Brown
];

/// Standard BPS BRS line chart palette (e.g. IT, IB, NTP).
pub const BPS_BRS_LINE_PALETTE: &[&str] = &[
    "#C00000", // IT: Crimson Red
    "#002060", // IB: Dark Navy
    "#418AB3", // NTP: Steel Blue
    "#0072BC", "#F5871F",
];

pub fn series_color(idx: usize) -> &'static str {
    BPS_CHART_PALETTE[idx % BPS_CHART_PALETTE.len()]
}

pub fn brs_series_color(idx: usize) -> &'static str {
    BPS_BRS_LINE_PALETTE[idx % BPS_BRS_LINE_PALETTE.len()]
}

/// Preset map color gradients: (name, min_hex, max_hex).
pub const MAP_COLOR_PRESETS: &[(&str, &str, &str)] = &[
    ("BPS Blue (Default)", "#E1F5FE", "#0072BC"),
    ("Flame Amber", "#FFF8E1", "#F5871F"),
    ("Emerald Green", "#E8F5E9", "#00A859"),
    ("Crimson Red", "#FFEBEE", "#E53935"),
    ("Purple Orchid", "#F3E5F5", "#8E24AA"),
    ("Teal Cyan", "#E0F2F1", "#00897B"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ChartType {
    #[default]
    BarClustered,
    HorizontalBar,
    Line,
    Donut,
    Map,
}

impl ChartType {
    pub const ALL: [ChartType; 5] = [
        ChartType::BarClustered,
        ChartType::HorizontalBar,
        ChartType::Line,
        ChartType::Donut,
        ChartType::Map,
    ];

    /// Localized display label for the UI.
    pub fn label(self, lang: UiLang) -> &'static str {
        crate::i18n::tr(
            lang,
            match self {
                Self::BarClustered => "chart_bar_clustered",
                Self::HorizontalBar => "chart_bar_horizontal",
                Self::Line => "chart_line",
                Self::Donut => "chart_donut",
                Self::Map => "chart_map",
            },
        )
    }

    /// Language-neutral label for logs and persisted extras.
    pub fn label_en(self) -> &'static str {
        match self {
            Self::BarClustered => "Bar Chart (Clustered)",
            Self::HorizontalBar => "Horizontal Bar Chart",
            Self::Line => "Line Chart",
            Self::Donut => "Donut / Pie Chart",
            Self::Map => "Choropleth Map",
        }
    }
}

impl std::fmt::Display for ChartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label_en())
    }
}

/// Helper to parse hex string `#RRGGBB` into `(r, g, b)`.
pub fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 114, 188) // default BPS blue
    }
}

/// Linearly interpolates between two colors: t in `0.0..=1.0`.
pub fn lerp_color(min_hex: &str, max_hex: &str, t: f64) -> String {
    let t = t.clamp(0.0, 1.0);
    let (r1, g1, b1) = parse_hex_color(min_hex);
    let (r2, g2, b2) = parse_hex_color(max_hex);
    let r = (r1 as f64 + (r2 as f64 - r1 as f64) * t).round() as u8;
    let g = (g1 as f64 + (g2 as f64 - g1 as f64) * t).round() as u8;
    let b = (b1 as f64 + (b2 as f64 - b1 as f64) * t).round() as u8;
    format!("#{r:02X}{g:02X}{b:02X}")
}

// ---------------------------------------------------------------------------
// Chart SVG Generators
// ---------------------------------------------------------------------------

pub fn render_bar_chart_svg(
    title: &str,
    categories: &[String],
    series_names: &[String],
    data: &[Vec<f64>], // [series_idx][category_idx]
    width: f64,
    height: f64,
) -> String {
    let pad_left = 60.0;
    let pad_right = 30.0;
    let pad_top = 40.0;
    let pad_bottom = 60.0;

    let chart_w = (width - pad_left - pad_right).max(10.0);
    let chart_h = (height - pad_top - pad_bottom).max(10.0);

    let max_val = data
        .iter()
        .flat_map(|s| s.iter().copied())
        .fold(0.0f64, f64::max);
    let upper_bound = if max_val <= 0.0 { 100.0 } else { max_val * 1.15 };

    let mut svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {width} {height}' width='{width}' height='{height}'>\n  <rect width='100%' height='100%' fill='#FFFFFF' />\n  <text x='{pad_left}' y='24' font-family='sans-serif' font-size='14' font-weight='bold' fill='#212121'>{}</text>\n",
        html_escape(title)
    );

    // Grid lines (4 ticks)
    for i in 0..=4 {
        let frac = i as f64 / 4.0;
        let y = pad_top + chart_h * (1.0 - frac);
        let val = upper_bound * frac;
        let x2 = pad_left + chart_w;
        let text_x = pad_left - 8.0;
        let text_y = y + 3.0;
        svg.push_str(&format!(
            "  <line x1='{pad_left}' y1='{y:.1}' x2='{x2:.1}' y2='{y:.1}' stroke='#E0E0E0' stroke-width='1' />\n  <text x='{text_x:.1}' y='{text_y:.1}' font-family='sans-serif' font-size='10' fill='#757575' text-anchor='end'>{val:.0}</text>\n"
        ));
    }

    let n_cats = categories.len().max(1);
    let n_series = series_names.len().max(1);
    let cat_step = chart_w / n_cats as f64;
    let bar_group_w = cat_step * 0.7;
    let bar_w = bar_group_w / n_series as f64;

    for (c_idx, cat) in categories.iter().enumerate() {
        let group_x = pad_left + c_idx as f64 * cat_step + (cat_step - bar_group_w) * 0.5;

        for (s_idx, s_data) in data.iter().enumerate() {
            let val = s_data.get(c_idx).copied().unwrap_or(0.0);
            let bar_h = (val / upper_bound * chart_h).max(0.0);
            let bx = group_x + s_idx as f64 * bar_w;
            let by = pad_top + chart_h - bar_h;
            let col = series_color(s_idx);
            let bw = bar_w - 2.0;

            svg.push_str(&format!(
                "  <rect x='{bx:.1}' y='{by:.1}' width='{bw:.1}' height='{bar_h:.1}' fill='{col}' rx='2' />\n"
            ));
            if bar_h > 12.0 {
                let lbl_x = bx + bw * 0.5;
                let lbl_y = by - 3.0;
                svg.push_str(&format!(
                    "  <text x='{lbl_x:.1}' y='{lbl_y:.1}' font-family='sans-serif' font-size='9' fill='#424242' text-anchor='middle'>{val:.0}</text>\n"
                ));
            }
        }

        let cat_x = group_x + bar_group_w * 0.5;
        let cat_y = pad_top + chart_h + 18.0;
        svg.push_str(&format!(
            "  <text x='{cat_x:.1}' y='{cat_y:.1}' font-family='sans-serif' font-size='10' fill='#424242' text-anchor='middle'>{}</text>\n",
            html_escape(cat)
        ));
    }

    // Legend
    let mut leg_x = pad_left;
    let leg_y = height - 15.0;
    for (s_idx, s_name) in series_names.iter().enumerate() {
        let col = series_color(s_idx);
        let rect_y = leg_y - 8.0;
        let text_x = leg_x + 14.0;
        svg.push_str(&format!(
            "  <rect x='{leg_x:.1}' y='{rect_y:.1}' width='10' height='10' fill='{col}' rx='2' />\n  <text x='{text_x:.1}' y='{leg_y:.1}' font-family='sans-serif' font-size='10' fill='#424242'>{}</text>\n",
            html_escape(s_name)
        ));
        leg_x += (s_name.len() as f64 * 7.0) + 32.0;
    }

    svg.push_str("</svg>\n");
    svg
}

pub fn render_line_chart_svg(
    title: &str,
    categories: &[String],
    series_names: &[String],
    data: &[Vec<f64>],
    width: f64,
    height: f64,
) -> String {
    let pad_left = 60.0;
    let pad_right = 30.0;
    let pad_top = 40.0;
    let pad_bottom = 60.0;

    let chart_w = (width - pad_left - pad_right).max(10.0);
    let chart_h = (height - pad_top - pad_bottom).max(10.0);

    let max_val = data
        .iter()
        .flat_map(|s| s.iter().copied())
        .fold(0.0f64, f64::max);
    let upper_bound = if max_val <= 0.0 { 100.0 } else { max_val * 1.15 };

    let mut svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {width} {height}' width='{width}' height='{height}'>\n  <rect width='100%' height='100%' fill='#FFFFFF' />\n  <text x='{pad_left}' y='24' font-family='sans-serif' font-size='14' font-weight='bold' fill='#212121'>{}</text>\n",
        html_escape(title)
    );

    // Grid lines
    for i in 0..=4 {
        let frac = i as f64 / 4.0;
        let y = pad_top + chart_h * (1.0 - frac);
        let val = upper_bound * frac;
        let x2 = pad_left + chart_w;
        let text_x = pad_left - 8.0;
        let text_y = y + 3.0;
        svg.push_str(&format!(
            "  <line x1='{pad_left}' y1='{y:.1}' x2='{x2:.1}' y2='{y:.1}' stroke='#E0E0E0' stroke-width='1' />\n  <text x='{text_x:.1}' y='{text_y:.1}' font-family='sans-serif' font-size='10' fill='#757575' text-anchor='end'>{val:.0}</text>\n"
        ));
    }

    let n_cats = categories.len().max(1);
    let x_step = if n_cats > 1 {
        chart_w / (n_cats - 1) as f64
    } else {
        chart_w
    };

    // Plot series polylines
    for (s_idx, s_data) in data.iter().enumerate() {
        let col = brs_series_color(s_idx);
        let mut pts = Vec::new();

        for (c_idx, &val) in s_data.iter().enumerate() {
            let px = pad_left + c_idx as f64 * x_step;
            let py = pad_top + chart_h - (val / upper_bound * chart_h);
            pts.push((px, py));
        }

        if !pts.is_empty() {
            let poly_str: Vec<String> = pts.iter().map(|(x, y)| format!("{x:.1},{y:.1}")).collect();
            svg.push_str(&format!(
                "  <polyline points='{}' fill='none' stroke='{col}' stroke-width='2.5' stroke-linejoin='round' />\n",
                poly_str.join(" ")
            ));
            for (px, py) in pts {
                svg.push_str(&format!(
                    "  <circle cx='{px:.1}' cy='{py:.1}' r='3.5' fill='{col}' stroke='#FFFFFF' stroke-width='1' />\n"
                ));
            }
        }
    }

    // X axis category labels
    for (c_idx, cat) in categories.iter().enumerate() {
        let px = pad_left + c_idx as f64 * x_step;
        let py = pad_top + chart_h + 18.0;
        svg.push_str(&format!(
            "  <text x='{px:.1}' y='{py:.1}' font-family='sans-serif' font-size='10' fill='#424242' text-anchor='middle'>{}</text>\n",
            html_escape(cat)
        ));
    }

    // Legend
    let mut leg_x = pad_left;
    let leg_y = height - 15.0;
    for (s_idx, s_name) in series_names.iter().enumerate() {
        let col = brs_series_color(s_idx);
        let line_y = leg_y - 4.0;
        let line_x2 = leg_x + 14.0;
        let text_x = leg_x + 18.0;
        svg.push_str(&format!(
            "  <line x1='{leg_x:.1}' y1='{line_y:.1}' x2='{line_x2:.1}' y2='{line_y:.1}' stroke='{col}' stroke-width='3' />\n  <text x='{text_x:.1}' y='{leg_y:.1}' font-family='sans-serif' font-size='10' fill='#424242'>{}</text>\n",
            html_escape(s_name)
        ));
        leg_x += (s_name.len() as f64 * 7.0) + 36.0;
    }

    svg.push_str("</svg>\n");
    svg
}

pub fn render_donut_chart_svg(
    title: &str,
    categories: &[String],
    values: &[f64],
    width: f64,
    height: f64,
) -> String {
    let total: f64 = values.iter().sum();
    let cx = width * 0.45;
    let cy = height * 0.52;
    let outer_r = (width.min(height) * 0.35).max(10.0);
    let inner_r = outer_r * 0.55;

    let mut svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {width} {height}' width='{width}' height='{height}'>\n  <rect width='100%' height='100%' fill='#FFFFFF' />\n  <text x='30' y='24' font-family='sans-serif' font-size='14' font-weight='bold' fill='#212121'>{}</text>\n",
        html_escape(title)
    );

    let mut cur_angle = -std::f64::consts::FRAC_PI_2;

    for (idx, &val) in values.iter().enumerate() {
        if val <= 0.0 || total <= 0.0 {
            continue;
        }
        let frac = val / total;
        let sweep = frac * 2.0 * std::f64::consts::PI;
        let end_angle = cur_angle + sweep;

        let x1_out = cx + outer_r * cur_angle.cos();
        let y1_out = cy + outer_r * cur_angle.sin();
        let x2_out = cx + outer_r * end_angle.cos();
        let y2_out = cy + outer_r * end_angle.sin();

        let x1_in = cx + inner_r * end_angle.cos();
        let y1_in = cy + inner_r * end_angle.sin();
        let x2_in = cx + inner_r * cur_angle.cos();
        let y2_in = cy + inner_r * cur_angle.sin();

        let large_arc = if sweep > std::f64::consts::PI { 1 } else { 0 };
        let col = series_color(idx);

        svg.push_str(&format!(
            "  <path d='M {x1_out:.2} {y1_out:.2} A {outer_r:.2} {outer_r:.2} 0 {large_arc} 1 {x2_out:.2} {y2_out:.2} L {x1_in:.2} {y1_in:.2} A {inner_r:.2} {inner_r:.2} 0 {large_arc} 0 {x2_in:.2} {y2_in:.2} Z' fill='{col}' stroke='#FFFFFF' stroke-width='1.5' />\n"
        ));

        cur_angle = end_angle;
    }

    // Legend on the right side
    let leg_x = width * 0.75;
    let mut leg_y = 60.0;
    for (idx, cat) in categories.iter().enumerate() {
        let val = values.get(idx).copied().unwrap_or(0.0);
        let pct = if total > 0.0 { val / total * 100.0 } else { 0.0 };
        let col = series_color(idx);
        let rect_y = leg_y - 8.0;
        let text_x = leg_x + 16.0;
        svg.push_str(&format!(
            "  <rect x='{leg_x:.1}' y='{rect_y:.1}' width='10' height='10' fill='{col}' rx='2' />\n  <text x='{text_x:.1}' y='{leg_y:.1}' font-family='sans-serif' font-size='10' fill='#424242'>{} ({pct:.1}%)</text>\n",
            html_escape(cat)
        ));
        leg_y += 20.0;
    }

    svg.push_str("</svg>\n");
    svg
}

// ---------------------------------------------------------------------------
// Choropleth Map Engine (GeoJSON + Data Matrix)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GeoFeature {
    pub name: String,
    pub polygons: Vec<Vec<(f64, f64)>>, // outer rings [lon, lat]
}

#[derive(Debug, Clone, Default)]
pub struct GeoMap {
    pub name: String,
    pub bbox: [f64; 4], // [min_lon, min_lat, max_lon, max_lat]
    pub features: Vec<GeoFeature>,
}

impl GeoMap {
    pub fn from_json_str(json_str: &str) -> Result<Self> {
        let root: serde_json::Value =
            serde_json::from_str(json_str).context("invalid GeoJSON syntax")?;

        let mut features = Vec::new();
        let mut min_lon = f64::INFINITY;
        let mut min_lat = f64::INFINITY;
        let mut max_lon = f64::NEG_INFINITY;
        let mut max_lat = f64::NEG_INFINITY;

        let raw_features = match root.get("features").and_then(serde_json::Value::as_array) {
            Some(arr) => arr.as_slice(),
            None => {
                if root.get("type").and_then(serde_json::Value::as_str) == Some("Feature") {
                    std::slice::from_ref(&root)
                } else {
                    bail!("GeoJSON has no 'features' array");
                }
            }
        };

        for feat in raw_features {
            let props = feat.get("properties").and_then(serde_json::Value::as_object);
            let name = props
                .and_then(|p| {
                    p.get("nmdesa")
                        .or_else(|| p.get("NMDESA"))
                        .or_else(|| p.get("nmkec"))
                        .or_else(|| p.get("NMKEC"))
                        .or_else(|| p.get("name"))
                        .or_else(|| p.get("NAME"))
                })
                .and_then(serde_json::Value::as_str)
                .unwrap_or("Unnamed")
                .trim()
                .to_string();

            let geom = match feat.get("geometry") {
                Some(g) if !g.is_null() => g,
                _ => continue,
            };

            let gtype = geom.get("type").and_then(serde_json::Value::as_str).unwrap_or("");
            let coords = match geom.get("coordinates") {
                Some(c) => c,
                None => continue,
            };

            let mut polygons = Vec::new();
            match gtype {
                "Polygon" => {
                    if let Some(rings) = coords.as_array() {
                        for ring in rings {
                            if let Some(pts) = parse_ring(ring, &mut min_lon, &mut min_lat, &mut max_lon, &mut max_lat)
                                && pts.len() >= 3
                            {
                                polygons.push(pts);
                            }
                        }
                    }
                }
                "MultiPolygon" => {
                    if let Some(poly_list) = coords.as_array() {
                        for poly in poly_list {
                            if let Some(rings) = poly.as_array() {
                                for ring in rings {
                                    if let Some(pts) = parse_ring(ring, &mut min_lon, &mut min_lat, &mut max_lon, &mut max_lat)
                                        && pts.len() >= 3
                                    {
                                        polygons.push(pts);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            if !polygons.is_empty() {
                features.push(GeoFeature { name, polygons });
            }
        }

        if min_lon > max_lon || min_lat > max_lat {
            bail!("GeoJSON contains no valid coordinates");
        }

        Ok(GeoMap {
            name: "GeoMap".into(),
            bbox: [min_lon, min_lat, max_lon, max_lat],
            features,
        })
    }
}

fn parse_ring(
    ring_val: &serde_json::Value,
    min_lon: &mut f64,
    min_lat: &mut f64,
    max_lon: &mut f64,
    max_lat: &mut f64,
) -> Option<Vec<(f64, f64)>> {
    let arr = ring_val.as_array()?;
    let mut pts = Vec::with_capacity(arr.len());
    for pt in arr {
        let pair = pt.as_array()?;
        if pair.len() >= 2 {
            let lon = pair[0].as_f64()?;
            let lat = pair[1].as_f64()?;
            if lon.is_finite() && lat.is_finite() {
                *min_lon = min_lon.min(lon);
                *max_lon = max_lon.max(lon);
                *min_lat = min_lat.min(lat);
                *max_lat = max_lat.max(lat);
                pts.push((lon, lat));
            }
        }
    }
    Some(pts)
}

pub fn render_choropleth_map_svg(
    title: &str,
    geo_map: &GeoMap,
    data: &HashMap<String, f64>,
    palette_idx: usize,
    width: f64,
    height: f64,
) -> String {
    let pad_left = 20.0;
    let pad_right = 20.0;
    let pad_top = 40.0;
    let pad_bottom = 40.0;

    let target_w = (width - pad_left - pad_right).max(10.0);
    let target_h = (height - pad_top - pad_bottom).max(10.0);

    let [min_lon, min_lat, max_lon, max_lat] = geo_map.bbox;
    let d_lon = (max_lon - min_lon).max(1e-9);
    let d_lat = (max_lat - min_lat).max(1e-9);

    let mean_lat_rad = ((min_lat + max_lat) * 0.5).to_radians();
    let cos_lat = mean_lat_rad.cos().abs().clamp(0.05, 1.0);
    let geo_w = d_lon * cos_lat;
    let geo_h = d_lat;
    let geo_aspect = geo_w / geo_h;
    let target_aspect = target_w / target_h;

    let (draw_w, draw_h) = if target_aspect > geo_aspect {
        (target_h * geo_aspect, target_h)
    } else {
        (target_w, target_w / geo_aspect)
    };

    let off_x = pad_left + (target_w - draw_w) * 0.5;
    let off_y = pad_top + (target_h - draw_h) * 0.5;

    let min_val = data.values().copied().fold(f64::INFINITY, f64::min);
    let max_val = data.values().copied().fold(f64::NEG_INFINITY, f64::max);
    let val_range = (max_val - min_val).max(1e-9);

    let (_, min_hex, max_hex) = MAP_COLOR_PRESETS[palette_idx % MAP_COLOR_PRESETS.len()];

    let mut svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {width} {height}' width='{width}' height='{height}'>\n  <desc>{}</desc>\n  <rect width='100%' height='100%' fill='#F8FAFC' />\n  <text x='20' y='24' font-family='sans-serif' font-size='14' font-weight='bold' fill='#212121'>{}</text>\n",
        html_escape(&geo_map.name),
        html_escape(title)
    );

    // Render polygon features
    for feat in &geo_map.features {
        let val_opt = data
            .get(&feat.name)
            .or_else(|| data.get(&feat.name.to_lowercase()))
            .or_else(|| data.get(&feat.name.to_uppercase()));

        let fill_col = match val_opt {
            Some(&v) => {
                let t = (v - min_val) / val_range;
                lerp_color(min_hex, max_hex, t)
            }
            None => "#E2E8F0".to_string(),
        };

        for poly in &feat.polygons {
            let mut path_d = String::new();
            for (idx, &(lon, lat)) in poly.iter().enumerate() {
                let norm_x = (lon - min_lon) / d_lon;
                let norm_y = (max_lat - lat) / d_lat;
                let px = off_x + norm_x * draw_w;
                let py = off_y + norm_y * draw_h;

                if idx == 0 {
                    path_d.push_str(&format!("M {px:.1} {py:.1}"));
                } else {
                    path_d.push_str(&format!(" L {px:.1} {py:.1}"));
                }
            }
            path_d.push_str(" Z");

            svg.push_str(&format!(
                "  <path d='{path_d}' fill='{fill_col}' stroke='#FFFFFF' stroke-width='0.8' />\n"
            ));
        }
    }

    // Legend color ramp at bottom
    if min_val.is_finite() && max_val.is_finite() {
        let leg_w = 120.0;
        let leg_h = 10.0;
        let leg_x = width - leg_w - 30.0;
        let leg_y = height - 20.0;
        let label1_x = leg_x - 6.0;
        let label1_y = leg_y + 8.0;
        let label2_x = leg_x + leg_w + 6.0;
        let label2_y = leg_y + 8.0;

        svg.push_str(&format!(
            "  <defs>\n    <linearGradient id='map_ramp' x1='0%' y1='0%' x2='100%' y2='0%'>\n      <stop offset='0%' stop-color='{min_hex}' />\n      <stop offset='100%' stop-color='{max_hex}' />\n    </linearGradient>\n  </defs>\n  <rect x='{leg_x:.1}' y='{leg_y:.1}' width='{leg_w:.1}' height='{leg_h:.1}' fill='url(#map_ramp)' rx='2' />\n  <text x='{label1_x:.1}' y='{label1_y:.1}' font-family='sans-serif' font-size='9' fill='#64748B' text-anchor='end'>{min_val:.1}</text>\n  <text x='{label2_x:.1}' y='{label2_y:.1}' font-family='sans-serif' font-size='9' fill='#64748B'>{max_val:.1}</text>\n"
        ));
    }

    svg.push_str("</svg>\n");
    svg
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
    fn chart_type_labels_are_localized() {
        assert_eq!(ChartType::Donut.label(UiLang::Id), "Donut / Pie Chart");
        assert_eq!(ChartType::Donut.label(UiLang::En), "Donut / Pie Chart");
        assert_eq!(ChartType::Map.label(UiLang::Id), "Peta Choropleth");
        assert_eq!(ChartType::Map.label(UiLang::En), "Choropleth Map");
        assert_eq!(ChartType::Map.label_en(), "Choropleth Map");
        assert_eq!(ChartType::Map.to_string(), "Choropleth Map");
    }

    #[test]
    fn test_bar_chart_svg() {
        let cats = vec!["2024".into(), "2025".into(), "2026".into()];
        let series = vec!["Padi".into(), "Jagung".into()];
        let data = vec![vec![100.0, 120.0, 140.0], vec![80.0, 95.0, 110.0]];
        let svg = render_bar_chart_svg("Produksi Tanaman Pangan", &cats, &series, &data, 500.0, 300.0);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Produksi Tanaman Pangan"));
        assert!(svg.contains("Padi"));
        assert!(svg.contains("2026"));
        assert!(svg.ends_with("</svg>\n"));
    }

    #[test]
    fn test_line_chart_svg() {
        let cats = vec!["Jan".into(), "Feb".into(), "Mar".into()];
        let series = vec!["NTP".into()];
        let data = vec![vec![105.4, 106.1, 105.8]];
        let svg = render_line_chart_svg("Perkembangan NTP", &cats, &series, &data, 450.0, 250.0);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("polyline"));
        assert!(svg.contains("circle"));
        assert!(svg.contains("Perkembangan NTP"));
    }

    #[test]
    fn test_donut_chart_svg() {
        let cats = vec!["Pertanian".into(), "Industri".into(), "Jasa".into()];
        let values = vec![35.0, 45.0, 20.0];
        let svg = render_donut_chart_svg("Distribusi PDRB", &cats, &values, 400.0, 300.0);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path d="));
        assert!(svg.contains("Distribusi PDRB"));
    }

    #[test]
    fn test_geojson_map_svg() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": { "name": "Kecamatan A" },
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[107.1, -6.3], [107.3, -6.3], [107.3, -6.1], [107.1, -6.1], [107.1, -6.3]]]
                    }
                }
            ]
        }"#;
        let geomap = GeoMap::from_json_str(geojson).unwrap();
        assert_eq!(geomap.features.len(), 1);
        assert_eq!(geomap.features[0].name, "Kecamatan A");

        let mut data = HashMap::new();
        data.insert("Kecamatan A".into(), 88.5);

        let svg = render_choropleth_map_svg("Peta Sebaran", &geomap, &data, 0, 400.0, 300.0);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path d="));
        assert!(svg.contains("Peta Sebaran"));
    }
}
