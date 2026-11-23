use leptos::prelude::*;
use minus27_core::{
    foods::FoodRef,
    plan::{MealKind, PlanDay, PlanItem, items_totals, plan_day_totals, plan_week_average, swap_grams, view_totals},
};

use crate::{
    app::Store,
    components::{bar, chips, food_picker, num, sheet},
    fmt, icons::icon, js,
    screens::today::plan_day_index,
};

const DAYS: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

const RULES: [(&str, &str); 6] = [
    ("scale", "Мясо, рыбу, крупы взвешивай сырыми, как в плане."),
    ("droplet", "Пей воду: норма в «Профиле»."),
    ("chart", "Шаги: +1 000 в неделю, цель 10–12 тыс."),
    ("flame", "Силовые 2–3 раза в неделю, чтобы сохранить мышцы."),
    ("moon", "Сон 7–9 часов."),
    ("utensils", "Не пропускай приёмы пищи ради ускорения."),
];

#[derive(Clone, PartialEq)]
enum Sheet {
    /// Правка продукта: день, приём, номер продукта.
    Item(usize, MealKind, usize),
    Swap(usize, MealKind, usize),
    Add(usize, MealKind),
    AddGrams(usize, MealKind, FoodRef),
}

/// Изменить план. Первая правка копирует встроенное меню в свой план.
fn edit(store: Store, f: impl FnOnce(&mut Vec<PlanDay>)) {
    let mut plan = store.plan();
    f(&mut plan);
    store.set(|s| s.custom_plan = Some(plan));
}

fn item_mut(plan: &mut [PlanDay], d: usize, k: MealKind, i: usize) -> Option<&mut PlanItem> {
    plan.get_mut(d)?.iter_mut().find(|m| m.kind == k)?.items.get_mut(i)
}

