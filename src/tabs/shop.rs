use leptos::prelude::*;
use leptos::task::spawn_local;
use minus27_core::{
    foods::{Cat, FoodId, shop_qty},
    plan::{self, PLAN},
};

use crate::{
    app::Store,
    components::{bar, chips},
    js,
};

#[component]
pub fn ShopTab() -> impl IntoView {
    let store = expect_context::<Store>();
    // Текст после «Скопировать»: подтверждение или сам список, если буфер недоступен.
    let copied = RwSignal::new(None::<String>);

    let days: Vec<(u8, String)> = (0..PLAN.len() as u8).map(|i| (i, format!("Д{}", i + 1))).collect();
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
    let toggle_food = move |f: FoodId, on: bool| {
        store.set(|s| {
            s.shop.retain(|&x| x != f);
            if on {
                s.shop.push(f);
            }
        })
    };
    let list = move || store.settings.with(|s| plan::shopping_list(&s.shop_days, s.mode));

    let copy = move |_: leptos::ev::MouseEvent| {
        let text = list().iter().map(|&(f, g)| format!("- {}: {}", f.info().name, shop_qty(f, g))).collect::<Vec<_>>().join("\n");
        spawn_local(async move {
            let msg = if js::copy(&text).await { "Список скопирован".to_string() } else { text };
            copied.set(Some(msg));
        });
    };

    let body = move || {
        let acc = list();
        if acc.is_empty() {
            return view! { <div class="card empty">"Выбери хотя бы один день"</div> }.into_any();
        }
        let bought = store.settings.with(|s| s.shop.clone());
        let done = acc.iter().filter(|(f, _)| bought.contains(f)).count();
        let total = acc.len();
        let groups = Cat::ALL
            .into_iter()
            .filter_map(|cat| {
                let items: Vec<(FoodId, u32)> = acc.iter().copied().filter(|(f, _)| f.info().cat == cat).collect();
                if items.is_empty() {
                    return None;
                }
                let rows = items
                    .into_iter()
                    .map(|(f, g)| {
                        let on = bought.contains(&f);
                        view! {
                            <label class={if on { "chk done" } else { "chk" }}>
                                <input
                                    type="checkbox"
                                    prop:checked=on
                                    on:change=move |ev| toggle_food(f, event_target_checked(&ev))
                                />
                                <span>{f.info().name}</span>
                                <span class="q">{shop_qty(f, g)}</span>
                            </label>
                        }
                    })
                    .collect_view();
                Some(view! {
                    <div class="card grp">
                        <h3>{format!("{} {}", cat.emoji(), cat.title())}</h3>
                        {rows}
                    </div>
                })
            })
            .collect_view();
        view! {
            <div class="card">
                <div class="row">
                    <b>{format!("✔ Куплено {done} из {total}")}</b>
                    <span style="flex:1"></span>
                    <button type="button" class="btn sm" on:click=copy>"📋 Скопировать"</button>
                    <button type="button" class="btn sm" on:click=move |_| store.set(|s| s.shop.clear())>"Сбросить"</button>
                </div>
                {move || copied.get().map(|t| view! { <p class="hint copied">{t}</p> })}
                <div style="margin-top:10px">{bar(done as f64 / total as f64 * 100.0, "ok")}</div>
            </div>
            {groups}
        }
        .into_any()
    };

    view! {
        <p class="hint">
            "Всё в сыром виде (мясо, рыба, крупы сухие). Выбери дни, на которые закупаешься. Список считается по выбранному режиму плана."
        </p>
        {chips(days, move |d| store.settings.with(|s| s.shop_days.contains(&d)), toggle_day, "days")}
        {body}
    }
}
