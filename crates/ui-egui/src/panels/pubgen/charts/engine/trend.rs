//! Trendline mathematics: least-squares fits and moving averages.

use super::types::{Trendline, TrendlineKind};

/// A fitted regression trendline.
#[derive(Clone, Debug, PartialEq)]
pub struct TrendFit {
    pub kind: TrendlineKind,
    /// Coefficients `c0..=cd` of `Y = Σ cₖ·Xᵏ`.
    pub coef: Vec<f64>,
    center: f64,
    scale: f64,
    a: Vec<f64>,
    /// Coefficient of determination R² (None when undefined).
    pub r2: Option<f64>,
}

fn degree(k: TrendlineKind) -> Option<usize> {
    match k {
        TrendlineKind::Linear | TrendlineKind::Exponential | TrendlineKind::Logarithmic | TrendlineKind::Power => Some(1),
        TrendlineKind::Polynomial(o) => Some(usize::from(o.clamp(2, 6))),
        TrendlineKind::MovingAverage(_) => None,
    }
}

fn log_x(k: TrendlineKind) -> bool {
    matches!(k, TrendlineKind::Logarithmic | TrendlineKind::Power)
}

fn log_y(k: TrendlineKind) -> bool {
    matches!(k, TrendlineKind::Exponential | TrendlineKind::Power)
}

/// Fits `t` to points `(x, y)`. Returns None if data is degenerate or non-finite.
pub fn fit_trendline(t: &Trendline, pts: &[(f64, f64)]) -> Option<TrendFit> {
    let kind = t.kind;
    let d = degree(kind)?;
    let data: Vec<(f64, f64)> = pts
        .iter()
        .filter(|(x, y)| x.is_finite() && y.is_finite() && (!log_x(kind) || *x > 0.0) && (!log_y(kind) || *y > 0.0))
        .map(|&(x, y)| (if log_x(kind) { x.ln() } else { x }, if log_y(kind) { y.ln() } else { y }))
        .collect();
    if data.len() < 2 || data.len() <= d {
        return None;
    }
    let fixed = match kind {
        TrendlineKind::Linear | TrendlineKind::Polynomial(_) => t.intercept,
        TrendlineKind::Exponential => t.intercept.filter(|b| *b > 0.0).map(f64::ln),
        _ => None,
    }
    .filter(|b| b.is_finite());
    let n = data.len() as f64;
    let center = if fixed.is_some() { 0.0 } else { data.iter().map(|p| p.0).sum::<f64>() / n };
    let scale = data.iter().map(|p| (p.0 - center).abs()).fold(0.0f64, f64::max);
    if !(scale.is_finite() && scale > 0.0) {
        return None;
    }
    let first = usize::from(fixed.is_some());
    let b0 = fixed.unwrap_or(0.0);
    let m = d + 1 - first;
    let mut ata = vec![vec![0.0f64; m]; m];
    let mut aty = vec![0.0f64; m];
    for &(x, y) in &data {
        let u = (x - center) / scale;
        let mut buf = [0.0f64; 7];
        for (slot, k) in buf.iter_mut().zip(first..=d) {
            *slot = u.powi(k as i32);
        }
        let pows = buf.get(..m)?;
        for (i, pi) in pows.iter().enumerate() {
            if let Some(v) = aty.get_mut(i) {
                *v += pi * (y - b0);
            }
            for (j, pj) in pows.iter().enumerate() {
                if let Some(v) = ata.get_mut(i).and_then(|r| r.get_mut(j)) {
                    *v += pi * pj;
                }
            }
        }
    }
    let sol = solve(ata, aty)?;
    let mut a = vec![b0; d + 1];
    for (k, v) in sol.into_iter().enumerate() {
        if let Some(slot) = a.get_mut(k + first) {
            *slot = v;
        }
    }
    let coef = expand(&a, center, scale)?;
    let fit = TrendFit { kind, coef, center, scale, a, r2: None };
    let mean = data.iter().map(|p| p.1).sum::<f64>() / n;
    let (mut ss_res, mut ss_tot) = (0.0f64, 0.0f64);
    for &(x, y) in &data {
        ss_res += (y - fit.poly(x)).powi(2);
        ss_tot += if fixed.is_some() { (y - b0).powi(2) } else { (y - mean).powi(2) };
    }
    let r2 = (ss_tot > 0.0 && ss_tot.is_finite() && ss_res.is_finite()).then(|| 1.0 - ss_res / ss_tot);
    Some(TrendFit { r2, ..fit })
}

fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    let big = a.iter().flatten().fold(0.0f64, |m, v| m.max(v.abs()));
    if !(big.is_finite() && big > 0.0) {
        return None;
    }
    for col in 0..n {
        let piv = (col..n).max_by(|&i, &j| {
            let vi = a.get(i).and_then(|r| r.get(col)).map_or(0.0, |v| v.abs());
            let vj = a.get(j).and_then(|r| r.get(col)).map_or(0.0, |v| v.abs());
            vi.total_cmp(&vj)
        })?;
        a.swap(col, piv);
        b.swap(col, piv);
        let p = *a.get(col)?.get(col)?;
        if !p.is_finite() || p.abs() <= big * 1e-13 {
            return None;
        }
        let prow = a.get(col)?.clone();
        let pb = *b.get(col)?;
        for r in col + 1..n {
            let row = a.get_mut(r)?;
            let f = *row.get(col)? / p;
            for (v, pv) in row.iter_mut().zip(&prow).skip(col) {
                *v -= f * pv;
            }
            *b.get_mut(r)? -= f * pb;
        }
    }
    let mut x = vec![0.0f64; n];
    for r in (0..n).rev() {
        let row = a.get(r)?;
        let s: f64 = (r + 1..n).map(|c| row.get(c).copied().unwrap_or(0.0) * x.get(c).copied().unwrap_or(0.0)).sum();
        let v = (b.get(r)? - s) / row.get(r)?;
        *x.get_mut(r)? = v;
    }
    x.iter().all(|v| v.is_finite()).then_some(x)
}

