//! Line smoothing (Catmull-Rom spline) and decimation.

pub const MAX_LINE_PTS: usize = 2000;

#[derive(Clone, Copy, Debug)]
pub struct Bx {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Bx {
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

/// Reduces a long polyline to first/min/max/last per pixel column.
pub fn decimate(pts: Vec<[f32; 2]>, max: usize) -> Vec<[f32; 2]> {
    if pts.len() <= max {
        return pts;
    }
    let mut out = Vec::with_capacity(max * 2);
    let mut i = 0;
    while let Some(p) = pts.get(i) {
        let col = p[0].floor();
        let (start, mut mn, mut mx) = (i, i, i);
        let mut k = i + 1;
        while let Some(q) = pts.get(k) {
            if q[0].floor() != col {
                break;
            }
            if pts.get(mn).is_some_and(|m| q[1] < m[1]) {
                mn = k;
            }
            if pts.get(mx).is_some_and(|m| q[1] > m[1]) {
                mx = k;
            }
            k += 1;
        }
        let mut idx = [start, mn, mx, k - 1];
        idx.sort_unstable();
        let mut last = usize::MAX;
        for ix in idx {
            if ix != last
                && let Some(q) = pts.get(ix)
            {
                out.push(*q);
                last = ix;
            }
        }
        i = k;
    }
    if out.len() > max * 4 {
        let step = out.len() / (max * 2) + 1;
        let last = out.last().copied();
        let mut s: Vec<[f32; 2]> = out.into_iter().step_by(step).collect();
        if let Some(l) = last
            && s.last() != Some(&l)
        {
            s.push(l);
        }
        return s;
    }
    out
}

/// Catmull-Rom spline through `pts`: every segment gets up to 100 sub-segments,
/// chosen so the whole curve stays within `cap` points. Clamped to `plot` box.
pub fn smooth_run(pts: Vec<[f32; 2]>, plot: Bx, cap: usize) -> Vec<[f32; 2]> {
    let n = pts.len();
    if n < 3 {
        return pts;
    }
    let steps = (cap / n).clamp(1, 100);
    let mut out = Vec::with_capacity(n * steps + 1);

    // Centripetal / uniform Catmull-Rom spline
    for i in 0..n.saturating_sub(1) {
        let p0 = if i == 0 { pts.first().copied().unwrap_or([0.0, 0.0]) } else { pts.get(i - 1).copied().unwrap_or([0.0, 0.0]) };
        let p1 = pts.get(i).copied().unwrap_or([0.0, 0.0]);
        let p2 = pts.get(i + 1).copied().unwrap_or([0.0, 0.0]);
        let p3 = if i + 2 < n { pts.get(i + 2).copied().unwrap_or(p2) } else { p2 };

        for step in 0..steps {
            let t = step as f32 / steps as f32;
            let t2 = t * t;
            let t3 = t2 * t;

            // Catmull-Rom standard matrix evaluation
            let qx = 0.5
                * ((2.0 * p1[0])
                    + (-p0[0] + p2[0]) * t
                    + (2.0 * p0[0] - 5.0 * p1[0] + 4.0 * p2[0] - p3[0]) * t2
                    + (-p0[0] + 3.0 * p1[0] - 3.0 * p2[0] + p3[0]) * t3);
            let qy = 0.5
                * ((2.0 * p1[1])
                    + (-p0[1] + p2[1]) * t
                    + (2.0 * p0[1] - 5.0 * p1[1] + 4.0 * p2[1] - p3[1]) * t2
                    + (-p0[1] + 3.0 * p1[1] - 3.0 * p2[1] + p3[1]) * t3);

            // Clamp overshoot to the plot area
            let cx = qx.clamp(plot.x, plot.right());
            let cy = qy.clamp(plot.y, plot.bottom());
            out.push([cx, cy]);
        }
    }
    if let Some(last) = pts.last() {
        out.push([last[0].clamp(plot.x, plot.right()), last[1].clamp(plot.y, plot.bottom())]);
    }
    out
}

/// Decimates area vertices preserving top and baseline pairs.
pub fn decimate_area(top: Vec<[f32; 2]>, base: Vec<[f32; 2]>, max: usize) -> Vec<[f32; 2]> {
    let n = top.len().min(base.len());
    if n == 0 {
        return Vec::new();
    }
    if n <= max {
        let mut poly = top;
        poly.truncate(n);
        poly.extend(base.into_iter().take(n).rev());
        return poly;
    }
    let step = n / max + 1;
    let mut poly = Vec::with_capacity(max * 2 + 2);
    for i in (0..n).step_by(step) {
        if let Some(p) = top.get(i) {
            poly.push(*p);
        }
    }
    if let Some(last_top) = top.get(n - 1) {
        poly.push(*last_top);
    }
    if let Some(last_base) = base.get(n - 1) {
        poly.push(*last_base);
    }
    for i in (0..n).step_by(step).rev() {
        if let Some(p) = base.get(i) {
            poly.push(*p);
        }
    }
    poly
}
