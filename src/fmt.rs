//! Форматирование чисел и дат по-русски.

use minus27_core::weight::{civil, iso_of};

/// 97,3
pub fn f1(n: f64) -> String {
    format!("{:.1}", n).replace('.', ",")
}

/// +0,4 / −0,4 / 0,0
pub fn signed(n: f64) -> String {
    let sign = if n >= 0.05 { "+" } else if n <= -0.05 { "−" } else { "" };
    format!("{sign}{}", f1(n.abs()))
}

/// 12 345
pub fn thousands(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('\u{a0}');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// "2026-11-23" → "23.11.26"
pub fn date(iso: &str) -> String {
    format!("{}.{}.{}", &iso[8..10], &iso[5..7], &iso[2..4])
}

/// Номер дня → "23.11"
pub fn day(n: i32) -> String {
    let i = iso_of(n);
    format!("{}.{}", &i[8..10], &i[5..7])
}

/// Номер дня → "ноябрь 2027"
pub fn month_year(n: i32) -> String {
    const M: [&str; 12] = [
        "январь", "февраль", "март", "апрель", "май", "июнь", "июль", "август", "сентябрь", "октябрь", "ноябрь", "декабрь",
    ];
    let (y, m, _) = civil(n);
    format!("{} {y}", M[m as usize - 1])
}

/// Класс для разницы: вниз зелёным, вверх красным.
pub fn delta_class(d: Option<f64>) -> &'static str {
    match d {
        Some(x) if x < 0.0 => "dn",
        Some(x) if x > 0.0 => "up",
        _ => "",
    }
}
