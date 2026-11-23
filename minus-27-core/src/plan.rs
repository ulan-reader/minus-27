use serde::{Deserialize, Serialize};

use crate::foods::{
    CustomFood, FoodRef, FoodView,
    FoodId::{self, *},
};

/// Цель по белку в день, г.
pub const TARGET_PROTEIN: f64 = 150.0;

/// «Оригинал ChatGPT» или «минус ~1 кг/нед».
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Orig,
    #[default]
    Cut,
}

/// Продукт в приёме пищи: граммы в оригинале и в режиме «минус» (`Some(0)` = убрано).
#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub food: FoodId,
    pub grams: u16,
    pub cut: Option<u16>,
}

#[derive(Debug)]
pub struct Meal {
    pub title: &'static str,
    pub emoji: &'static str,
    pub items: &'static [Item],
}

pub type Day = [Meal; 4];

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Totals {
    pub kcal: f64,
    pub protein: f64,
}

impl std::ops::AddAssign for Totals {
    fn add_assign(&mut self, o: Self) {
        self.kcal += o.kcal;
        self.protein += o.protein;
    }
}

const fn i(food: FoodId, grams: u16) -> Item {
    Item { food, grams, cut: None }
}

const fn c(food: FoodId, grams: u16, cut: u16) -> Item {
    Item { food, grams, cut: Some(cut) }
}

/// Приём пищи литералом структуры: так `&[...]` живёт столько же, сколько `PLAN`.
macro_rules! meal {
    ($title:literal, $emoji:literal, [$($it:expr),* $(,)?]) => {
        Meal { title: $title, emoji: $emoji, items: &[$($it),*] }
    };
}

pub static PLAN: [Day; 7] = [
    [
        meal!("Завтрак", "🍳", [i(Egg, 100), i(White, 150), i(Oats, 60), i(Berry, 100), c(Banana, 100, 0)]),
        meal!("Обед", "🍗", [i(Chicken, 200), c(Rice, 80, 60), i(Veg, 200), c(Oil, 10, 5)]),
        meal!("Ужин", "🐟", [i(Salmon, 180), i(Potato, 250), i(Broccoli, 200), i(Gyog, 150)]),
        meal!("Перекус", "🍎", [i(Apple, 180), c(Nuts, 15, 0)]),
    ],
    [
        meal!("Завтрак", "🥣", [i(Curd, 250), c(Nuts, 15, 0), c(Banana, 120, 0), i(Berry, 80)]),
        meal!("Обед", "🦃", [i(Turkey, 200), c(Buck, 80, 60), i(Veg, 200), c(Oil, 10, 5)]),
        meal!("Ужин", "🍳", [i(Egg, 150), i(White, 100), i(Avocado, 70), i(Veg, 200), c(Bread, 80, 60)]),
        meal!("Перекус", "🥛", [i(Kefir, 250), i(Apple, 150)]),
    ],
    [
        meal!("Завтрак", "🥣", [i(Oats, 70), i(Protein, 30), i(Pb, 15), i(Milk, 200), i(Berry, 50)]),
        meal!("Обед", "🥩", [i(Beef, 180), i(Potato, 300), i(Veg, 200), i(Oil, 5)]),
        meal!("Ужин", "🐟", [i(Fish, 220), c(Quinoa, 70, 60), i(Veg, 200), i(Gyog, 100)]),
        meal!("Перекус", "🧀", [i(Curd, 150), i(Kiwi, 100)]),
    ],
    [
        meal!("Завтрак", "🥣", [i(Gyog, 300), i(Chia, 15), i(Oats, 40), c(Banana, 100, 0)]),
        meal!("Обед", "🍗", [i(Chicken, 200), c(Bulgur, 80, 60), i(Veg, 200), c(Oil, 10, 5)]),
        meal!("Ужин", "🍤", [i(Shrimp, 220), i(Avocado, 70), i(Veg, 250), c(Bread, 80, 60)]),
        meal!("Перекус", "🥛", [i(Kefir, 250), i(Orange, 150), c(Nuts, 15, 0)]),
    ],
    [
        meal!("Завтрак", "🍳", [i(Egg, 150), i(White, 100), c(Bread, 80, 60), i(Veg, 200), i(Cheese, 30)]),
        meal!("Обед", "🦃", [i(Turkey, 200), c(Rice, 80, 60), i(Veg, 200), i(Oil, 5)]),
        meal!("Ужин", "🥣", [i(Curd, 250), c(Nuts, 15, 0), i(Berry, 100), i(Bread, 60)]),
        meal!("Перекус", "🍌", [i(Banana, 120), i(Kefir, 250)]),
    ],
    [
        meal!("Завтрак", "🥣", [i(Oats, 60), i(Protein, 30), i(Milk, 200), c(Banana, 100, 0)]),
        meal!("Обед", "🐟", [i(Salmon, 150), c(Quinoa, 70, 60), i(Veg, 200)]),
        meal!("Ужин", "🍳", [i(Egg, 100), i(White, 200), i(Potato, 250), i(Veg, 200), i(Cheese, 20), i(Oil, 5)]),
        meal!("Перекус", "🥛", [i(Gyog, 200), i(Apple, 180), c(Almond, 15, 0)]),
    ],
    [
        meal!("Завтрак", "🥣", [i(Curd, 250), i(Berry, 100), c(Honey, 10, 0), i(Bread, 60)]),
        meal!("Обед", "🥩", [i(Beef, 180), c(Buck, 80, 60), i(Veg, 200), i(Oil, 5)]),
        meal!("Ужин", "🐟", [i(Fish, 220), i(Potato, 250), i(Veg, 200), i(Gyog, 100)]),
        meal!("Перекус", "🍌", [i(Banana, 100), c(Nuts, 15, 0), i(Kefir, 250)]),
    ],
];

