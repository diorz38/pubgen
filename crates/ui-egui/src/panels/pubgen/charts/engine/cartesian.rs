//! Cartesian chart layout engine (Column, Bar, Line, Area, Combo, Secondary Axis).

use super::axis::{Scale, format_number, nice_scale};
use super::lines::{Bx, decimate_area, smooth_run};
use super::trend::fit_trendline;
use super::types::{ChartConfig, ChartData, ChartKind, ChartStyle, HAlign, LegendPos, Measure, Prim, Rgba, SeriesData, VAlign, WHITE};

pub const TITLE_SIZE: f32 = 13.0;
pub const AXIS_TITLE_SIZE: f32 = 9.5;
pub const TICK_LABEL_SIZE: f32 = 8.5;
pub const DATA_LABEL_SIZE: f32 = 8.0;

/// Text color on top of a fill color for contrast.
pub fn contrast_text(fill: Rgba) -> Rgba {
    let l = 0.299 * fill[0] as f32 + 0.587 * fill[1] as f32 + 0.114 * fill[2] as f32;
    if l < 150.0 { WHITE } else { [0x26, 0x26, 0x26, 0xFF] }
}

/// Renders a full cartesian chart within `bounds = [x, y, w, h]`.
pub fn render_cartesian(config: &ChartConfig, data: &ChartData, style: &ChartStyle, bounds: [f32; 4], measure: &dyn Measure) -> Vec<Prim> {
    let mut prims = Vec::new();
    let [bx, by, bw, bh] = bounds;

    // Optional background
    if let Some(bg) = style.background {
        prims.push(Prim::Rect { x: bx, y: by, w: bw, h: bh, fill: Some(bg), stroke: style.card_border.map(|c| (c, 1.0)), radius: 0.0 });
    }

    let mut cur_y = by + 6.0;
    let mut cur_h = bh - 12.0;

    // Title
    if let Some(ref title) = config.title {
        if !title.trim().is_empty() {
            prims.push(Prim::Text {
                x: bx + bw / 2.0,
                y: cur_y,
                text: title.trim().to_string(),
                size: TITLE_SIZE,
                color: style.text,
                bold: true,
                align: HAlign::Center,
                valign: VAlign::Top,
                rotation: 0.0,
            });
            let th = TITLE_SIZE * 1.5;
            cur_y += th;
            cur_h = (cur_h - th).max(0.0);
        }
    }

    // Legend reservation
    let legend_h = if config.legend != LegendPos::None && !data.series.is_empty() { 24.0 } else { 0.0 };
    if config.legend == LegendPos::Top && legend_h > 0.0 {
        render_legend(&mut prims, config, data, style, bx, cur_y, bw, legend_h);
        cur_y += legend_h;
        cur_h = (cur_h - legend_h).max(0.0);
    } else if config.legend == LegendPos::Bottom && legend_h > 0.0 {
        cur_h = (cur_h - legend_h).max(0.0);
    }

    // Split series into primary (left) and secondary (right)
    let (primary_series, secondary_series): (Vec<&SeriesData>, Vec<&SeriesData>) = data.series.iter().partition(|s| !s.secondary);

    let has_secondary = !secondary_series.is_empty();

    // Calculate value range for primary axis
    let (p_min, p_max) = calculate_range(&primary_series, config.kind);
    let force_zero = matches!(
        config.kind,
        ChartKind::ColumnClustered
            | ChartKind::ColumnStacked
            | ChartKind::BarClustered
            | ChartKind::BarStacked
            | ChartKind::Area
            | ChartKind::AreaStacked
    );
    let scale_p = nice_scale(p_min, p_max, force_zero);

    // Calculate value range for secondary axis if present
    let scale_s = if has_secondary {
        let (s_min, s_max) = calculate_range(&secondary_series, config.kind);
        Some(nice_scale(s_min, s_max, false))
    } else {
        None
    };

    // Calculate axis widths
    let p_ticks = scale_p.ticks();
    let max_p_label_w = p_ticks.iter().map(|t| measure.text_width(&format_number(*t, None), TICK_LABEL_SIZE, false)).fold(20.0f32, f32::max);

    let left_margin = max_p_label_w + 14.0 + if config.y_title.is_some() { 16.0 } else { 0.0 };

    let right_margin = if let Some(ref s_scale) = scale_s {
        let max_s_label_w =
            s_scale.ticks().iter().map(|t| measure.text_width(&format_number(*t, None), TICK_LABEL_SIZE, false)).fold(20.0f32, f32::max);
        max_s_label_w + 14.0 + if config.y2_title.is_some() { 16.0 } else { 0.0 }
    } else {
        16.0
    };

    let bottom_margin = 28.0 + if config.x_title.is_some() { 16.0 } else { 0.0 };

    let plot_x = bx + left_margin;
    let plot_y = cur_y + 8.0;
    let plot_w = (bw - left_margin - right_margin).max(40.0);
    let plot_h = (cur_h - bottom_margin - 8.0).max(40.0);
    let plot_box = Bx { x: plot_x, y: plot_y, w: plot_w, h: plot_h };

    // Draw Primary Y-Axis Ticks & Horizontal Grid lines
    for t in &p_ticks {
        let frac = scale_p.frac(*t);
        let gy = plot_y + plot_h * (1.0 - frac);

        if config.show_grid {
            prims.push(Prim::Line { pts: vec![[plot_x, gy], [plot_x + plot_w, gy]], color: style.grid, width: 0.75, dash: false });
        }

        // Left tick label
        prims.push(Prim::Text {
            x: plot_x - 6.0,
            y: gy,
            text: format_number(*t, None),
            size: TICK_LABEL_SIZE,
            color: style.text,
            bold: false,
            align: HAlign::Right,
            valign: VAlign::Middle,
            rotation: 0.0,
        });
    }

    // Draw Secondary Y-Axis Ticks if present
    if let Some(ref s_scale) = scale_s {
        for t in s_scale.ticks() {
            let frac = s_scale.frac(t);
            let gy = plot_y + plot_h * (1.0 - frac);

            prims.push(Prim::Text {
                x: plot_x + plot_w + 6.0,
                y: gy,
                text: format_number(t, None),
                size: TICK_LABEL_SIZE,
                color: style.text,
                bold: false,
                align: HAlign::Left,
                valign: VAlign::Middle,
                rotation: 0.0,
            });
        }
    }

    // Category count
    let num_cats = data.categories.len().max(1);
    let slot_w = plot_w / num_cats as f32;

    // Draw Category Labels below plot
    for (i, cat) in data.categories.iter().enumerate() {
        let cx = plot_x + i as f32 * slot_w + slot_w / 2.0;
        let cy = plot_y + plot_h + 8.0;

        prims.push(Prim::Text {
            x: cx,
            y: cy,
            text: cat.clone(),
            size: TICK_LABEL_SIZE,
            color: style.text,
            bold: false,
            align: HAlign::Center,
            valign: VAlign::Top,
            rotation: 0.0,
        });
    }

    // Render Data Elements: Columns, Bars, Lines, Areas
    render_series_elements(&mut prims, data, config, &scale_p, scale_s.as_ref(), plot_box, num_cats, slot_w);

    // Draw Axis Titles if present
    if let Some(ref yt) = config.y_title {
        prims.push(Prim::Text {
            x: bx + 4.0,
            y: plot_y + plot_h / 2.0,
            text: yt.clone(),
            size: AXIS_TITLE_SIZE,
            color: style.text,
            bold: false,
            align: HAlign::Center,
            valign: VAlign::Middle,
            rotation: -std::f32::consts::FRAC_PI_2,
        });
    }

    if let Some(ref yt2) = config.y2_title {
        prims.push(Prim::Text {
            x: bx + bw - 4.0,
            y: plot_y + plot_h / 2.0,
            text: yt2.clone(),
            size: AXIS_TITLE_SIZE,
            color: style.text,
            bold: false,
            align: HAlign::Center,
            valign: VAlign::Middle,
            rotation: std::f32::consts::FRAC_PI_2,
        });
    }

    if let Some(ref xt) = config.x_title {
        prims.push(Prim::Text {
            x: plot_x + plot_w / 2.0,
            y: plot_y + plot_h + 20.0,
            text: xt.clone(),
            size: AXIS_TITLE_SIZE,
            color: style.text,
            bold: false,
            align: HAlign::Center,
            valign: VAlign::Top,
            rotation: 0.0,
        });
    }

    // Bottom Legend
    if config.legend == LegendPos::Bottom && legend_h > 0.0 {
        let leg_y = by + bh - legend_h - 4.0;
        render_legend(&mut prims, config, data, style, bx, leg_y, bw, legend_h);
    }

    prims
}

