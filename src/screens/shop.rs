use leptos::prelude::*;
use leptos::task::spawn_local;
use minus27_core::{
    foods::{Cat, FoodRef},
    plan::plan_shopping_list,
};

use crate::{
    app::Store,
    components::{bar, chips},
    icons::icon, js,
};

const DAYS: [&str; 7] = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

#[component]
pub fn ShopScreen() -> impl IntoView {
    let store = expect_context::<Store>();

    let toggle_day = move |d: u8| {
        store.set(|s| {
            match s.shop_days.iter().position(|&x| x == d) {
                Some(i) => {
                    s.shop_days.remove(i);
                }
                None => s.shop_days.push(d),
            }
            s.shop_days.sort_unstable();
        })
    };
    let toggle_food = move |f: FoodRef, on: bool| {
        store.set(|s| {
            s.shop.retain(|x| *x != f);
            if on {
                s.shop.push(f);
            }
        })
    };
    // Список: (продукт, название, количество, категория)
    let list = move || {
        let plan = store.plan();
        store.settings.with(|s| {
            plan_shopping_list(&plan, &s.shop_days)
                .into_iter()
                .filter_map(|(r, g)| {
                    let v = r.view(&s.custom_foods)?;
                    Some((r, v.name.clone(), v.shop_qty(g), v.cat))
                })
                .collect::<Vec<_>>()
        })
    };

    let copy = move |_: leptos::ev::MouseEvent| {
        let text = list().iter().map(|(_, n, q, _)| format!("- {n}: {q}")).collect::<Vec<_>>().join("\n");
        spawn_local(async move {
            if js::copy(&text).await {
                store.toast("Список скопирован");
            } else {
                store.toast("Не удалось скопировать");
            }
        });
    };

    let body = move || {
        let acc = list();
        if acc.is_empty() {
            return view! { <div class="card empty">"Выбери хотя бы один день"</div> }.into_any();
        }
        let bought = store.settings.with(|s| s.shop.clone());
        let done = acc.iter().filter(|(r, ..)| bought.contains(r)).count();
        let total = acc.len();
        let groups = Cat::ALL
            .into_iter()
            .filter_map(|cat| {
                let items: Vec<_> = acc.iter().filter(|(.., c)| *c == cat).cloned().collect();
                if items.is_empty() {
                    return None;
                }
                let rows = items
                    .into_iter()
                    .map(|(r, name, q, _)| {
                        let on = bought.contains(&r);
                        view! {
                            <label class={if on { "chk done" } else { "chk" }}>
                                <input
                                    type="checkbox"
                                    prop:checked=on
                                    on:change=move |ev| toggle_food(r.clone(), event_target_checked(&ev))
                                />
                                <span class="grow">{name}</span>
                                <span class="mut">{q}</span>
                            </label>
                        }
                    })
                    .collect_view();
                Some(view! {
                    <div class="card grp">
                        <h3>{icon(cat.icon())} {cat.title()}</h3>
                        {rows}
                    </div>
                })
            })
            .collect_view();
        view! {
            <div class="card">
                <div class="row">
                    <b class="grow">{format!("Куплено {done} из {total}")}</b>
                    <button type="button" class="icon-btn" aria-label="Скопировать список" on:click=copy>{icon("copy")}</button>
                    <button type="button" class="icon-btn" aria-label="Сбросить отметки" on:click=move |_| store.set(|s| s.shop.clear())>
                        {icon("refresh")}
                    </button>
                </div>
                {bar(done as f64 / total as f64 * 100.0, "ok")}
            </div>
            {groups}
        }
        .into_any()
    };

    view! {
        <header class="top">
            <p class="mut">"Всё в сыром виде, крупы сухие"</p>
            <h1>"Покупки"</h1>
        </header>
        {chips(
            (0..7u8).map(|i| (i, DAYS[i as usize].to_string())).collect(),
            move |d| store.settings.with(|s| s.shop_days.contains(&d)),
            toggle_day,
            "days",
        )}
        {body}
    }
}
