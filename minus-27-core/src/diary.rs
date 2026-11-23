//! Дневник за день: что съедено и сколько выпито воды.

use serde::{Deserialize, Serialize};

use crate::{
    foods::CustomFood,
    plan::{MealKind, PlanMeal, Totals, view_totals},
};

/// Откуда запись.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Отмечен приём пищи из плана.
    Plan,
    #[default]
    Manual,
    /// Распознано по фото.
    Photo,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiaryItem {
    pub id: String,
    pub meal: MealKind,
    pub name: String,
    pub grams: Option<u32>,
    pub kcal: f64,
    pub protein: f64,
    #[serde(default)]
    pub source: Source,
}

/// Один день. Синхронизируется целиком, одна строка на дату.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DayLog {
    pub date: String,
    pub items: Vec<DiaryItem>,
    pub water_ml: u32,
    pub updated_ms: i64,
}

impl DayLog {
    pub fn new(date: &str) -> Self {
        Self { date: date.to_string(), ..Self::default() }
    }

    pub fn totals(&self) -> Totals {
        let mut t = Totals::default();
        for it in &self.items {
            t += Totals { kcal: it.kcal, protein: it.protein };
        }
        t
    }

    pub fn meal_totals(&self, meal: MealKind) -> Totals {
        let mut t = Totals::default();
        for it in self.items.iter().filter(|i| i.meal == meal) {
            t += Totals { kcal: it.kcal, protein: it.protein };
        }
        t
    }

    /// Отмечен ли приём пищи из плана.
    pub fn plan_done(&self, meal: MealKind) -> bool {
        self.items.iter().any(|i| i.meal == meal && i.source == Source::Plan)
    }

    /// Отметить приём из плана съеденным: все его продукты попадают в дневник.
    /// `id` — уникальная приставка (например, время в мс), к ней добавляется номер.
    pub fn eat_plan_meal(&mut self, meal: &PlanMeal, customs: &[CustomFood], id: &str) {
        self.unmark_plan_meal(meal.kind);
        for (n, it) in meal.items.iter().enumerate() {
            let Some(v) = it.food.view(customs) else { continue };
            let t = view_totals(&v, it.grams);
            self.items.push(DiaryItem {
                id: format!("{id}-{n}"),
                meal: meal.kind,
                name: v.name,
                grams: Some(it.grams),
                kcal: t.kcal,
                protein: t.protein,
                source: Source::Plan,
            });
        }
    }

    pub fn unmark_plan_meal(&mut self, meal: MealKind) {
        self.items.retain(|i| !(i.meal == meal && i.source == Source::Plan));
    }

    pub fn remove(&mut self, id: &str) {
        self.items.retain(|i| i.id != id);
    }

    pub fn add_water(&mut self, ml: i32) {
        self.water_ml = (self.water_ml as i32 + ml).clamp(0, 10_000) as u32;
    }
}

/// Найти день или создать пустой. Список отсортирован по дате.
pub fn day_mut<'a>(days: &'a mut Vec<DayLog>, date: &str) -> &'a mut DayLog {
    let i = match days.binary_search_by(|d| d.date.as_str().cmp(date)) {
        Ok(i) => i,
        Err(i) => {
            days.insert(i, DayLog::new(date));
            i
        }
    };
    &mut days[i]
}

/// Вставить или заменить день целиком (для синка).
pub fn upsert(days: &mut Vec<DayLog>, d: DayLog) {
    match days.binary_search_by(|x| x.date.cmp(&d.date)) {
        Ok(i) => days[i] = d,
        Err(i) => days.insert(i, d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{Mode, base_plan};

    #[test]
    fn eat_plan_meal_and_undo() {
        let plan = base_plan(Mode::Cut);
        let mut d = DayLog::new("2026-11-23");
        d.eat_plan_meal(&plan[0][0], &[], "a");
        let t = d.totals();
        let expected = crate::plan::items_totals(&plan[0][0].items, &[]);
        assert!((t.kcal - expected.kcal).abs() < 1e-9);
        assert!(d.plan_done(MealKind::Breakfast));
        // повторная отметка не удваивает
        d.eat_plan_meal(&plan[0][0], &[], "b");
        assert!((d.totals().kcal - expected.kcal).abs() < 1e-9);
        d.unmark_plan_meal(MealKind::Breakfast);
        assert!(d.items.is_empty());
    }

    #[test]
    fn water_and_days() {
        let mut days = Vec::new();
        day_mut(&mut days, "2026-11-23").add_water(250);
        day_mut(&mut days, "2026-11-21").add_water(500);
        day_mut(&mut days, "2026-11-23").add_water(-1000);
        assert_eq!(days.iter().map(|d| (d.date.as_str(), d.water_ml)).collect::<Vec<_>>(), [
            ("2026-11-21", 500),
            ("2026-11-23", 0)
        ]);
    }
}
