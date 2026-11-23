use serde::{Deserialize, Serialize};

use crate::{foods::FoodId, plan::Mode};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tab {
    #[default]
    Plan,
    Shop,
    Weight,
}

impl Tab {
    pub const ALL: [(Tab, &'static str); 3] =
        [(Tab::Plan, "🍽️ План"), (Tab::Shop, "🛒 Покупки"), (Tab::Weight, "⚖️ Вес")];
}

/// Настройки интерфейса, которые переживают перезапуск.
/// `serde(default)`: старый файл без новых полей тоже прочитается.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub goal: f64,
    pub mode: Mode,
    pub day: u8,
    pub tab: Tab,
    /// Отмеченные в списке покупок продукты.
    pub shop: Vec<FoodId>,
    /// Дни, на которые закупаемся.
    pub shop_days: Vec<u8>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { goal: 70.0, mode: Mode::Cut, day: 0, tab: Tab::Plan, shop: Vec::new(), shop_days: (0..7).collect() }
    }
}

/// Что показывать в карточке синхронизации.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SyncInfo {
    pub enabled: bool,
    /// Email, если выполнен вход.
    pub account: Option<String>,
}