fn expand(a: &[f64], c: f64, s: f64) -> Option<Vec<f64>> {
    let mut out = vec![0.0f64; a.len()];
    for (k, ak) in a.iter().enumerate() {
        let lead = ak / s.powi(k as i32);
        let mut binom = 1.0f64;
        for j in 0..=k {
            if let Some(o) = out.get_mut(j) {
                *o += lead * binom * (-c).powi((k - j) as i32);
            }
            binom = binom * (k - j) as f64 / (j + 1) as f64;
        }
    }
    out.iter().all(|v| v.is_finite()).then_some(out)
}

impl TrendFit {
    fn poly(&self, x: f64) -> f64 {
        let u = (x - self.center) / self.scale;
        self.a.iter().rev().fold(0.0, |acc, c| acc * u + c)
    }

    pub fn eval(&self, x: f64) -> Option<f64> {
        let xx = if log_x(self.kind) { if x > 0.0 { x.ln() } else { return None } } else { x };
        let p = self.poly(xx);
        let y = if log_y(self.kind) { p.exp() } else { p };
        y.is_finite().then_some(y)
    }

    pub fn equation(&self) -> String {
        let c0 = self.coef.first().copied().unwrap_or(0.0);
        let c1 = self.coef.get(1).copied().unwrap_or(0.0);
        match self.kind {
            TrendlineKind::Exponential => format!("y = {}e^({}x)", num_text(c0.exp()), num_text(c1)),
            TrendlineKind::Power => format!("y = {}x^{}", num_text(c0.exp()), num_text(c1)),
            TrendlineKind::Logarithmic => terms(&[(c1, "ln(x)"), (c0, "")]),
            _ => {
                let powers = ["", "x", "x²", "x³", "x⁴", "x⁵", "x⁶"];
                let parts: Vec<(f64, &str)> = self.coef.iter().enumerate().rev().map(|(k, c)| (*c, powers.get(k).copied().unwrap_or("x"))).collect();
                terms(&parts)
            }
        }
    }

    pub fn r_squared(&self) -> String {
        match self.r2 {
            Some(r) => format!("R² = {}", num_text(r)),
            None => "R² = #N/A".into(),
        }
    }
}

fn terms(parts: &[(f64, &str)]) -> String {
    let mut s = String::from("y = ");
    let mut first = true;
    for (c, t) in parts {
        let txt = num_text(c.abs());
        if txt == "0" {
            continue;
        }
        let neg = *c < 0.0;
        if first {
            if neg {
                s.push('-');
            }
        } else {
            s.push_str(if neg { " - " } else { " + " });
        }
        if txt != "1" || t.is_empty() {
            s.push_str(&txt);
        }
        s.push_str(t);
        first = false;
    }
    if first {
        s.push('0');
    }
    s
}

pub fn num_text(v: f64) -> String {
    if !v.is_finite() {
        return "#NUM!".into();
    }
    let a = v.abs();
    if a == 0.0 {
        return "0".into();
    }
    if !(1e-4..1e11).contains(&a) {
        let s = format!("{v:.4E}");
        let (mant, exp) = s.split_once('E').unwrap_or((s.as_str(), "0"));
        let mant = trim_zeros(mant);
        let e: i32 = exp.parse().unwrap_or(0);
        return format!("{mant}E{}{:02}", if e < 0 { '-' } else { '+' }, e.unsigned_abs());
    }
    let s = trim_zeros(&format!("{v:.4}"));
    if s == "-0" { "0".into() } else { s }
}

fn trim_zeros(s: &str) -> String {
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s.to_string() }
}

/// Calculates moving average of consecutive points over a given window period.
pub fn moving_average(pts: &[(f64, f64)], period: usize) -> Vec<(f64, f64)> {
    let p = period.clamp(2, 255);
    let pts: Vec<(f64, f64)> = pts.iter().copied().filter(|(x, y)| x.is_finite() && y.is_finite()).collect();
    let n = pts.len();
    if n < p {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n - p + 1);
    let mut running_sum: f64 = pts.iter().take(p).map(|q| q.1).sum();
    if let Some(first_pt) = pts.get(p - 1) {
        out.push((first_pt.0, running_sum / p as f64));
    }
    for i in p..n {
        let prev = pts.get(i - p).map_or(0.0, |q| q.1);
        let curr = pts.get(i).map_or(0.0, |q| q.1);
        running_sum = running_sum - prev + curr;
        if let Some(pt) = pts.get(i) {
            out.push((pt.0, running_sum / p as f64));
        }
    }
    out
}
