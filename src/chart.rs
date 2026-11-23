//! График веса строкой SVG: точки по дням, линия среднего за 7 дней, пунктир цели.

use std::fmt::Write;

use minus27_core::weight::Point;

use crate::fmt;

/// `None`, если записей меньше двух.
pub fn svg(list: &[Point], roll: &[f64], goal: f64) -> Option<String> {
    if list.len() < 2 {
        return None;
    }
    let (w, h, pl, pr, pt, pb) = (720.0, 260.0, 46.0, 14.0, 14.0, 28.0);
    let ws = list.iter().map(|p| p.entry.weight);
    let mut lo = ws.clone().fold(f64::INFINITY, f64::min);
    let mut hi = ws.fold(f64::NEG_INFINITY, f64::max);
    let show_goal = goal >= lo - 8.0 && goal <= hi + 8.0;
    if show_goal {
        lo = lo.min(goal);
        hi = hi.max(goal);
    }
    lo = (lo - 0.5).floor();
    hi = (hi + 0.5).ceil();
    let x0 = list[0].n;
    let last = list[list.len() - 1].n;
    let x1 = last.max(x0 + 6);
    let x = |n: i32| pl + (n - x0) as f64 / (x1 - x0) as f64 * (w - pl - pr);
    let y = |v: f64| pt + (hi - v) / (hi - lo) * (h - pt - pb);

    let mut s = format!(r#"<svg viewBox="0 0 {w} {h}" class="chart" role="img" aria-label="График веса">"#);
    for i in 0..=4 {
        let v = lo + (hi - lo) * i as f64 / 4.0;
        let _ = write!(
            s,
            r#"<line x1="{pl}" x2="{}" y1="{y:.1}" y2="{y:.1}" class="grid"/><text x="{}" y="{:.1}" class="ax" text-anchor="end">{v:.1}</text>"#,
            w - pr,
            pl - 6.0,
            y(v) + 4.0,
            y = y(v),
        );
    }
    if show_goal {
        let _ = write!(s, r#"<line x1="{pl}" x2="{}" y1="{g:.1}" y2="{g:.1}" class="goal"/>"#, w - pr, g = y(goal));
    }
    for p in list {
        let _ = write!(s, r#"<circle cx="{:.1}" cy="{:.1}" r="3.5" class="dot"/>"#, x(p.n), y(p.entry.weight));
    }
    let pts: Vec<String> = list.iter().zip(roll).map(|(p, r)| format!("{:.1},{:.1}", x(p.n), y(*r))).collect();
    let _ = write!(s, r#"<polyline class="avg" points="{}"/>"#, pts.join(" "));
    let _ = write!(
        s,
        r#"<text x="{pl}" y="{}" class="ax">{}</text><text x="{}" y="{}" class="ax" text-anchor="end">{}</text></svg>"#,
        h - 8.0,
        fmt::day(x0),
        w - pr,
        h - 8.0,
        fmt::day(last),
    );
    Some(s)
}
