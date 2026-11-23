use serde::{Deserialize, Serialize};

use crate::{
    foods::{CustomFood, FoodRef},
    plan::{Mode, PlanDay, scaled_plan},
    profile::{Profile, Targets},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tab {
    #[default]
    Today,
    Plan,
    Shop,
    /// Раньше вкладка называлась «Вес».
    #[serde(alias = "weight")]
    Progress,
    Profile,
}

impl Tab {
    /// Вкладка, подпись, иконка (src/icons.rs).
    pub const ALL: [(Tab, &'static str, &'static str); 5] = [
        (Tab::Today, "Сегодня", "home"),
        (Tab::Plan, "План", "utensils"),
        (Tab::Shop, "Покупки", "cart"),
        (Tab::Progress, "Прогресс", "chart"),
        (Tab::Profile, "Профиль", "user"),
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReminderKind {
    Weigh,
    Breakfast,
    Lunch,
    Dinner,
    Water,
}

impl ReminderKind {
    pub fn title(self) -> &'static str {
        match self {
            ReminderKind::Weigh => "Взвеситься",
            ReminderKind::Breakfast => "Завтрак",
            ReminderKind::Lunch => "Обед",
            ReminderKind::Dinner => "Ужин",
            ReminderKind::Water => "Выпить воды",
        }
    }

    pub fn body(self) -> &'static str {
        match self {
            ReminderKind::Weigh => "Утром, до еды и воды, на одних весах.",
            ReminderKind::Breakfast | ReminderKind::Lunch | ReminderKind::Dinner => "Загляни в план и отметь, что съел.",
            ReminderKind::Water => "Стакан воды — и отметь его в приложении.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reminder {
    pub kind: ReminderKind,
    pub hour: u8,
    pub minute: u8,
    pub on: bool,
}

pub fn default_reminders() -> Vec<Reminder> {
    use ReminderKind::*;
    [(Weigh, 7, 30), (Breakfast, 8, 30), (Lunch, 13, 0), (Water, 15, 0), (Dinner, 19, 0)]
        .into_iter()
        .map(|(kind, hour, minute)| Reminder { kind, hour, minute, on: false })
        .collect()
}

/// Настройки, которые переживают перезапуск и синхронизируются между устройствами.
/// `serde(default)`: старый файл без новых полей тоже прочитается.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub profile: Option<Profile>,
    /// Свой план. `None` — встроенное меню, подогнанное под норму.
    pub custom_plan: Option<Vec<PlanDay>>,
    pub custom_foods: Vec<CustomFood>,
    pub reminders: Vec<Reminder>,
    pub day: u8,
    pub tab: Tab,
    /// Отмеченные в списке покупок продукты.
    pub shop: Vec<FoodRef>,
    /// Дни, на которые закупаемся.
    pub shop_days: Vec<u8>,
    /// Когда настройки последний раз менялись, мс. Для синка: побеждает более новое.
    pub updated_ms: i64,
    /// Из первой версии: цель до появления профиля. Нужна только для подсказки в анкете.
    pub goal: f64,
    /// Из первой версии, больше не используется.
    pub mode: Mode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            profile: None,
            custom_plan: None,
            custom_foods: Vec::new(),
            reminders: default_reminders(),
            day: 0,
            tab: Tab::Today,
            shop: Vec::new(),
            shop_days: (0..7).collect(),
            updated_ms: 0,
            goal: 70.0,
            mode: Mode::Cut,
        }
    }
}

impl Settings {
    pub fn targets(&self, weight: Option<f64>, year: i32) -> Option<Targets> {
        let p = self.profile.as_ref()?;
        Some(p.targets(weight.unwrap_or(p.start_weight), year))
    }

    /// План, который видит человек: свой или встроенный под его норму.
    pub fn plan(&self, weight: Option<f64>, year: i32) -> Vec<PlanDay> {
        if let Some(p) = &self.custom_plan {
            return p.clone();
        }
        let kcal = self.targets(weight, year).map(|t| t.kcal).unwrap_or(1816.0);
        scaled_plan(kcal)
    }
}

/// Что показывать в карточке синхронизации.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SyncInfo {
    pub enabled: bool,
    /// Email, если выполнен вход.
    pub account: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_first_version_settings() {
        let old = r#"{"goal":72.0,"mode":"orig","day":3,"tab":"weight","shop":["egg","nuts"],"shop_days":[0,1]}"#;
        let s: Settings = serde_json::from_str(old).unwrap();
        assert_eq!(s.tab, Tab::Progress);
        assert_eq!(s.shop, vec![FoodRef::Base(crate::foods::FoodId::Egg), FoodRef::Base(crate::foods::FoodId::Nuts)]);
        assert_eq!(s.goal, 72.0);
        assert!(s.profile.is_none());
        assert_eq!(s.reminders.len(), 5);
    }
}