pub fn grams_of(it: &Item, mode: Mode) -> u16 {
    match (mode, it.cut) {
        (Mode::Cut, Some(g)) => g,
        _ => it.grams,
    }
}

/// Изменён ли продукт в режиме «минус».
pub fn is_changed(it: &Item, mode: Mode) -> bool {
    mode == Mode::Cut && it.cut.is_some()
}

pub fn item_totals(food: FoodId, grams: u16) -> Totals {
    let f = food.info();
    let g = grams as f64 / 100.0;
    Totals { kcal: f.kcal * g, protein: f.protein * g }
}

pub fn meal_totals(meal: &Meal, mode: Mode) -> Totals {
    let mut t = Totals::default();
    for it in meal.items {
        t += item_totals(it.food, grams_of(it, mode));
    }
    t
}

pub fn day_totals(day: usize, mode: Mode) -> Totals {
    let mut t = Totals::default();
    for m in &PLAN[day] {
        t += meal_totals(m, mode);
    }
    t
}

/// Среднее за день по всей неделе.
pub fn week_average(mode: Mode) -> Totals {
    let mut t = Totals::default();
    for d in 0..PLAN.len() {
        t += day_totals(d, mode);
    }
    let n = PLAN.len() as f64;
    Totals { kcal: t.kcal / n, protein: t.protein / n }
}

/// Сколько чего купить на выбранные дни, в порядке первого появления в плане.
pub fn shopping_list(days: &[u8], mode: Mode) -> Vec<(FoodId, u32)> {
    let mut acc: Vec<(FoodId, u32)> = Vec::new();
    for &d in days {
        let Some(day) = PLAN.get(d as usize) else { continue };
        for m in day {
            for it in m.items {
                let g = grams_of(it, mode) as u32;
                if g == 0 {
                    continue;
                }
                match acc.iter_mut().find(|(f, _)| *f == it.food) {
                    Some((_, sum)) => *sum += g,
                    None => acc.push((it.food, g)),
                }
            }
        }
    }
    acc
}

// ---------- План пользователя ----------
// Встроенный PLAN — шаблон. Человек видит его копию, масштабированную под свою норму,
// а после первой правки — свой план (`Settings::custom_plan`).

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MealKind {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
}

impl MealKind {
    pub const ALL: [MealKind; 4] = [MealKind::Breakfast, MealKind::Lunch, MealKind::Dinner, MealKind::Snack];

