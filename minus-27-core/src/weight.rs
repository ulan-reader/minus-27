use serde::{Deserialize, Serialize};

/// Запись за день. Дата в формате "2026-11-22", одна запись на дату.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub date: String,
    pub weight: f64,
    pub waist: Option<f64>,
    pub steps: Option<u32>,
    /// Когда запись последний раз менялась, мс от эпохи. Ставит бэкенд.
    #[serde(default)]
    pub updated_ms: i64,
}

/// Запись с номером дня, для расчётов.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub n: i32,
    pub entry: Entry,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Week {
    /// Номер недели от первой записи, с нуля.
    pub index: i32,
    pub from: i32,
    pub to: i32,
    /// Сколько взвешиваний в неделе.
    pub n: usize,
    pub avg: f64,
    pub steps: Option<f64>,
    /// Последний замер талии за неделю.
    pub waist: Option<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level {
    Ok,
    Warn,
    Info,
}

impl Level {
    pub fn css(self) -> &'static str {
        match self {
            Level::Ok => "ok",
            Level::Warn => "warn",
            Level::Info => "info",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Level::Ok => "✅",
            Level::Warn => "⚠️",
            Level::Info => "ℹ️",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Verdict {
    pub level: Level,
    pub title: &'static str,
    pub text: &'static str,
}

/// Прогноз даты цели.
#[derive(Clone, Debug, PartialEq)]
pub enum Eta {
    Reached,
    /// День, когда при текущем темпе дойдём до цели, и сколько недель до него.
    At { day: i32, weeks: f64 },
    /// Темп около нуля или вес растёт.
    Flat,
    /// Меньше двух полноценных недель.
    NoData,
}

/// Номер дня от 1970-01-01 (алгоритм Хиннанта), как dayNum в JS.
pub fn day_num(iso: &str) -> Option<i32> {
    let b = iso.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i32 = iso[0..4].parse().ok()?;
    let m: i32 = iso[5..7].parse().ok()?;
    let d: i32 = iso[8..10].parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

/// Обратно к "2026-11-22".
pub fn iso_of(n: i32) -> String {
    let (y, m, d) = civil(n);
    format!("{y:04}-{m:02}-{d:02}")
}

/// (год, месяц 1–12, день).
pub fn civil(n: i32) -> (i32, u32, u32) {
    let z = n + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

/// Проверка записи перед сохранением. `today` — номер сегодняшнего дня, если известен.
pub fn check(e: &Entry, today: Option<i32>) -> Result<(), &'static str> {
    let n = day_num(&e.date).ok_or("Неверная дата")?;
    if today.is_some_and(|t| n > t) {
        return Err("Дата не может быть в будущем");
    }
    if !(30.0..=300.0).contains(&e.weight) {
        return Err("Вес должен быть от 30 до 300 кг");
    }
    if e.waist.is_some_and(|w| !(40.0..=250.0).contains(&w)) {
        return Err("Талия: от 40 до 250 см");
    }
    if e.steps.is_some_and(|s| s > 100_000) {
        return Err("Шаги: от 0 до 100 000");
    }
    Ok(())
}

/// Вставить или заменить запись за дату, список остаётся отсортированным.
pub fn upsert(list: &mut Vec<Entry>, e: Entry) {
    match list.binary_search_by(|x| x.date.cmp(&e.date)) {
        Ok(i) => list[i] = e,
        Err(i) => list.insert(i, e),
    }
}

/// Удалить запись за дату. `true`, если она была.
pub fn remove(list: &mut Vec<Entry>, date: &str) -> bool {
    let before = list.len();
    list.retain(|x| x.date != date);
    list.len() != before
}

/// Записи по возрастанию даты, с номерами дней. Записи с битой датой пропускаются.
pub fn points(entries: &[Entry]) -> Vec<Point> {
    let mut v: Vec<Point> =
        entries.iter().filter_map(|e| Some(Point { n: day_num(&e.date)?, entry: e.clone() })).collect();
    v.sort_by_key(|p| p.n);
    v
}

/// Скользящее среднее за 7 календарных дней для каждой точки.
pub fn rolling(list: &[Point]) -> Vec<f64> {
    (0..list.len())
        .map(|i| {
            let (mut s, mut c) = (0.0, 0.0);
            for p in list[..=i].iter().rev().take_while(|p| list[i].n - p.n < 7) {
                s += p.entry.weight;
                c += 1.0;
            }
            s / c
        })
        .collect()
}

/// Недели от первой записи. Пустые недели пропускаются.
pub fn weekly_blocks(list: &[Point]) -> Vec<Week> {
    let Some(first) = list.first() else { return Vec::new() };
    let start = first.n;
    let mut out: Vec<(i32, Vec<&Entry>)> = Vec::new();
    for p in list {
        let i = (p.n - start).div_euclid(7);
        match out.last_mut() {
            Some((j, items)) if *j == i => items.push(&p.entry),
            _ => out.push((i, vec![&p.entry])),
        }
    }
    out.into_iter()
        .map(|(i, items)| {
            let n = items.len();
            let avg = items.iter().map(|e| e.weight).sum::<f64>() / n as f64;
            let st: Vec<f64> = items.iter().filter_map(|e| e.steps).map(f64::from).collect();
            Week {
                index: i,
                from: start + i * 7,
                to: start + i * 7 + 6,
                n,
                avg,
                steps: (!st.is_empty()).then(|| st.iter().sum::<f64>() / st.len() as f64),
                waist: items.iter().rev().find_map(|e| e.waist),
            }
        })
        .collect()
}

/// Потеря в кг за неделю между соседними полноценными (от 3 взвешиваний) неделями.
pub fn week_losses(weeks: &[Week]) -> Vec<f64> {
    let rel: Vec<&Week> = weeks.iter().filter(|w| w.n >= 3).collect();
    rel.windows(2).map(|p| (p[0].avg - p[1].avg) / (p[1].index - p[0].index) as f64).collect()
}

/// Средний темп за последние до трёх недель, кг/нед.
pub fn pace(losses: &[f64]) -> Option<f64> {
    if losses.is_empty() {
        return None;
    }
    let l = &losses[losses.len().saturating_sub(3)..];
    Some(l.iter().sum::<f64>() / l.len() as f64)
}

pub fn verdict(weeks: &[Week]) -> Verdict {
    let loss = week_losses(weeks);
    let Some(&last) = loss.last() else {
        return Verdict {
            level: Level::Info,
            title: "Пока мало данных",
            text: "Сравнение начнётся, когда будет две недели минимум по 3 взвешивания. Первые 1–2 недели вес падает в основном за счёт воды.",
        };
    };
    let tail = |k: usize| &loss[loss.len().saturating_sub(k)..];
    let (l2, l3) = (tail(2), tail(3));
    if l2.len() == 2 && l2.iter().all(|&x| x < 0.4) {
        return Verdict {
            level: Level::Warn,
            title: "Медленно две недели подряд",
            text: "Минус ~150–200 ккал (масло, гарнир, перекус) или +2 000 шагов в день.",
        };
    }
    if l3.len() == 3 && l3.iter().all(|&x| x > 1.2) {
        return Verdict {
            level: Level::Warn,
            title: "Слишком быстро три недели подряд",
            text: "Добавь ~150–200 ккал, иначе уйдут мышцы.",
        };
    }
    if last < 0.4 {
        return Verdict {
            level: Level::Info,
            title: "Эта неделя медленная",
            text: "Одна неделя ничего не значит. Ничего не меняй, смотри следующую.",
        };
    }
    if last > 1.2 {
        return Verdict {
            level: Level::Info,
            title: "Быстро. В начале это в основном вода",
            text: "Если так будет три недели подряд, добавь ~150–200 ккал.",
        };
    }
    Verdict { level: Level::Ok, title: "Темп хороший", text: "Ничего не меняй." }
}

/// Когда дойдём до цели. `last_n` — день последней записи, `current` — текущее среднее.
pub fn eta(last_n: i32, current: f64, goal: f64, pace: Option<f64>) -> Eta {
    let left = current - goal;
    if left <= 0.0 {
        return Eta::Reached;
    }
    match pace {
        Some(p) if p > 0.05 => {
            let weeks = left / p;
            Eta::At { day: last_n + (weeks * 7.0).round() as i32, weeks }
        }
        Some(_) => Eta::Flat,
        None => Eta::NoData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(date: &str, weight: f64) -> Entry {
        Entry { date: date.into(), weight, waist: None, steps: None, updated_ms: 0 }
    }

    /// Ежедневные записи с шагом `step` кг в день, начиная с 2026-01-01.
    fn daily(days: i32, start: f64, step: f64) -> Vec<Entry> {
        let n0 = day_num("2026-01-01").unwrap();
        (0..days).map(|i| e(&iso_of(n0 + i), start - step * i as f64)).collect()
    }

    #[test]
    fn dates_roundtrip() {
        assert_eq!(day_num("1970-01-01"), Some(0));
        assert_eq!(day_num("2000-03-01"), Some(11017));
        assert_eq!(iso_of(11017), "2000-03-01");
        for n in [-1000, 0, 365, 10957, 20000, 20785] {
            assert_eq!(day_num(&iso_of(n)), Some(n));
        }
        assert_eq!(day_num("2026-13-01"), None);
        assert_eq!(day_num("2026-1-01"), None);
    }

    #[test]
    fn upsert_keeps_order() {
        let mut v = vec![];
        upsert(&mut v, e("2026-01-03", 90.0));
        upsert(&mut v, e("2026-01-01", 92.0));
        upsert(&mut v, e("2026-01-03", 89.0));
        assert_eq!(v.iter().map(|x| x.date.as_str()).collect::<Vec<_>>(), ["2026-01-01", "2026-01-03"]);
        assert_eq!(v[1].weight, 89.0);
        assert!(remove(&mut v, "2026-01-01"));
        assert!(!remove(&mut v, "2026-01-01"));
    }

    #[test]
    fn rolling_uses_seven_calendar_days() {
        let list = points(&[e("2026-01-01", 100.0), e("2026-01-07", 98.0), e("2026-01-08", 96.0)]);
        assert_eq!(rolling(&list), [100.0, 99.0, 97.0]);
    }

    #[test]
    fn weeks_and_pace() {
        // −0,1 кг в день = −0,7 кг в неделю
        let list = points(&daily(28, 100.0, 0.1));
        let weeks = weekly_blocks(&list);
        assert_eq!(weeks.len(), 4);
        assert_eq!(weeks[0].n, 7);
        let p = pace(&week_losses(&weeks)).unwrap();
        assert!((p - 0.7).abs() < 1e-9, "{p}");
        assert_eq!(verdict(&weeks).level, Level::Ok);
    }

    #[test]
    fn verdicts() {
        assert_eq!(verdict(&[]).title, "Пока мало данных");
        let slow = weekly_blocks(&points(&daily(21, 100.0, 0.02)));
        assert_eq!(verdict(&slow).title, "Медленно две недели подряд");
        let fast = weekly_blocks(&points(&daily(28, 100.0, 0.2)));
        assert_eq!(verdict(&fast).title, "Слишком быстро три недели подряд");
    }

    #[test]
    fn short_weeks_are_ignored() {
        let mut v = daily(7, 100.0, 0.1);
        v.push(e("2026-01-10", 99.0)); // во второй неделе одно взвешивание
        let weeks = weekly_blocks(&points(&v));
        assert_eq!(weeks.len(), 2);
        assert!(week_losses(&weeks).is_empty());
    }

    #[test]
    fn eta_cases() {
        assert_eq!(eta(0, 69.0, 70.0, None), Eta::Reached);
        assert_eq!(eta(0, 80.0, 70.0, None), Eta::NoData);
        assert_eq!(eta(0, 80.0, 70.0, Some(0.0)), Eta::Flat);
        assert_eq!(eta(100, 80.0, 70.0, Some(1.0)), Eta::At { day: 170, weeks: 10.0 });
    }

    #[test]
    fn checks() {
        let today = day_num("2026-11-23");
        assert!(check(&e("2026-11-23", 97.0), today).is_ok());
        assert_eq!(check(&e("2026-11-24", 97.0), today), Err("Дата не может быть в будущем"));
        assert_eq!(check(&e("2026-11-23", 20.0), today), Err("Вес должен быть от 30 до 300 кг"));
        let mut x = e("2026-11-23", 97.0);
        x.waist = Some(30.0);
        assert!(check(&x, today).is_err());
    }
}
