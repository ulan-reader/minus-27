use leptos::prelude::*;
use leptos::task::spawn_local;
use minus27_core::{
    diary::{DiaryItem, Source},
    foods::FoodRef,
    plan::{MealKind, items_totals, view_totals},
    weight::{self, Entry},
};
use minus27_data::sync::EstimateItem;

use crate::{
    api,
    app::Store,
    components::{food_picker, num, ring, sheet},
    fmt, icons::icon, js,
};

/// Какая шторка открыта.
#[derive(Clone, PartialEq)]
enum Sheet {
    Add(MealKind),
    Grams(MealKind, FoodRef),
    Manual(MealKind),
    Photo(Vec<EstimateItem>, Option<String>),
    Weight,
}

/// Номер дня плана для даты: понедельник — день 1.
pub fn plan_day_index(iso: &str) -> usize {
    weight::day_num(iso).map(|n| (n + 3).rem_euclid(7) as usize).unwrap_or(0)
}

fn new_id() -> String {
    format!("{:x}", js::now_ms())
}

#[component]
pub fn TodayScreen() -> impl IntoView {
    let store = expect_context::<Store>();
    let open = RwSignal::new(None::<Sheet>);
    let photo_busy = RwSignal::new(false);
    let close = move || open.set(None);
    let date = js::today_iso();
    let date2 = date.clone();

    let add_item = move |meal: MealKind, name: String, grams: Option<u32>, kcal: f64, protein: f64, source: Source| {
        let date = js::today_iso();
        store.update_day(&date, |d| {
            d.items.push(DiaryItem { id: new_id(), meal, name, grams, kcal, protein, source });
        });
    };

    let take_photo = move |_| {
        if !store.sync.with_untracked(|s| s.enabled && s.account.is_some()) {
            store.toast("Чтобы распознавать фото, войди в аккаунт в «Профиле»");
            return;
        }
        spawn_local(async move {
            let Some(image) = js::pick_photo().await else { return };
            photo_busy.set(true);
            match api::estimate_food(&image).await {
                Ok(est) if est.items.is_empty() => {
                    store.toast(est.note.unwrap_or_else(|| "Еда на фото не найдена".into()))
                }
                Ok(est) => {
                    if let Some(left) = est.left_today {
                        store.toast(format!("Осталось распознаваний сегодня: {left}"));
                    }
                    open.set(Some(Sheet::Photo(est.items, est.note)));
                }
                Err(e) => store.toast(e),
            }
            photo_busy.set(false);
        });
    };

    let header = move || {
        let name = store.settings.with(|s| s.profile.as_ref().map(|p| p.name.clone()).unwrap_or_default());
        view! {
            <header class="top">
                <p class="mut">{fmt::long_date(&date)}</p>
                <h1>{if name.is_empty() { "Сегодня".to_string() } else { format!("Привет, {name}") }}</h1>
            </header>
        }
    };

    let rings = move || {
        let day = store.today();
        let t = day.totals();
        let Some(goal) = store.targets() else { return ().into_any() };
        view! {
            <div class="card rings">
                {ring(t.kcal, goal.kcal, "kcal", fmt::int(t.kcal), format!("из {} ккал", fmt::int(goal.kcal)))}
                {ring(t.protein, goal.protein, "protein", format!("{} г", fmt::int(t.protein)), format!("из {} белка", fmt::int(goal.protein)))}
                {ring(day.water_ml as f64, goal.water_ml as f64, "water", fmt::liters(day.water_ml), format!("из {} л", fmt::liters(goal.water_ml)))}
            </div>
            <p class="hint center">
                {let left = goal.kcal - t.kcal;
                if left >= 0.0 { format!("Осталось {} ккал", fmt::int(left)) } else { format!("Больше нормы на {} ккал", fmt::int(-left)) }}
            </p>
        }
        .into_any()
    };

    let actions = move || {
        view! {
            <div class="actions">
                <button type="button" class="btn pri" on:click=take_photo disabled=move || photo_busy.get()>
                    {icon("camera")}
                    {move || if photo_busy.get() { "Распознаю…" } else { "Фото еды" }}
                </button>
                <button
                    type="button"
                    class="btn"
                    on:click=move |_| open.set(Some(Sheet::Add(MealKind::at_hour(js::hour()))))
                >
                    {icon("plus")}
                    "Добавить"
                </button>
            </div>
        }
    };

    let weight_card = move || {
        let pts = store.entries.with(|e| weight::points(e));
        let last = pts.last().cloned();
        let week_ago = last.as_ref().and_then(|l| pts.iter().rev().find(|p| l.n - p.n >= 7).map(|p| p.entry.weight));
        let today = js::today_iso();
        let done = last.as_ref().is_some_and(|l| l.entry.date == today);
        view! {
            <button type="button" class="card row tap" on:click=move |_| open.set(Some(Sheet::Weight))>
                <span class="badge green">{icon("scale")}</span>
                <div class="grow">
                    {match &last {
                        Some(l) => view! { <b>{format!("{} кг", fmt::f1(l.entry.weight))}</b> }.into_any(),
                        None => view! { <b>"Взвесься"</b> }.into_any(),
                    }}
                    <div class="mut">
                        {match (week_ago, &last) {
                            (Some(w), Some(l)) => format!("{} за неделю", fmt::signed(l.entry.weight - w)),
                            _ if done => "Сегодня уже записано".into(),
                            _ => "Утром, до еды и воды".into(),
                        }}
                    </div>
                </div>
                <span class=if done { "chip" } else { "chip on" }>{if done { "Изменить" } else { "+ вес" }}</span>
            </button>
        }
    };

    let meals = move || {
        let plan = store.plan();
        let customs = store.settings.with(|s| s.custom_foods.clone());
        let day = store.today();
        let pd = plan.get(plan_day_index(&date2)).cloned().unwrap_or_default();
        let now_meal = MealKind::at_hour(js::hour());
        MealKind::ALL
            .into_iter()
            .map(|kind| {
                let pm = pd.iter().find(|m| m.kind == kind).cloned();
                let done = day.plan_done(kind);
                let planned = pm.as_ref().map(|m| items_totals(&m.items, &customs)).unwrap_or_default();
                let eaten: Vec<DiaryItem> = day.items.iter().filter(|i| i.meal == kind).cloned().collect();
                let got = day.meal_totals(kind);
                let summary = pm
                    .as_ref()
                    .map(|m| m.items.iter().filter_map(|i| i.food.view(&customs).map(|v| v.name)).collect::<Vec<_>>().join(", "))
                    .unwrap_or_default();
                let customs2 = customs.clone();
                let toggle = move |_| {
                    let Some(m) = pm.clone() else { return };
                    let customs = customs2.clone();
                    let date = js::today_iso();
                    let id = new_id();
                    store.update_day(&date, |d| {
                        if d.plan_done(m.kind) { d.unmark_plan_meal(m.kind) } else { d.eat_plan_meal(&m, &customs, &id) }
                    });
                };
                view! {
                    <div class=if kind == now_meal && !done { "card meal now" } else { "card meal" }>
                        <div class="row">
                            <span class="badge">{icon(kind.icon())}</span>
                            <div class="grow">
                                <b>{kind.title()}</b>
                                <div class="mut">
                                    {if got.kcal > 0.0 {
                                        format!("{} ккал · Б {} г", fmt::int(got.kcal), fmt::int(got.protein))
                                    } else {
                                        format!("по плану {} ккал", fmt::int(planned.kcal))
                                    }}
                                </div>
                            </div>
                            <button type="button" class="icon-btn" aria-label="Добавить" on:click=move |_| open.set(Some(Sheet::Add(kind)))>
                                {icon("plus")}
                            </button>
                            <button
                                type="button"
                                class=if done { "check on" } else { "check" }
                                aria-label="Съел по плану"
                                on:click=toggle
                            >
                                {icon(if done { "check-circle" } else { "circle" })}
                            </button>
                        </div>
                        {(!done && !summary.is_empty()).then(|| view! { <p class="plan-line">{summary}</p> })}
                        <ul class="items">
                            {eaten
                                .into_iter()
                                .filter(|i| i.source != Source::Plan)
                                .map(|i| {
                                    let id = i.id.clone();
                                    view! {
                                        <li>
                                            {(i.source == Source::Photo).then(|| icon("sparkles"))}
                                            <span class="grow">
                                                {i.name}
                                                {i.grams.map(|g| format!(" · {g} г"))}
                                            </span>
                                            <span class="mut">{format!("{} ккал", fmt::int(i.kcal))}</span>
                                            <button
                                                type="button"
                                                class="icon-btn sm"
                                                aria-label="Удалить"
                                                on:click=move |_| {
                                                    let id = id.clone();
                                                    store.update_day(&js::today_iso(), |d| d.remove(&id));
                                                }
                                            >
                                                {icon("x")}
                                            </button>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    </div>
                }
            })
            .collect_view()
    };

    let water = move || {
        let day = store.today();
        let goal = store.targets().map(|t| t.water_ml).unwrap_or(2500);
        let add = move |ml: i32| store.update_day(&js::today_iso(), |d| d.add_water(ml));
        view! {
            <div class="card row">
                <span class="badge blue">{icon("droplet")}</span>
                <div class="grow">
                    <b>{format!("Вода {} из {} л", fmt::liters(day.water_ml), fmt::liters(goal))}</b>
                    {crate::components::bar(day.water_ml as f64 / goal as f64 * 100.0, "water")}
                </div>
                <button type="button" class="icon-btn" aria-label="Минус стакан" on:click=move |_| add(-250)>{icon("minus")}</button>
                <button type="button" class="btn sm" on:click=move |_| add(250)>"+250 мл"</button>
            </div>
        }
    };

    let sheets = move || {
        let s = open.get()?;
        let customs = store.settings.with(|s| s.custom_foods.clone());
        Some(match s {
            Sheet::Add(meal) => sheet(
                format!("Добавить: {}", meal.title().to_lowercase()),
                close,
                view! {
                    {crate::components::chips(
                        MealKind::ALL.into_iter().map(|m| (m, m.title().to_string())).collect(),
                        move |m| m == meal,
                        move |m| open.set(Some(Sheet::Add(m))),
                        "",
                    )}
                    <button type="button" class="btn ghost wide" on:click=move |_| open.set(Some(Sheet::Manual(meal)))>
                        {icon("edit")}
                        "Ввести калории вручную"
                    </button>
                    {food_picker(customs, None, move |r| open.set(Some(Sheet::Grams(meal, r))))}
                }
                .into_any(),
            )
            .into_any(),
            Sheet::Grams(meal, r) => {
                let Some(v) = r.view(&customs) else { return None };
                let grams = RwSignal::new("100".to_string());
                let v2 = v.clone();
                let total = move || view_totals(&v2, num(&grams.get()).unwrap_or(0.0).max(0.0) as u32);
                sheet(
                    v.name.clone(),
                    close,
                    view! {
                        <label class="f">
                            <span>{if v.ml { "Объём" } else { "Вес порции" }}</span>
                            <div class="unit">
                                <input
                                    type="text"
                                    inputmode="numeric"
                                    prop:value=move || grams.get()
                                    on:input=move |ev| grams.set(event_target_value(&ev))
                                />
                                <em>{if v.ml { "мл" } else { "г" }}</em>
                            </div>
                        </label>
                        <p class="big-total">
                            {move || {
                                let t = total();
                                format!("{} ккал · Б {} г", fmt::int(t.kcal), fmt::f1(t.protein))
                            }}
                        </p>
                        <button
                            type="button"
                            class="btn pri wide"
                            on:click=move |_| {
                                let g = num(&grams.get_untracked()).unwrap_or(0.0).round() as u32;
                                if g == 0 {
                                    return;
                                }
                                let t = view_totals(&v, g);
                                add_item(meal, v.name.clone(), Some(g), t.kcal, t.protein, Source::Manual);
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
            Sheet::Manual(meal) => {
                let name = RwSignal::new(String::new());
                let kcal = RwSignal::new(String::new());
                let protein = RwSignal::new(String::new());
                sheet(
                    format!("{}: вручную", meal.title()),
                    close,
                    view! {
                        <label class="f">
                            <span>"Что съел"</span>
                            <input type="text" placeholder="Шаурма" prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev))/>
                        </label>
                        <div class="grid2">
                            <label class="f">
                                <span>"Калории"</span>
                                <input type="text" inputmode="numeric" prop:value=move || kcal.get() on:input=move |ev| kcal.set(event_target_value(&ev))/>
                            </label>
                            <label class="f">
                                <span>"Белок, г"</span>
                                <input type="text" inputmode="decimal" prop:value=move || protein.get() on:input=move |ev| protein.set(event_target_value(&ev))/>
                            </label>
                        </div>
                        <button
                            type="button"
                            class="btn pri wide"
                            on:click=move |_| {
                                let Some(k) = num(&kcal.get_untracked()).filter(|k| *k > 0.0) else { return };
                                let n = name.get_untracked().trim().to_string();
                                let n = if n.is_empty() { "Без названия".to_string() } else { n };
                                add_item(meal, n, None, k, num(&protein.get_untracked()).unwrap_or(0.0), Source::Manual);
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
            Sheet::Photo(items, note) => {
                let meal = RwSignal::new(MealKind::at_hour(js::hour()));
                // Каждую позицию можно поправить по граммам и выключить.
                let rows: Vec<(EstimateItem, RwSignal<String>, RwSignal<bool>)> =
                    items.into_iter().map(|i| { let g = i.grams.to_string(); (i, RwSignal::new(g), RwSignal::new(true)) }).collect();
                let rows2 = rows.clone();
                let scaled = move |i: &EstimateItem, g: &RwSignal<String>| {
                    let grams = num(&g.get()).unwrap_or(i.grams as f64).max(0.0);
                    let k = if i.grams > 0 { grams / i.grams as f64 } else { 1.0 };
                    (grams.round() as u32, i.kcal * k, i.protein * k)
                };
                sheet(
                    "Распознано по фото".into(),
                    close,
                    view! {
                        {note.map(|n| view! { <p class="hint">{icon("info")} {n}</p> })}
                        <div class="photo-items">
                            {rows
                                .into_iter()
                                .map(|(i, g, on)| {
                                    let i2 = i.clone();
                                    view! {
                                        <div class=move || if on.get() { "pi" } else { "pi off" }>
                                            <button type="button" class="check" on:click=move |_| on.update(|v| *v = !*v)>
                                                {move || icon(if on.get() { "check-circle" } else { "circle" })}
                                            </button>
                                            <div class="grow">
                                                <b>{i.name.clone()}</b>
                                                <div class="mut">
                                                    {move || {
                                                        let (_, k, p) = scaled(&i2, &g);
                                                        format!("{} ккал · Б {} г", fmt::int(k), fmt::f1(p))
                                                    }}
                                                </div>
                                            </div>
                                            <div class="unit sm">
                                                <input type="text" inputmode="numeric" prop:value=move || g.get() on:input=move |ev| g.set(event_target_value(&ev))/>
                                                <em>"г"</em>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                        {crate::components::chips(
                            MealKind::ALL.into_iter().map(|m| (m, m.title().to_string())).collect(),
                            move |m| meal.get() == m,
                            move |m| meal.set(m),
                            "",
                        )}
                        <button
                            type="button"
                            class="btn pri wide"
                            on:click=move |_| {
                                let m = meal.get_untracked();
                                let date = js::today_iso();
                                let id = new_id();
                                let picked: Vec<DiaryItem> = rows2
                                    .iter()
                                    .filter(|(_, _, on)| on.get_untracked())
                                    .enumerate()
                                    .map(|(n, (i, g, _))| {
                                        let (grams, kcal, protein) = scaled(i, g);
                                        DiaryItem { id: format!("{id}-{n}"), meal: m, name: i.name.clone(), grams: Some(grams), kcal, protein, source: Source::Photo }
                                    })
                                    .collect();
                                store.update_day(&date, |d| d.items.extend(picked));
                                close();
                            }
                        >
                            "Добавить в дневник"
                        </button>
                        <p class="hint center">"Оценка по фото приблизительная: проверь граммы, особенно масло и соусы."</p>
                    }
                    .into_any(),
                )
                .into_any()
            }
            Sheet::Weight => {
                let today = js::today_iso();
                let cur = store.entries.with_untracked(|e| e.iter().find(|x| x.date == today).cloned());
                let w = RwSignal::new(cur.as_ref().map(|e| fmt::trim0(e.weight)).unwrap_or_default());
                let err = RwSignal::new(String::new());
                sheet(
                    "Вес сегодня".into(),
                    close,
                    view! {
                        <label class="f">
                            <span>"Вес"</span>
                            <div class="unit">
                                <input type="text" inputmode="decimal" placeholder="97,0" prop:value=move || w.get() on:input=move |ev| w.set(event_target_value(&ev))/>
                                <em>"кг"</em>
                            </div>
                        </label>
                        <p class="err">{move || err.get()}</p>
                        <button
                            type="button"
                            class="btn pri wide"
                            on:click=move |_| {
                                let Some(kg) = num(&w.get_untracked()) else { err.set("Введи вес".into()); return };
                                let mut e = cur.clone().unwrap_or_else(|| Entry { date: js::today_iso(), ..Entry::default() });
                                e.weight = (kg * 10.0).round() / 10.0;
                                if let Err(m) = weight::check(&e, Some(js::today_num())) {
                                    err.set(m.into());
                                    return;
                                }
                                spawn_local(async move {
                                    match api::save_entry(&e).await {
                                        Ok(list) => {
                                            store.entries.set(list);
                                            store.sync();
                                            store.toast("Вес записан");
                                        }
                                        Err(m) => store.toast(m),
                                    }
                                });
                                close();
                            }
                        >
                            "Сохранить"
                        </button>
                        <p class="hint">"Замеры талии, бёдер и шагов — во вкладке «Прогресс»."</p>
                    }
                    .into_any(),
                )
                .into_any()
            }
        })
    };

    view! {
        {header}
        {rings}
        {actions}
        {weight_card}
        <h2>"Приёмы пищи"</h2>
        {meals}
        {water}
        {sheets}
    }
}
