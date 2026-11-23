use serde::{Deserialize, Serialize};

use crate::foods::FoodId::{self, *};

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
}
