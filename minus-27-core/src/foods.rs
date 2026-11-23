use serde::{Deserialize, Serialize};

/// Категория продукта, по ней группируется список покупок.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Meat,
    Dairy,
    Grain,
    Veg,
    Other,
}

impl Cat {
    pub const ALL: [Cat; 5] = [Cat::Meat, Cat::Dairy, Cat::Grain, Cat::Veg, Cat::Other];

    pub fn emoji(self) -> &'static str {
        match self {
            Cat::Meat => "🍗",
            Cat::Dairy => "🥛",
            Cat::Grain => "🌾",
            Cat::Veg => "🥦",
            Cat::Other => "🥜",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Cat::Meat => "Мясо и рыба",
            Cat::Dairy => "Молочка и яйца",
            Cat::Grain => "Крупы и хлеб",
            Cat::Veg => "Овощи и фрукты",
            Cat::Other => "Прочее",
        }
    }
}

/// Штучный продукт: вес одной штуки и подпись («шт», «бел.»).
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub grams: u16,
    pub label: &'static str,
}

/// Продукт на 100 г.
#[derive(Debug)]
pub struct Food {
    pub name: &'static str,
    pub kcal: f64,
    pub protein: f64,
    pub cat: Cat,
    pub piece: Option<Piece>,
    /// Жидкость: показывать в мл и литрах.
    pub ml: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FoodId {
    Egg,
    White,
    Oats,
    Berry,
    Banana,
    Chicken,
    Rice,
    Veg,
    Oil,
    Salmon,
    Potato,
    Broccoli,
    Gyog,
    Apple,
    Nuts,
    Almond,
    Curd,
    Turkey,
    Buck,
    Avocado,
    Bread,
    Kefir,
    Protein,
    Pb,
    Milk,
    Beef,
    Fish,
    Quinoa,
    Kiwi,
    Chia,
    Bulgur,
    Shrimp,
    Orange,
    Cheese,
    Honey,
}

const fn food(name: &'static str, kcal: f64, protein: f64, cat: Cat) -> Food {
    Food { name, kcal, protein, cat, piece: None, ml: false }
}

const fn liquid(name: &'static str, kcal: f64, protein: f64, cat: Cat) -> Food {
    Food { name, kcal, protein, cat, piece: None, ml: true }
}

const fn pieces(name: &'static str, kcal: f64, protein: f64, cat: Cat, grams: u16, label: &'static str) -> Food {
    Food { name, kcal, protein, cat, piece: Some(Piece { grams, label }), ml: false }
}

impl FoodId {
    pub fn info(self) -> &'static Food {
        use Cat::*;
        static EGG: Food = pieces("Яйца", 143.0, 12.6, Dairy, 50, "шт");
        static WHITE: Food = pieces("Яичные белки", 52.0, 10.9, Dairy, 33, "бел.");
        static OATS: Food = food("Овсянка", 370.0, 13.0, Grain);
        static BERRY: Food = food("Ягоды", 45.0, 0.8, Veg);
        static BANANA: Food = food("Банан", 89.0, 1.1, Veg);
        static CHICKEN: Food = food("Куриная грудка", 120.0, 22.5, Meat);
        static RICE: Food = food("Рис (сухой)", 350.0, 7.0, Grain);
        static VEG: Food = food("Овощи / салат", 28.0, 1.5, Veg);
        static OIL: Food = food("Оливковое масло", 884.0, 0.0, Other);
        static SALMON: Food = food("Лосось", 208.0, 20.0, Meat);
        static POTATO: Food = food("Картофель", 77.0, 2.0, Veg);
        static BROCCOLI: Food = food("Брокколи", 34.0, 2.8, Veg);
        static GYOG: Food = food("Греческий йогурт", 80.0, 9.0, Dairy);
        static APPLE: Food = food("Яблоко", 52.0, 0.3, Veg);
        static NUTS: Food = food("Орехи", 600.0, 16.0, Other);
        static ALMOND: Food = food("Миндаль", 580.0, 21.0, Other);
        static CURD: Food = food("Творог 5%", 121.0, 17.0, Dairy);
        static TURKEY: Food = food("Филе индейки", 115.0, 23.0, Meat);
        static BUCK: Food = food("Гречка (сухая)", 340.0, 12.6, Grain);
        static AVOCADO: Food = food("Авокадо", 160.0, 2.0, Veg);
        static BREAD: Food = food("Цельнозерновой хлеб", 250.0, 9.0, Grain);
        static KEFIR: Food = liquid("Кефир", 53.0, 3.0, Dairy);
        static PROTEIN: Food = food("Протеин", 380.0, 80.0, Other);
        static PB: Food = food("Арахисовая паста", 600.0, 25.0, Other);
        static MILK: Food = liquid("Молоко 1,5%", 47.0, 3.1, Dairy);
        static BEEF: Food = food("Говядина постная", 130.0, 21.0, Meat);
        static FISH: Food = food("Белая рыба", 82.0, 18.0, Meat);
        static QUINOA: Food = food("Киноа (сухая)", 368.0, 14.0, Grain);
        static KIWI: Food = food("Киви", 61.0, 1.1, Veg);
        static CHIA: Food = food("Семена чиа", 486.0, 17.0, Other);
        static BULGUR: Food = food("Булгур (сухой)", 342.0, 12.0, Grain);
        static SHRIMP: Food = food("Креветки", 85.0, 18.0, Meat);
        static ORANGE: Food = food("Апельсин", 47.0, 0.9, Veg);
        static CHEESE: Food = food("Сыр", 350.0, 25.0, Dairy);
        static HONEY: Food = food("Мёд", 304.0, 0.3, Other);
        match self {
            Self::Egg => &EGG,
            Self::White => &WHITE,
            Self::Oats => &OATS,
            Self::Berry => &BERRY,
            Self::Banana => &BANANA,
            Self::Chicken => &CHICKEN,
            Self::Rice => &RICE,
            Self::Veg => &VEG,
            Self::Oil => &OIL,
            Self::Salmon => &SALMON,
            Self::Potato => &POTATO,
            Self::Broccoli => &BROCCOLI,
            Self::Gyog => &GYOG,
            Self::Apple => &APPLE,
            Self::Nuts => &NUTS,
            Self::Almond => &ALMOND,
            Self::Curd => &CURD,
            Self::Turkey => &TURKEY,
            Self::Buck => &BUCK,
            Self::Avocado => &AVOCADO,
            Self::Bread => &BREAD,
            Self::Kefir => &KEFIR,
            Self::Protein => &PROTEIN,
            Self::Pb => &PB,
            Self::Milk => &MILK,
            Self::Beef => &BEEF,
            Self::Fish => &FISH,
            Self::Quinoa => &QUINOA,
            Self::Kiwi => &KIWI,
            Self::Chia => &CHIA,
            Self::Bulgur => &BULGUR,
            Self::Shrimp => &SHRIMP,
            Self::Orange => &ORANGE,
            Self::Cheese => &CHEESE,
            Self::Honey => &HONEY,
        }
    }
}

