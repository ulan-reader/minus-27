use leptos::prelude::*;
use minus27_core::{
    foods::qty,
    plan::{self, Item, Mode, PLAN, TARGET_PROTEIN},
};

use crate::{
    app::Store,
    components::{bar, chips},
    js,
};

const RULES: [&str; 6] = [
    "⚖️ Мясо, рыбу, крупы взвешивай сырыми, как в плане.",
    "💧 Вода 2–2,5 л в день.",
    "👣 Шаги: +1 000 в неделю, цель 10–12 тыс.",
    "🏋️ Силовые: 2 в неделю, потом 3. Бассейн пока заменяй ходьбой в горку.",
    "😴 Сон 7–9 часов.",
    "🍽️ Приёмы пищи не пропускай ради ускорения.",
];

fn round(x: f64) -> i64 {
    x.round() as i64
}

fn item_row(it: &Item, mode: Mode) -> AnyView {
    let f = it.food.info();
    let g = plan::grams_of(it, mode);
    if g == 0 {
        let lost = round(plan::item_totals(it.food, it.grams).kcal);
        return view! {
            <li class="item gone">
                <span>{f.name}</span>
                <span class="q">{format!("убрано · −{lost} ккал")}</span>
            </li>
        }
        .into_any();
    }
    if plan::is_changed(it, mode) {
        return view! {
            <li class="item">
                <span>{f.name}</span>
                <span class="q">
                    <s>{qty(it.food, it.grams as u32)}</s>
                    " → "
                    {qty(it.food, g as u32)}
                </span>
            </li>
        }
        .into_any();
    }
    view! {
        <li class="item">
            <span>{f.name}</span>
            <span class="q">{qty(it.food, g as u32)}</span>
        </li>
    }
    .into_any()
}

#[component]
pub fn PlanTab() -> impl IntoView {
    let store = expect_context::<Store>();
    let mode = move || store.settings.with(|s| s.mode);
    let day = move || store.settings.with(|s| (s.day as usize).min(PLAN.len() - 1));

    let modes = vec![(Mode::Cut, "🔥 Минус ~1 кг/нед".to_string()), (Mode::Orig, "📋 Оригинал ChatGPT".to_string())];
    let days: Vec<(usize, String)> = (0..PLAN.len()).map(|i| (i, format!("День {}", i + 1))).collect();

    let summary = move || {
        let t = plan::day_totals(day(), mode());
        let cls = if t.protein >= TARGET_PROTEIN { "ok" } else { "low" };
        view! {
            <div class="card sum">
                <div class="big">{format!("≈ {} ккал", round(t.kcal))}</div>
                <div class="sub">{format!("Белок {} г · цель от {} г", round(t.protein), TARGET_PROTEIN)}</div>
                {bar(t.protein / TARGET_PROTEIN * 100.0, cls)}
            </div>
        }
    };

    let meals = move || {
        let m = mode();
        PLAN[day()]
            .iter()
            .map(|meal| {
                let mt = plan::meal_totals(meal, m);
                view! {
                    <div class="card meal">
                        <h3>
                            <span>{format!("{} {}", meal.emoji, meal.title)}</span>
                            <small>{format!("{} ккал · Б {} г", round(mt.kcal), round(mt.protein))}</small>
                        </h3>
                        <ul>{meal.items.iter().map(|it| item_row(it, m)).collect_view()}</ul>
                    </div>
                }
            })
            .collect_view()
    };

    let week = move || {
        let m = mode();
        let avg = plan::week_average(m);
        let rows = (0..PLAN.len())
            .map(|i| {
                let t = plan::day_totals(i, m);
                view! {
                    <button
                        type="button"
                        class=move || if day() == i { "wk on" } else { "wk" }
                        on:click=move |_| {
                            store.set(|s| s.day = i as u8);
                            js::scroll_top();
                        }
                    >
                        <span>{format!("Д{}", i + 1)}</span>
                        {bar(t.kcal / 2300.0 * 100.0, "")}
                        <span class="n">{format!("{} ккал · Б {}", round(t.kcal), round(t.protein))}</span>
                    </button>
                }
            })
            .collect_view();
        view! {
            <div class="card">
                {rows}
                <p class="hint">
                    {format!(
                        "В среднем ≈ {} ккал и {} г белка в день. Цифры приблизительные, считал по таблицам калорийности.",
                        round(avg.kcal),
                        round(avg.protein),
                    )}
                </p>
            </div>
        }
    };

    view! {
        {chips(modes, move |v| mode() == v, move |v| store.set(|s| s.mode = v), "seg")}
        <p class="hint">
            {move || match mode() {
                Mode::Cut => "От оригинала: без орехов и мёда, без банана в завтраках, масло 5 г, крупы 60 г, хлеб 60 г. Мясо, рыба, творог, яйца, кефир не тронуты.",
                Mode::Orig => "Меню ChatGPT без правок. Это дефицит около 400 ккал, примерно 0,5 кг в неделю.",
            }}
        </p>
        {chips(days, move |v| day() == v, move |v| store.set(|s| s.day = v as u8), "days")}
        {summary}
        {meals}
        <h2>"📊 Неделя целиком"</h2>
        {week}
        <h2>"✅ Правила"</h2>
        <div class="card">
            <ul class="rules">{RULES.iter().map(|r| view! { <li>{*r}</li> }).collect_view()}</ul>
        </div>
    }
}
