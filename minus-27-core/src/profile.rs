//! Профиль человека и его дневные нормы: калории, белок, вода.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sex {
    #[default]
    Male,
    Female,
}

/// Активность вне тренировок и сами тренировки.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    /// Сидячая работа, мало шагов.
    #[default]
    Sedentary,
    /// 1–3 тренировки в неделю или 7–10 тыс. шагов.
    Light,
    /// 3–5 тренировок.
    Moderate,
    /// Почти каждый день.
    High,
}

impl Activity {
    pub const ALL: [Activity; 4] = [Activity::Sedentary, Activity::Light, Activity::Moderate, Activity::High];

    pub fn factor(self) -> f64 {
        match self {
            Activity::Sedentary => 1.2,
            Activity::Light => 1.375,
            Activity::Moderate => 1.55,
            Activity::High => 1.725,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Activity::Sedentary => "Сидячий образ жизни",
            Activity::Light => "1–3 тренировки в неделю",
            Activity::Moderate => "3–5 тренировок",
            Activity::High => "Почти каждый день",
        }
    }
}

/// Желаемый темп похудения.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pace {
    Slow,
    #[default]
    Normal,
    Fast,
}

impl Pace {
    pub const ALL: [Pace; 3] = [Pace::Slow, Pace::Normal, Pace::Fast];

    /// кг в неделю
    pub fn kg_per_week(self) -> f64 {
        match self {
            Pace::Slow => 0.25,
            Pace::Normal => 0.5,
            Pace::Fast => 0.75,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Pace::Slow => "Мягко · 0,25 кг/нед",
            Pace::Normal => "Обычно · 0,5 кг/нед",
            Pace::Fast => "Быстро · 0,75 кг/нед",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub name: String,
    pub sex: Sex,
    pub birth_year: i32,
    pub height_cm: f64,
    pub start_weight: f64,
    pub goal_weight: f64,
    pub activity: Activity,
    pub pace: Pace,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: String::new(),
            sex: Sex::Male,
            birth_year: 2004,
            height_cm: 169.0,
            start_weight: 97.0,
            goal_weight: 70.0,
            activity: Activity::Light,
            pace: Pace::Normal,
        }
    }
}

/// Дневные нормы.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Targets {
    /// Основной обмен (Миффлин — Сан Жеор).
    pub bmr: f64,
    /// Расход с учётом активности.
    pub tdee: f64,
    pub kcal: f64,
    pub protein: f64,
    pub water_ml: u32,
}

/// Калорий в килограмме жира, примерно.
const KCAL_PER_KG: f64 = 7700.0;

impl Profile {
    pub fn check(&self, year: i32) -> Result<(), &'static str> {
        let age = year - self.birth_year;
        if !(14..=100).contains(&age) {
            return Err("Возраст: от 14 до 100 лет");
        }
        if !(120.0..=230.0).contains(&self.height_cm) {
            return Err("Рост: от 120 до 230 см");
        }
        if !(30.0..=300.0).contains(&self.start_weight) || !(30.0..=300.0).contains(&self.goal_weight) {
            return Err("Вес: от 30 до 300 кг");
        }
        Ok(())
    }

    /// Нормы на сегодня. `weight` — текущий вес (среднее за неделю), `year` — текущий год.
    pub fn targets(&self, weight: f64, year: i32) -> Targets {
        let age = (year - self.birth_year).max(14) as f64;
        let sex = if self.sex == Sex::Male { 5.0 } else { -161.0 };
        let bmr = 10.0 * weight + 6.25 * self.height_cm - 5.0 * age + sex;
        let tdee = bmr * self.activity.factor();
        let losing = weight > self.goal_weight + 0.5;
        // Дефицит под выбранный темп, но не больше 25% расхода.
        let deficit = if losing { (self.pace.kg_per_week() * KCAL_PER_KG / 7.0).min(tdee * 0.25) } else { 0.0 };
        let floor = if self.sex == Sex::Male { 1500.0 } else { 1200.0 };
        let kcal = round_to(tdee - deficit, 10.0).max(floor);
        // Белок: 1,6 г на кг текущего веса, но не больше 2,2 г на кг цели. Держит мышцы в дефиците.
        let protein = (1.6 * weight).min(2.2 * self.goal_weight).max(60.0).round();
        let water_ml = (round_to(30.0 * weight, 250.0) as u32).clamp(1500, 4000);
        Targets { bmr, tdee, kcal, protein, water_ml }
    }
}

fn round_to(v: f64, step: f64) -> f64 {
    (v / step).round() * step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_for_default_profile() {
        // 22 года, 169 см, 97 кг, мужчина, 1–3 тренировки, 0,5 кг/нед.
        let p = Profile { birth_year: 2004, ..Profile::default() };
        let t = p.targets(97.0, 2026);
        assert!((t.bmr - 1921.25).abs() < 0.01, "{t:?}");
        // 1921,25 × 1,375 − 550 ≈ 2092 → 2090
        assert_eq!(t.kcal, 2090.0);
        assert_eq!(t.protein, 154.0);
        assert_eq!(t.water_ml, 3000);
    }

    #[test]
    fn no_deficit_at_goal_and_floor() {
        let p = Profile { goal_weight: 70.0, ..Profile::default() };
        let t = p.targets(70.0, 2026);
        assert_eq!(t.kcal, round_to(t.tdee, 10.0));
        // Маленький расход и быстрый темп: упираемся в нижний порог 1200 ккал.
        let small = Profile {
            sex: Sex::Female,
            height_cm: 150.0,
            birth_year: 1960,
            goal_weight: 45.0,
            pace: Pace::Fast,
            ..Profile::default()
        };
        assert_eq!(small.targets(50.0, 2026).kcal, 1200.0);
    }

    #[test]
    fn checks() {
        assert!(Profile::default().check(2026).is_ok());
        assert!(Profile { height_cm: 300.0, ..Profile::default() }.check(2026).is_err());
    }
}
