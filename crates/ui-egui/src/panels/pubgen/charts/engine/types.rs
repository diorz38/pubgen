//! GridCraft-inspired chart primitives and data structures (toolkit-free).
//!
//! Provides drawing primitives, axis scaling, trendline calculation, spline interpolation,
//! and layout calculation emitting `Vec<Prim>`.

use serde::{Deserialize, Serialize};

/// Straight (non-premultiplied) RGBA colour [R, G, B, A].
pub type Rgba = [u8; 4];

pub const WHITE: Rgba = [0xFF, 0xFF, 0xFF, 0xFF];
pub const BLACK: Rgba = [0x00, 0x00, 0x00, 0xFF];
pub const DARK_TEXT: Rgba = [0x26, 0x26, 0x26, 0xFF];
pub const LIGHT_GRID: Rgba = [0xE5, 0xE7, 0xEB, 0xFF];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VAlign {
    #[default]
    Top,
    Middle,
    Bottom,
}

/// A drawing primitive in chart coordinates (origin top-left, y down, units = points/pixels).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Prim {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: Option<Rgba>,
        stroke: Option<(Rgba, f32)>,
        radius: f32,
    },
    Line {
        pts: Vec<[f32; 2]>,
        color: Rgba,
        width: f32,
        dash: bool,
    },
    Polygon {
        pts: Vec<[f32; 2]>,
        fill: Rgba,
        stroke: Option<(Rgba, f32)>,
    },
    /// Pie/doughnut sector: centre, radii, angles in radians (0 = 12 o'clock, clockwise).
    Wedge {
        cx: f32,
        cy: f32,
        r_outer: f32,
        r_inner: f32,
        a0: f32,
        a1: f32,
        fill: Rgba,
        stroke: Option<(Rgba, f32)>,
    },
    Circle {
        cx: f32,
        cy: f32,
        r: f32,
        fill: Option<Rgba>,
        stroke: Option<(Rgba, f32)>,
    },
    /// Text anchored at `(x, y)` per `align`/`valign`, rotated by `rotation` radians about the anchor.
    Text {
        x: f32,
        y: f32,
        text: String,
        size: f32,
        color: Rgba,
        bold: bool,
        align: HAlign,
        valign: VAlign,
        rotation: f32,
    },
}

/// Measures text width (the UI passes real font metrics; tests use a fixed-width approximation).
pub trait Measure: Send + Sync {
    fn text_width(&self, text: &str, size: f32, bold: bool) -> f32;
}

/// Fixed-width approximation: `0.55 × size` per character (bold 10% wider).
#[derive(Clone, Copy, Debug, Default)]
pub struct ApproxMeasure;

impl Measure for ApproxMeasure {
    fn text_width(&self, text: &str, size: f32, bold: bool) -> f32 {
        let n = text.chars().count() as f32;
        n * 0.55 * size * if bold { 1.1 } else { 1.0 }
    }
}

/// Chart Archetypes supported by the engine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartKind {
    #[default]
    ColumnClustered,
    ColumnStacked,
    ColumnStacked100,
    BarClustered,
    BarStacked,
    BarStacked100,
    Line,
    LineMarkers,
    LineStacked,
    Area,
    AreaStacked,
    AreaStacked100,
    Pie,
    Doughnut,
    Scatter,
    ScatterLines,
    Bubble,
    Radar,
    Histogram,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegendPos {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TrendlineKind {
    Linear,
    Exponential,
    Logarithmic,
    Power,
    Polynomial(u8),
    MovingAverage(usize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trendline {
    pub kind: TrendlineKind,
    pub intercept: Option<f64>,
    pub display_equation: bool,
    pub display_r2: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrendlineData {
    pub spec: Trendline,
    pub color: Rgba,
}

/// Series data with values, styling, and per-series options.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeriesData {
    pub name: String,
    pub values: Vec<Option<f64>>,
    pub x: Option<Vec<Option<f64>>>,
    pub sizes: Option<Vec<Option<f64>>>,
    pub color: Rgba,
    pub kind: ChartKind,
    pub secondary: bool,
    pub smooth: bool,
    pub trendlines: Vec<TrendlineData>,
    pub number_format: String,
}

impl Default for SeriesData {
    fn default() -> Self {
        SeriesData {
            name: String::new(),
            values: Vec::new(),
            x: None,
            sizes: None,
            color: [0x15, 0x60, 0x82, 0xFF],
            kind: ChartKind::ColumnClustered,
            secondary: false,
            smooth: false,
            trendlines: Vec::new(),
            number_format: "General".into(),
        }
    }
}

/// Resolved chart data (categories and series).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChartData {
    pub categories: Vec<String>,
    pub category_levels: Vec<Vec<String>>,
    pub series: Vec<SeriesData>,
    pub palette: Vec<Rgba>,
}

/// Styling configuration for chart borders, grids, fonts, and background.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChartStyle {
    pub text: Rgba,
    pub grid: Rgba,
    pub axis: Rgba,
    pub background: Option<Rgba>,
    pub card_border: Option<Rgba>,
}

impl Default for ChartStyle {
    fn default() -> Self {
        ChartStyle { text: DARK_TEXT, grid: LIGHT_GRID, axis: [0x9C, 0xA3, 0xAF, 0xFF], background: None, card_border: None }
    }
}

/// High-level configuration of the chart being rendered.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChartConfig {
    pub title: Option<String>,
    pub x_title: Option<String>,
    pub y_title: Option<String>,
    pub y2_title: Option<String>,
    pub kind: ChartKind,
    pub legend: LegendPos,
    pub show_grid: bool,
    pub show_data_labels: bool,
    pub show_data_table: bool,
}

impl Default for ChartConfig {
    fn default() -> Self {
        ChartConfig {
            title: None,
            x_title: None,
            y_title: None,
            y2_title: None,
            kind: ChartKind::ColumnClustered,
            legend: LegendPos::Bottom,
            show_grid: true,
            show_data_labels: false,
            show_data_table: false,
        }
    }
}