#[component]
pub fn PlanScreen() -> impl IntoView {
    let store = expect_context::<Store>();
    let open = RwSignal::new(None::<Sheet>);
    let close = move || open.set(None);
    let day = move || store.settings.with(|s| (s.day as usize).min(6));
    let today_idx = plan_day_index(&js::today_iso());

    let header = move || {
        let custom = store.settings.with(|s| s.custom_plan.is_some());
        let kcal = store.targets().map(|t| t.kcal).unwrap_or(0.0);
        view! {
            <header class="top">
                <p class="mut">
                    {if custom { "Свой план".to_string() } else { format!("Подобран под норму {} ккал", fmt::int(kcal)) }}
                </p>
                <h1>"План"</h1>
            </header>
        }
    };

    let day_chips = move || {
        chips(
            (0..7).map(|i| (i, if i == today_idx { format!("{} •", DAYS[i]) } else { DAYS[i].to_string() })).collect(),
            move |i| day() == i,
            move |i| store.set(|s| s.day = i as u8),
            "days",
        )
    };

    let summary = move || {
        let plan = store.plan();
        let customs = store.settings.with(|s| s.custom_foods.clone());
        let t = plan.get(day()).map(|d| plan_day_totals(d, &customs)).unwrap_or_default();
        let goal = store.targets();
        let pk = goal.map(|g| t.kcal / g.kcal * 100.0).unwrap_or(0.0);
        let pp = goal.map(|g| t.protein / g.protein * 100.0).unwrap_or(0.0);
        view! {
            <div class="card sum">
                <div class="row">
                    <div class="grow">
                        <div class="big">{format!("{} ккал", fmt::int(t.kcal))}</div>
                        <div class="mut">{format!("Белок {} г", fmt::int(t.protein))}</div>
                    </div>
                    {goal.map(|g| view! { <div class="mut right">{format!("норма {}", fmt::int(g.kcal))}<br/>{format!("белок {} г", fmt::int(g.protein))}</div> })}
                </div>
                {bar(pk, "kcal")}
                {bar(pp, if pp >= 95.0 { "ok" } else { "protein" })}
            </div>
        }
    };

    let meals = move || {
        let plan = store.plan();
        let customs = store.settings.with(|s| s.custom_foods.clone());
        let d = day();
        let pd = plan.get(d).cloned().unwrap_or_default();
        pd.into_iter()
            .map(|m| {
                let kind = m.kind;
                let t = items_totals(&m.items, &customs);
                view! {
                    <div class="card meal">
                        <div class="row">
                            <span class="badge">{icon(kind.icon())}</span>
                            <b class="grow">{kind.title()}</b>
                            <span class="mut">{format!("{} ккал · Б {}", fmt::int(t.kcal), fmt::int(t.protein))}</span>
                        </div>
                        <ul class="items">
                            {m.items
                                .iter()
                                .enumerate()
                                .map(|(i, it)| {
                                    let v = it.food.view(&customs);
                                    let name = v.as_ref().map(|v| v.name.clone()).unwrap_or_else(|| "Удалённый продукт".into());
                                    let q = v.as_ref().map(|v| v.qty(it.grams)).unwrap_or_default();
                                    view! {
                                        <li class="tap" on:click=move |_| open.set(Some(Sheet::Item(d, kind, i)))>
                                            <span class="grow">{name}</span>
                                            <span class="mut">{q}</span>
                                            {icon("chevron-right")}
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                        <button type="button" class="btn ghost sm" on:click=move |_| open.set(Some(Sheet::Add(d, kind)))>
                            {icon("plus")}
                            "Добавить продукт"
                        </button>
                    </div>
                }
            })
            .collect_view()
    };

    let week = move || {
        let plan = store.plan();
        let customs = store.settings.with(|s| s.custom_foods.clone());
        let avg = plan_week_average(&plan, &customs);
        let max = store.targets().map(|t| t.kcal * 1.25).unwrap_or(2500.0);
        let custom = store.settings.with(|s| s.custom_plan.is_some());
        view! {
            <div class="card">
                {plan
                    .iter()
                    .enumerate()
                    .map(|(i, d)| {
                        let t = plan_day_totals(d, &customs);
                        view! {
                            <button
                                type="button"
                                class=move || if day() == i { "wk on" } else { "wk" }
                                on:click=move |_| {
                                    store.set(|s| s.day = i as u8);
                                    js::scroll_top();
                                }
                            >
                                <span>{DAYS[i]}</span>
                                {bar(t.kcal / max * 100.0, "kcal")}
                                <span class="mut">{format!("{} · Б {}", fmt::int(t.kcal), fmt::int(t.protein))}</span>
                            </button>
                        }
                    })
                    .collect_view()}
                <p class="hint">
                    {format!("В среднем {} ккал и {} г белка в день.", fmt::int(avg.kcal), fmt::int(avg.protein))}
                </p>
                {custom.then(|| view! {
                    <button
                        type="button"
                        class="btn ghost sm"
                        on:click=move |_| {
                            store.set(|s| s.custom_plan = None);
                            store.toast("План снова подобран под твою норму");
                        }
                    >
                        {icon("refresh")}
                        "Вернуть стандартный план"
                    </button>
                })}
            </div>
        }
    };

    let sheets = move || {
        let s = open.get()?;
        let plan = store.plan();
        let customs = store.settings.with(|s| s.custom_foods.clone());
        Some(match s {
            Sheet::Item(d, k, i) => {
                let it = plan.get(d)?.iter().find(|m| m.kind == k)?.items.get(i)?.clone();
                let v = it.food.view(&customs)?;
                let grams = RwSignal::new(it.grams.to_string());
                let v2 = v.clone();
                sheet(
                    v.name.clone(),
                    close,
                    view! {
                        <label class="f">
                            <span>{if v.ml { "Объём" } else { "Вес (сырой)" }}</span>
                            <div class="unit">
                                <input type="text" inputmode="numeric" prop:value=move || grams.get() on:input=move |ev| grams.set(event_target_value(&ev))/>
                                <em>{if v.ml { "мл" } else { "г" }}</em>
                            </div>
                        </label>
                        <p class="big-total">
                            {move || {
                                let t = view_totals(&v2, num(&grams.get()).unwrap_or(0.0).max(0.0) as u32);
                                format!("{} ккал · Б {} г", fmt::int(t.kcal), fmt::f1(t.protein))
                            }}
                        </p>
                        <button
                            type="button"
                            class="btn pri wide"
                            on:click=move |_| {
                                let Some(g) = num(&grams.get_untracked()).filter(|g| *g > 0.0) else { return };
                                edit(store, |p| if let Some(x) = item_mut(p, d, k, i) { x.grams = g.round() as u32 });
                                close();
                            }
                        >
                            "Сохранить"
                        </button>
                        <div class="grid2">
                            <button type="button" class="btn" on:click=move |_| open.set(Some(Sheet::Swap(d, k, i)))>
                                {icon("swap")}
                                "Заменить"
                            </button>
                            <button
                                type="button"
                                class="btn danger"
                                on:click=move |_| {
                                    edit(store, |p| {
                                        if let Some(m) = p.get_mut(d).and_then(|day| day.iter_mut().find(|m| m.kind == k)) && i < m.items.len() {
                                            m.items.remove(i);
                                        }
                                    });
                                    close();
                                }
                            >
                                {icon("trash")}
                                "Убрать"
                            </button>
                        </div>
                    }
                    .into_any(),
                )
                .into_any()
            }
            Sheet::Swap(d, k, i) => {
                let it = plan.get(d)?.iter().find(|m| m.kind == k)?.items.get(i)?.clone();
                let from = it.food.view(&customs)?;
                let customs2 = customs.clone();
                sheet(
                    format!("Заменить: {}", from.name),
                    close,
                    view! {
                        <p class="hint">"Граммы пересчитаются, чтобы калории остались прежними."</p>
                        {food_picker(customs, Some(from.cat), move |r| {
                            let Some(to) = r.view(&customs2) else { return };
                            let g = swap_grams(&from, it.grams, &to);
                            edit(store, |p| if let Some(x) = item_mut(p, d, k, i) { *x = PlanItem { food: r, grams: g } });
                            store.toast(format!("Заменено: {} {} г", to.name, g));
                            close();
                        })}
                    }
                    .into_any(),
                )
                .into_any()
            }
            Sheet::Add(d, k) => sheet(
                format!("{}: добавить", k.title()),
                close,
                food_picker(customs, None, move |r| open.set(Some(Sheet::AddGrams(d, k, r)))).into_any(),
            )
            .into_any(),
            Sheet::AddGrams(d, k, r) => {
                let v = r.view(&customs)?;
                let grams = RwSignal::new("100".to_string());
                sheet(
                    v.name.clone(),
                    close,
                    view! {
                        <label class="f">
                            <span>"Сколько"</span>
                            <div class="unit">
                                <input type="text" inputmode="numeric" prop:value=move || grams.get() on:input=move |ev| grams.set(event_target_value(&ev))/>
                                <em>{if v.ml { "мл" } else { "г" }}</em>
                            </div>
                        </label>
                        <button
                            type="button"
                            class="btn pri wide"
                            on:click=move |_| {
                                let Some(g) = num(&grams.get_untracked()).filter(|g| *g > 0.0) else { return };
                                let food = r.clone();
                                edit(store, |p| {
                                    if let Some(m) = p.get_mut(d).and_then(|day| day.iter_mut().find(|m| m.kind == k)) {
                                        m.items.push(PlanItem { food, grams: g.round() as u32 });
                                    }
                                });
                                close();
                            }
                        >
                            "Добавить"
                        </button>
                    }
                    .into_any(),
                )
                .into_any()
            }
        })
    };

    view! {
        {header}
        {day_chips}
        {summary}
        {meals}
        <h2>"Неделя"</h2>
        {week}
        <h2>"Правила"</h2>
        <div class="card list">
            {RULES.iter().map(|(ic, r)| view! { <div class="rule">{icon(ic)}<span>{*r}</span></div> }).collect_view()}
        </div>
        {sheets}
    }
}