/// Количество в плане: «2 шт · 100 г», «250 мл», «80 г».
pub fn qty(id: FoodId, grams: u32) -> String {
    let f = id.info();
    match f.piece {
        Some(p) if p.label == "шт" => format!("{} шт · {} г", round_div(grams, p.grams as u32), grams),
        _ if f.ml => format!("{grams} мл"),
        _ => format!("{grams} г"),
    }
}

/// Количество в списке покупок: штуки округляются вверх, большие объёмы в кг и л.
pub fn shop_qty(id: FoodId, grams: u32) -> String {
    let f = id.info();
    if let Some(p) = f.piece {
        return format!("≈ {} {} ({} г)", grams.div_ceil(p.grams as u32), p.label, grams);
    }
    let (small, big) = if f.ml { ("мл", "л") } else { ("г", "кг") };
    if grams >= 1000 {
        format!("{} {big}", format!("{:.2}", grams as f64 / 1000.0).replace('.', ","))
    } else {
        format!("{grams} {small}")
    }
}

fn round_div(a: u32, b: u32) -> u32 {
    (a + b / 2) / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantities() {
        assert_eq!(qty(FoodId::Egg, 100), "2 шт · 100 г");
        assert_eq!(qty(FoodId::Egg, 150), "3 шт · 150 г");
        assert_eq!(qty(FoodId::White, 150), "150 г");
        assert_eq!(qty(FoodId::Kefir, 250), "250 мл");
        assert_eq!(shop_qty(FoodId::White, 550), "≈ 17 бел. (550 г)");
        assert_eq!(shop_qty(FoodId::Veg, 2650), "2,65 кг");
        assert_eq!(shop_qty(FoodId::Kefir, 1000), "1,00 л");
        assert_eq!(shop_qty(FoodId::Oil, 35), "35 г");
    }
}