fn calculate_range(series: &[&SeriesData], _kind: ChartKind) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;

    for s in series {
        for v in s.values.iter().flatten() {
            if v.is_finite() {
                lo = lo.min(*v);
                hi = hi.max(*v);
            }
        }
    }

    if !lo.is_finite() || !hi.is_finite() {
        return (0.0, 1.0);
    }
    (lo, hi)
}

fn render_series_elements(
    prims: &mut Vec<Prim>,
    data: &ChartData,
    _config: &ChartConfig,
    scale_p: &Scale,
    scale_s: Option<&Scale>,
    plot: Bx,
    num_cats: usize,
    slot_w: f32,
) {
    let bar_series: Vec<&SeriesData> = data.series.iter().filter(|s| is_bar_kind(s.kind)).collect();
    let num_bar_series = bar_series.len().max(1);

    for s in &data.series {
        let active_scale = if s.secondary && scale_s.is_some() { scale_s.unwrap_or(scale_p) } else { scale_p };

        match s.kind {
            ChartKind::ColumnClustered => {
                let bar_slot = slot_w * 0.8;
                let single_bar_w = (bar_slot / num_bar_series as f32).max(2.0);
                let col_offset = bar_series.iter().position(|bs| std::ptr::eq(*bs, s)).unwrap_or(0);

                for (c_idx, val_opt) in s.values.iter().enumerate().take(num_cats) {
                    if let Some(val) = val_opt {
                        if !val.is_finite() {
                            continue;
                        }
                        let frac = active_scale.frac(*val);
                        let zero_frac = active_scale.frac(0.0);

                        let bar_top = plot.y + plot.h * (1.0 - frac.max(zero_frac));
                        let bar_h = (plot.h * (frac - zero_frac).abs()).max(1.0);

                        let bar_x = plot.x + c_idx as f32 * slot_w + (slot_w - bar_slot) / 2.0 + col_offset as f32 * single_bar_w;

                        prims.push(Prim::Rect {
                            x: bar_x,
                            y: bar_top,
                            w: single_bar_w - 1.0,
                            h: bar_h,
                            fill: Some(s.color),
                            stroke: None,
                            radius: 1.0,
                        });
                    }
                }
            }
            ChartKind::Line | ChartKind::LineMarkers => {
                let mut pts: Vec<[f32; 2]> = Vec::new();

                for (c_idx, val_opt) in s.values.iter().enumerate().take(num_cats) {
                    if let Some(val) = val_opt {
                        if !val.is_finite() {
                            continue;
                        }
                        let frac = active_scale.frac(*val);
                        let px = plot.x + c_idx as f32 * slot_w + slot_w / 2.0;
                        let py = plot.y + plot.h * (1.0 - frac);
                        pts.push([px, py]);
                    }
                }

                if pts.len() >= 2 {
                    let final_pts = if s.smooth { smooth_run(pts.clone(), plot, 1000) } else { pts.clone() };

                    prims.push(Prim::Line { pts: final_pts, color: s.color, width: 2.0, dash: false });
                }

                // Markers
                if s.kind == ChartKind::LineMarkers || pts.len() == 1 {
                    for pt in &pts {
                        prims.push(Prim::Circle { cx: pt[0], cy: pt[1], r: 3.5, fill: Some(s.color), stroke: Some((WHITE, 1.0)) });
                    }
                }
            }
            ChartKind::Area => {
                let mut top_pts: Vec<[f32; 2]> = Vec::new();
                let mut base_pts: Vec<[f32; 2]> = Vec::new();
                let zero_frac = active_scale.frac(0.0);
                let base_y = plot.y + plot.h * (1.0 - zero_frac);

                for (c_idx, val_opt) in s.values.iter().enumerate().take(num_cats) {
                    if let Some(val) = val_opt {
                        if !val.is_finite() {
                            continue;
                        }
                        let frac = active_scale.frac(*val);
                        let px = plot.x + c_idx as f32 * slot_w + slot_w / 2.0;
                        let py = plot.y + plot.h * (1.0 - frac);
                        top_pts.push([px, py]);
                        base_pts.push([px, base_y]);
                    }
                }

                if top_pts.len() >= 2 {
                    let poly_pts = decimate_area(top_pts.clone(), base_pts, 500);
                    let mut fill_color = s.color;
                    fill_color[3] = 160; // Semi-transparent area fill

                    prims.push(Prim::Polygon { pts: poly_pts, fill: fill_color, stroke: None });

                    // Top stroke
                    prims.push(Prim::Line { pts: top_pts, color: s.color, width: 1.5, dash: false });
                }
            }
            _ => {}
        }

        // Trendlines
        for tl in &s.trendlines {
            let pts_f64: Vec<(f64, f64)> = s.values.iter().enumerate().filter_map(|(i, v)| v.map(|val| (i as f64, val))).collect();

            if let Some(fit) = fit_trendline(&tl.spec, &pts_f64) {
                let mut trend_pts = Vec::new();
                for i in 0..num_cats {
                    if let Some(y_val) = fit.eval(i as f64) {
                        let frac = active_scale.frac(y_val);
                        let px = plot.x + i as f32 * slot_w + slot_w / 2.0;
                        let py = plot.y + plot.h * (1.0 - frac);
                        trend_pts.push([px, py]);
                    }
                }
                if trend_pts.len() >= 2 {
                    prims.push(Prim::Line { pts: trend_pts, color: tl.color, width: 1.5, dash: true });
                }
            }
        }
    }
}

fn is_bar_kind(k: ChartKind) -> bool {
    matches!(k, ChartKind::ColumnClustered | ChartKind::ColumnStacked | ChartKind::BarClustered | ChartKind::BarStacked)
}

fn render_legend(prims: &mut Vec<Prim>, _config: &ChartConfig, data: &ChartData, style: &ChartStyle, x: f32, y: f32, w: f32, _h: f32) {
    let mut cx = x + 12.0;
    for s in &data.series {
        // Swatch
        prims.push(Prim::Rect { x: cx, y: y + 5.0, w: 12.0, h: 10.0, fill: Some(s.color), stroke: None, radius: 1.5 });

        // Label
        prims.push(Prim::Text {
            x: cx + 16.0,
            y: y + 4.0,
            text: s.name.clone(),
            size: TICK_LABEL_SIZE,
            color: style.text,
            bold: false,
            align: HAlign::Left,
            valign: VAlign::Top,
            rotation: 0.0,
        });

        cx += (s.name.len() as f32 * 6.0) + 36.0;
        if cx > x + w - 30.0 {
            break;
        }
    }
}
