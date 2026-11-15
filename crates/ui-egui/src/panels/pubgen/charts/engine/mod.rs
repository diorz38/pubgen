//! GridCraft-inspired statistical charting engine for DesignCraft.
//!
//! Provides pure, toolkit-free drawing primitives, scaling, spline smoothing,
//! trendline regressions, and cartesian/combo layouts.

pub mod axis;
pub mod cartesian;
pub mod lines;
pub mod trend;
pub mod types;

pub use axis::{Scale, format_number, nice_scale};
pub use cartesian::render_cartesian;
pub use lines::{decimate, decimate_area, smooth_run};
pub use trend::{TrendFit, fit_trendline, moving_average};
pub use types::{
    ApproxMeasure, ChartConfig, ChartData, ChartKind, ChartStyle, HAlign, LegendPos, Measure, Prim, Rgba, SeriesData, Trendline, TrendlineData,
    TrendlineKind, VAlign,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nice_scale_ticks_and_frac() {
        let scale = nice_scale(12.0, 95.0, true);
        assert_eq!(scale.min, 0.0);
        assert!(scale.max >= 100.0);
        assert!(scale.step > 0.0);

        let ticks = scale.ticks();
        assert!(!ticks.is_empty());
        assert_eq!(ticks[0], 0.0);

        let frac_0 = scale.frac(0.0);
        let frac_max = scale.frac(scale.max);
        assert_eq!(frac_0, 0.0);
        assert_eq!(frac_max, 1.0);
    }

    #[test]
    fn test_catmull_rom_spline_smoothing() {
        let plot = lines::Bx { x: 0.0, y: 0.0, w: 500.0, h: 300.0 };
        let pts = vec![[10.0, 50.0], [100.0, 200.0], [200.0, 80.0], [300.0, 220.0]];
        let smoothed = smooth_run(pts.clone(), plot, 500);

        // Smoothed curve must generate more sub-segments than original 4 points
        assert!(smoothed.len() > pts.len());

        // First and last points should match original endpoints
        assert_eq!(smoothed.first().unwrap()[0], 10.0);
        assert_eq!(smoothed.last().unwrap()[0], 300.0);

        // All points must be clamped inside plot area
        for p in &smoothed {
            assert!(p[0] >= plot.x && p[0] <= plot.right());
            assert!(p[1] >= plot.y && p[1] <= plot.bottom());
        }
    }

    #[test]
    fn test_trendline_linear_fit() {
        let trend = Trendline { kind: TrendlineKind::Linear, intercept: None, display_equation: true, display_r2: true };

        // Perfectly linear: y = 2x + 10
        let pts = vec![(0.0, 10.0), (1.0, 12.0), (2.0, 14.0), (3.0, 16.0)];
        let fit = fit_trendline(&trend, &pts).expect("trendline must fit linear data");

        let eval_0 = fit.eval(0.0).expect("eval at 0");
        let eval_4 = fit.eval(4.0).expect("eval at 4");

        assert!((eval_0 - 10.0).abs() < 1e-4);
        assert!((eval_4 - 18.0).abs() < 1e-4);
        assert!(fit.r2.is_some_and(|r| (r - 1.0).abs() < 1e-4));
    }

    #[test]
    fn test_moving_average() {
        let pts = vec![(1.0, 10.0), (2.0, 20.0), (3.0, 30.0), (4.0, 40.0)];
        let ma = moving_average(&pts, 2);
        assert_eq!(ma.len(), 3);
        assert_eq!(ma[0], (2.0, 15.0)); // (10+20)/2
        assert_eq!(ma[1], (3.0, 25.0)); // (20+30)/2
        assert_eq!(ma[2], (4.0, 35.0)); // (30+40)/2
    }

    #[test]
    fn test_render_cartesian_dual_axis_and_spline() {
        let measure = ApproxMeasure;
        let mut data = ChartData::default();
        data.categories = vec!["2021".into(), "2022".into(), "2023".into(), "2024".into()];

        // Series 1: Column on primary axis
        data.series.push(SeriesData {
            name: "Produksi (Ton)".into(),
            values: vec![Some(120.0), Some(150.0), Some(180.0), Some(210.0)],
            color: [0x15, 0x60, 0x82, 0xFF],
            kind: ChartKind::ColumnClustered,
            secondary: false,
            smooth: false,
            ..Default::default()
        });

        // Series 2: Spline line on secondary axis
        data.series.push(SeriesData {
            name: "Pertumbuhan (%)".into(),
            values: vec![Some(5.2), Some(8.1), Some(6.4), Some(11.0)],
            color: [0xE0, 0x56, 0x38, 0xFF],
            kind: ChartKind::LineMarkers,
            secondary: true,
            smooth: true,
            ..Default::default()
        });

        let config = ChartConfig {
            title: Some("Produksi & Pertumbuhan Padi".into()),
            x_title: Some("Tahun".into()),
            y_title: Some("Ton".into()),
            y2_title: Some("%".into()),
            kind: ChartKind::ColumnClustered,
            legend: LegendPos::Bottom,
            show_grid: true,
            show_data_labels: false,
            show_data_table: false,
        };

        let style = ChartStyle::default();
        let prims = render_cartesian(&config, &data, &style, [0.0, 0.0, 600.0, 400.0], &measure);

        assert!(!prims.is_empty(), "prims must not be empty");

        // Verify title exists
        let has_title = prims.iter().any(|p| matches!(p, Prim::Text { text, .. } if text.contains("Produksi & Pertumbuhan Padi")));
        assert!(has_title, "title must be rendered");

        // Verify rects (4 columns + 1 legend swatch) exist
        let column_count =
            prims.iter().filter(|p| matches!(p, Prim::Rect { fill: Some(c), w, .. } if *c == [0x15, 0x60, 0x82, 0xFF] && *w > 12.0)).count();
        assert_eq!(column_count, 4, "must have 4 column rects");

        let swatch_count =
            prims.iter().filter(|p| matches!(p, Prim::Rect { fill: Some(c), w, .. } if *c == [0x15, 0x60, 0x82, 0xFF] && *w == 12.0)).count();
        assert_eq!(swatch_count, 1, "must have 1 legend swatch rect");

        // Verify line (spline curve) exists
        let has_line = prims.iter().any(|p| matches!(p, Prim::Line { color, pts, .. } if *color == [0xE0, 0x56, 0x38, 0xFF] && pts.len() > 4));
        assert!(has_line, "must have smoothed spline line for secondary series");

        // Verify secondary axis label exists
        let has_y2_title = prims.iter().any(|p| matches!(p, Prim::Text { text, .. } if text == "%"));
        assert!(has_y2_title, "must have y2 title");
    }
}
