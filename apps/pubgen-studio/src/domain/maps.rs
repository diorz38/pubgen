//! Pure SVG choropleth map generator for GeoJSON polygons.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct GeoFeature {
    pub name: String,
    pub polygons: Vec<Vec<(f64, f64)>>,
}

#[derive(Debug, Clone, Default)]
pub struct GeoMap {
    pub name: String,
    pub bbox: [f64; 4], // [min_lon, min_lat, max_lon, max_lat]
    pub features: Vec<GeoFeature>,
}

impl GeoMap {
    pub fn from_json_str(json_str: &str) -> Result<Self, String> {
        let root: serde_json::Value = serde_json::from_str(json_str).map_err(|e| format!("Invalid GeoJSON syntax: {e}"))?;

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
                    return Err("GeoJSON has no 'features' array".into());
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
            return Err("GeoJSON contains no valid coordinates".into());
        }

        Ok(GeoMap { name: "GeoMap".into(), bbox: [min_lon, min_lat, max_lon, max_lat], features })
    }
}

fn parse_ring(ring_val: &serde_json::Value, min_lon: &mut f64, min_lat: &mut f64, max_lon: &mut f64, max_lat: &mut f64) -> Option<Vec<(f64, f64)>> {
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

pub fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(128);
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(128);
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(128);
        (r, g, b)
    } else {
        (128, 128, 128)
    }
}

pub fn lerp_color(min_hex: &str, max_hex: &str, t: f64) -> String {
    let t_clamped = t.clamp(0.0, 1.0);
    let (r1, g1, b1) = parse_hex_color(min_hex);
    let (r2, g2, b2) = parse_hex_color(max_hex);
    let r = (f64::from(r1) + (f64::from(r2) - f64::from(r1)) * t_clamped).round() as u8;
    let g = (f64::from(g1) + (f64::from(g2) - f64::from(g1)) * t_clamped).round() as u8;
    let b = (f64::from(b1) + (f64::from(b2) - f64::from(b1)) * t_clamped).round() as u8;
    format!("#{r:02X}{g:02X}{b:02X}")
}

pub fn render_choropleth_map_svg(
    title: &str,
    geo_map: &GeoMap,
    data: &HashMap<String, f64>,
    min_color: &str,
    max_color: &str,
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

    let (draw_w, draw_h) = if target_aspect > geo_aspect { (target_h * geo_aspect, target_h) } else { (target_w, target_w / geo_aspect) };

    let offset_x = pad_left + (target_w - draw_w) * 0.5;
    let offset_y = pad_top + (target_h - draw_h) * 0.5;

    let mut val_min = f64::INFINITY;
    let mut val_max = f64::NEG_INFINITY;
    for &v in data.values() {
        if v.is_finite() {
            val_min = val_min.min(v);
            val_max = val_max.max(v);
        }
    }
    if val_min > val_max {
        val_min = 0.0;
        val_max = 1.0;
    }
    let val_range = (val_max - val_min).max(1e-9);

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.1} {height:.1}\" width=\"{width:.1}\" height=\"{height:.1}\">\n"
    ));
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\" />\n");
    svg.push_str(&format!(
        "<text x=\"{pad_left}\" y=\"24\" font-family=\"sans-serif\" font-size=\"14\" font-weight=\"bold\" fill=\"#1e293b\">{title}</text>\n"
    ));
    svg.push_str("<g id=\"regions\">\n");

    for feat in &geo_map.features {
        let val = data.get(&feat.name).copied().unwrap_or(val_min);
        let t = ((val - val_min) / val_range).clamp(0.0, 1.0);
        let fill = lerp_color(min_color, max_color, t);
        let name = &feat.name;

        for poly in &feat.polygons {
            let mut path_d = String::new();
            for (idx, &(lon, lat)) in poly.iter().enumerate() {
                let norm_x = (lon - min_lon) / d_lon;
                let norm_y = (max_lat - lat) / d_lat;
                let px = offset_x + norm_x * draw_w;
                let py = offset_y + norm_y * draw_h;

                if idx == 0 {
                    path_d.push_str(&format!("M{px:.1},{py:.1}"));
                } else {
                    path_d.push_str(&format!(" L{px:.1},{py:.1}"));
                }
            }
            path_d.push('Z');
            svg.push_str(&format!(
                "  <path d=\"{path_d}\" fill=\"{fill}\" stroke=\"#ffffff\" stroke-width=\"0.8\" opacity=\"0.95\"><title>{name}: {val:.1}</title></path>\n"
            ));
        }
    }

    svg.push_str("</g>\n</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choropleth_map_generation() {
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": { "name": "Kecamatan Utara" },
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[112.0, -7.0], [112.5, -7.0], [112.5, -7.5], [112.0, -7.5], [112.0, -7.0]]]
                    }
                },
                {
                    "type": "Feature",
                    "properties": { "name": "Kecamatan Selatan" },
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[112.0, -7.5], [112.5, -7.5], [112.5, -8.0], [112.0, -8.0], [112.0, -7.5]]]
                    }
                }
            ]
        }"#;

        let map = GeoMap::from_json_str(geojson).expect("Failed to parse GeoJSON");
        assert_eq!(map.features.len(), 2);

        let mut data = HashMap::new();
        data.insert("Kecamatan Utara".into(), 85.0);
        data.insert("Kecamatan Selatan".into(), 42.0);

        let svg = render_choropleth_map_svg("Peta Tematik Kemiskinan", &map, &data, "#fee2e2", "#b91c1c", 600.0, 400.0);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("Peta Tematik Kemiskinan"));
        assert!(svg.contains("<path d="));
        assert!(svg.contains("</svg>"));
    }
}
