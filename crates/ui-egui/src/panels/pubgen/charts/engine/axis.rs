//! Value-axis scaling ("nice numbers") and label formatting.

/// Largest magnitude we plot; bigger values are clamped so ranges stay finite.
pub const MAX_ABS: f64 = 1e300;

/// A linear axis scale: `min..=max` with gridlines every `step`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl Scale {
    /// Tick values from `min` to `max` (at most 51).
    pub fn ticks(&self) -> Vec<f64> {
        let span = self.max - self.min;
        if !(span.is_finite() && self.step.is_finite() && self.step > 0.0 && span > 0.0) {
            return vec![self.min];
        }
        let n = ((span / self.step).round() as i64).clamp(1, 50);
        (0..=n)
            .map(|i| {
                let v = self.min + i as f64 * self.step;
                // Snap away accumulated float error
                let snapped = (v / self.step).round() * self.step;
                let v = if (snapped - v).abs() < self.step * 1e-9 { snapped } else { v };
                if v.abs() < self.step * 1e-9 { 0.0 } else { v }
            })
            .collect()
    }

    /// Position of `v` along the axis, 0 at `min`, 1 at `max` (clamped to a sane band).
    pub fn frac(&self, v: f64) -> f32 {
        let span = self.max - self.min;
        if !(span.is_finite() && span > 0.0 && v.is_finite()) {
            return 0.0;
        }
        (((v - self.min) / span).clamp(-0.5, 1.5)) as f32
    }
}

pub fn clean(v: f64) -> f64 {
    if v.is_finite() { v.clamp(-MAX_ABS, MAX_ABS) } else { 0.0 }
}

/// Picks a "nice" scale (steps of 1, 2 or 5 × 10ⁿ, 4–9 gridlines) covering `lo..=hi`.
///
/// `force_zero` always includes 0 (columns, bars, areas). Otherwise 0 is still included when the
/// data sits reasonably close to it (min below 5/6 of max).
pub fn nice_scale(lo: f64, hi: f64, force_zero: bool) -> Scale {
    let (mut lo, mut hi) = (clean(lo), clean(hi));
    if lo > hi {
        std::mem::swap(&mut lo, &mut hi);
    }
    if force_zero || (lo > 0.0 && lo < hi * 5.0 / 6.0) || lo == hi {
        lo = lo.min(0.0);
    }
    if force_zero || (hi < 0.0 && hi > lo * 5.0 / 6.0) || lo == hi {
        hi = hi.max(0.0);
    }
    if lo == hi {
        // Both zero or equal
        return Scale { min: 0.0, max: 1.0, step: 0.2 };
    }
    let range = hi - lo;
    if hi > 0.0 {
        hi += range * 0.05;
    }
    if lo < 0.0 {
        lo -= range * 0.05;
    }
    let range = hi - lo;
    if !range.is_finite() || range <= 0.0 {
        return Scale { min: 0.0, max: 1.0, step: 0.2 };
    }
    let e = (range / 8.0).log10().floor();
    let mut chosen = None;
    'outer: for k in [e - 1.0, e, e + 1.0, e + 2.0] {
        let p = 10f64.powf(k);
        for m in [1.0, 2.0, 5.0] {
            let s = m * p;
            if !(s.is_finite() && s > 0.0) {
                continue;
            }
            let intervals = (hi / s - 1e-9).ceil() - (lo / s + 1e-9).floor();
            if intervals <= 8.0 {
                chosen = Some(s);
                break 'outer;
            }
        }
    }
    let step = chosen.unwrap_or(range);
    let mut min = (lo / step + 1e-9).floor() * step;
    let mut max = (hi / step - 1e-9).ceil() * step;
    if min.abs() < step * 1e-9 {
        min = 0.0;
    }
    if max.abs() < step * 1e-9 {
        max = 0.0;
    }
    Scale { min, max, step }
}

/// Formats a number for axis ticks or data labels.
pub fn format_number(v: f64, decimals: Option<usize>) -> String {
    if !v.is_finite() {
        return "0".to_string();
    }
    if let Some(d) = decimals {
        format!("{v:.d$}")
    } else if v.fract().abs() < 1e-6 {
        format!("{:.0}", v)
    } else if v.abs() >= 1000.0 {
        format!("{:.1}", v)
    } else {
        format!("{:.2}", v)
    }
}