    pub fn title(self) -> &'static str {
        match self {
            MealKind::Breakfast => "Завтрак",
            MealKind::Lunch => "Обед",
            MealKind::Dinner => "Ужин",
            MealKind::Snack => "Перекус",
        }
    }

    /// Имя иконки в UI (src/icons.rs).
    pub fn icon(self) -> &'static str {
        match self {
            MealKind::Breakfast => "sunrise",
            MealKind::Lunch => "sun",
            MealKind::Dinner => "moon",
            MealKind::Snack => "apple",
        }
    }

    /// Обычное время приёма, часы.
    pub fn hour(self) -> u32 {
        match self {
            MealKind::Breakfast => 8,
            MealKind::Lunch => 13,
            MealKind::Snack => 16,
            MealKind::Dinner => 19,
        }
    }

    /// Какой приём сейчас по времени.
    pub fn at_hour(h: u32) -> MealKind {
        match h {
            0..=10 => MealKind::Breakfast,
            11..=14 => MealKind::Lunch,
            15..=17 => MealKind::Snack,
            _ => MealKind::Dinner,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanItem {
    pub food: FoodRef,
    pub grams: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanMeal {
    pub kind: MealKind,
    pub items: Vec<PlanItem>,
}

pub type PlanDay = Vec<PlanMeal>;

const KINDS: [MealKind; 4] = [MealKind::Breakfast, MealKind::Lunch, MealKind::Dinner, MealKind::Snack];

/// Встроенное меню как данные (убранные в режиме продукты выпадают).
pub fn base_plan(mode: Mode) -> Vec<PlanDay> {
    PLAN.iter()
        .map(|day| {
            day.iter()
                .zip(KINDS)
                .map(|(m, kind)| PlanMeal {
                    kind,
                    items: m
                        .items
                        .iter()
                        .map(|it| PlanItem { food: it.food.into(), grams: grams_of(it, mode) as u32 })
                        .filter(|it| it.grams > 0)
                        .collect(),
                })
                .collect()
        })
        .collect()
}

/// Встроенное меню «минус», масштабированное под норму калорий.
/// Порции округляются до 5 г, масштаб ограничен 0,6–1,6, чтобы меню оставалось разумным.
pub fn scaled_plan(target_kcal: f64) -> Vec<PlanDay> {
    let base = week_average(Mode::Cut).kcal;
    let k = (target_kcal / base).clamp(0.6, 1.6);
    let mut plan = base_plan(Mode::Cut);
    for day in &mut plan {
        for meal in day {
            for it in &mut meal.items {
                // Яйца — целыми штуками, остальное с шагом 5 г.
                let step = match &it.food {
                    FoodRef::Base(id) => id.info().piece.filter(|p| p.label == "шт").map_or(5.0, |p| p.grams as f64),
                    FoodRef::Custom { .. } => 5.0,
                };
                it.grams = ((it.grams as f64 * k / step).round() * step).max(step) as u32;
            }
        }
    }
    plan
}

pub fn items_totals(items: &[PlanItem], customs: &[CustomFood]) -> Totals {
    let mut t = Totals::default();
    for it in items {
        if let Some(v) = it.food.view(customs) {
            t += view_totals(&v, it.grams);
        }
    }
    t
}

pub fn view_totals(v: &FoodView, grams: u32) -> Totals {
    let g = grams as f64 / 100.0;
    Totals { kcal: v.kcal * g, protein: v.protein * g }
}

pub fn plan_day_totals(day: &PlanDay, customs: &[CustomFood]) -> Totals {
    let mut t = Totals::default();
    for m in day {
        t += items_totals(&m.items, customs);
    }
    t
}

pub fn plan_week_average(plan: &[PlanDay], customs: &[CustomFood]) -> Totals {
    if plan.is_empty() {
        return Totals::default();
    }
    let mut t = Totals::default();
    for d in plan {
        t += plan_day_totals(d, customs);
    }
    let n = plan.len() as f64;
    Totals { kcal: t.kcal / n, protein: t.protein / n }
}

/// Покупки на выбранные дни плана, в порядке первого появления.
pub fn plan_shopping_list(plan: &[PlanDay], days: &[u8]) -> Vec<(FoodRef, u32)> {
    let mut acc: Vec<(FoodRef, u32)> = Vec::new();
    for &d in days {
        let Some(day) = plan.get(d as usize) else { continue };
        for it in day.iter().flat_map(|m| &m.items) {
            match acc.iter_mut().find(|(f, _)| *f == it.food) {
                Some((_, sum)) => *sum += it.grams,
                None => acc.push((it.food.clone(), it.grams)),
            }
        }
    }
    acc
}

/// Сколько граммов нового продукта дают те же калории. Округление до 5 г.
pub fn swap_grams(from: &FoodView, grams: u32, to: &FoodView) -> u32 {
    if to.kcal <= 0.0 {
        return grams;
    }
    let g = from.kcal * grams as f64 / to.kcal;
    ((g / 5.0).round() * 5.0).clamp(5.0, 2000.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_average_matches_js() {
        let o = week_average(Mode::Orig);
        assert!((o.kcal - 2050.229).abs() < 0.01, "{o:?}");
        assert!((o.protein - 160.814).abs() < 0.01, "{o:?}");
        let c = week_average(Mode::Cut);
        assert!((c.kcal - 1815.686).abs() < 0.01, "{c:?}");
        assert!((c.protein - 155.351).abs() < 0.01, "{c:?}");
    }

    #[test]
    fn day_totals_match_js() {
        let orig: Vec<_> = (0..7).map(|d| day_totals(d, Mode::Orig).kcal.round() as i32).collect();
        assert_eq!(orig, [2180, 2027, 1961, 2070, 2065, 2095, 1955]);
        let cut: Vec<_> = (0..7).map(|d| day_totals(d, Mode::Cut).kcal.round() as i32).collect();
        assert_eq!(cut, [1887, 1668, 1924, 1728, 1855, 1882, 1766]);
    }

    #[test]
    fn shopping_list_whole_week() {
        let all: Vec<u8> = (0..7).collect();
        let list = shopping_list(&all, Mode::Cut);
        let get = |f| list.iter().find(|(x, _)| *x == f).map(|(_, g)| *g);
        assert_eq!(list[0], (Egg, 500));
        assert_eq!(get(Veg), Some(2650));
        assert_eq!(get(Banana), Some(220));
        assert_eq!(get(Nuts), None);
        assert_eq!(get(Honey), None);
        assert_eq!(list.len(), 32);
    }

    #[test]
    fn base_plan_matches_static_plan() {
        let p = base_plan(Mode::Cut);
        let avg = plan_week_average(&p, &[]);
        assert!((avg.kcal - week_average(Mode::Cut).kcal).abs() < 1e-9);
        // убранные в «минусе» продукты выпали
        assert!(p.iter().flatten().flat_map(|m| &m.items).all(|it| it.grams > 0));
        assert_eq!(p[0][0].kind, MealKind::Breakfast);
    }

    #[test]
    fn scaled_plan_hits_target() {
        for target in [1600.0, 2090.0, 2400.0] {
            let avg = plan_week_average(&scaled_plan(target), &[]).kcal;
            assert!((avg - target).abs() / target < 0.03, "{target}: {avg}");
        }
    }

    #[test]
    fn scaled_eggs_are_whole() {
        let p = scaled_plan(2090.0);
        let eggs = p.iter().flatten().flat_map(|m| &m.items).filter(|i| i.food == FoodRef::Base(Egg));
        for e in eggs {
            assert_eq!(e.grams % 50, 0, "{}", e.grams);
        }
    }

    #[test]
    fn swap_keeps_kcal() {
        let rice = FoodRef::Base(Rice).view(&[]).unwrap();
        let buck = FoodRef::Base(Buck).view(&[]).unwrap();
        assert_eq!(swap_grams(&rice, 60, &buck), 60); // 350 vs 340 ккал → ~62 → 60
        let chicken = FoodRef::Base(Chicken).view(&[]).unwrap();
        let salmon = FoodRef::Base(Salmon).view(&[]).unwrap();
        assert_eq!(swap_grams(&chicken, 200, &salmon), 115);
    }

    #[test]
    fn plan_shopping_matches_static() {
        let all: Vec<u8> = (0..7).collect();
        let new = plan_shopping_list(&base_plan(Mode::Cut), &all);
        let old = shopping_list(&all, Mode::Cut);
        assert_eq!(new.len(), old.len());
        for ((a, ga), (b, gb)) in new.iter().zip(&old) {
            assert_eq!((a, ga), (&FoodRef::Base(*b), gb));
        }
    }
}
