use leptos::prelude::*;
use minus27_core::foods::{Cat, CustomFood, FoodRef, catalog};

use crate::icons::icon;

/// Ряд кнопок-чипов. `cls`: "seg" (на всю ширину) или "days".
pub fn chips<T>(
    items: Vec<(T, String)>,
    is_on: impl Fn(T) -> bool + Copy + Send + Sync + 'static,
    on_pick: impl Fn(T) + Copy + Send + Sync + 'static,
    cls: &'static str,
) -> impl IntoView
where
    T: Copy + Send + Sync + 'static,
{
    view! {
        <div class={format!("chips {cls}")}>
            {items
                .into_iter()
                .map(|(v, label)| {
                    view! {
                        <button
                            type="button"
                            class=move || if is_on(v) { "chip on" } else { "chip" }
                            on:click=move |_| on_pick(v)
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Полоска прогресса, `pct` от 0 до 100.
pub fn bar(pct: f64, cls: &'static str) -> impl IntoView {
    view! {
        <span class="bar">
            <i class=cls style={format!("width:{:.1}%", pct.clamp(0.0, 100.0))}></i>
        </span>
    }
}

/// Кольцо прогресса: `value` из `target`, цвет по классу (`kcal`, `protein`, `water`).
pub fn ring(value: f64, target: f64, cls: &'static str, big: String, small: String) -> impl IntoView {
    let r = 42.0;
    let len = 2.0 * std::f64::consts::PI * r;
    let pct = if target > 0.0 { (value / target).clamp(0.0, 1.0) } else { 0.0 };
    let over = target > 0.0 && value > target * 1.05;
    // Пустую дугу не рисуем: закруглённый конец дал бы точку.
    let fg = if pct > 0.0 {
        format!(r#"<circle class="ring-fg" cx="50" cy="50" r="{r}" stroke-dasharray="{:.1} {len:.1}" transform="rotate(-90 50 50)"/>"#, pct * len)
    } else {
        String::new()
    };
    let svg = format!(r#"<svg viewBox="0 0 100 100" aria-hidden="true"><circle class="ring-bg" cx="50" cy="50" r="{r}"/>{fg}</svg>"#);
    view! {
        <div class={format!("ring {cls}{}", if over { " over" } else { "" })}>
            <div inner_html=svg></div>
            <div class="ring-c">
                <b>{big}</b>
                <span>{small}</span>
            </div>
        </div>
    }
}

/// Шторка снизу поверх экрана. Закрывается по фону и крестику.
pub fn sheet(title: String, close: impl Fn() + Copy + Send + Sync + 'static, body: impl IntoView) -> impl IntoView {
    view! {
        <div class="sheet-bg" on:click=move |_| close()>
            <div class="sheet" on:click=|ev| ev.stop_propagation()>
                <div class="sheet-h">
                    <b>{title}</b>
                    <button type="button" class="icon-btn" aria-label="Закрыть" on:click=move |_| close()>
                        {icon("x")}
                    </button>
                </div>
                {body}
            </div>
        </div>
    }
}

/// Число из поля ввода: запятая или точка, пусто = None.
pub fn num(s: &str) -> Option<f64> {
    let s = s.trim().replace(',', ".");
    s.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Список продуктов с поиском. Продукты категории `prefer` идут первыми (для замены).
pub fn food_picker(
    customs: Vec<CustomFood>,
    prefer: Option<Cat>,
    on_pick: impl Fn(FoodRef) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let mut all = catalog(&customs);
    if let Some(c) = prefer {
        all.sort_by_key(|(_, v)| v.cat != c);
    }
    view! {
        <label class="search">
            {icon("search")}
            <input
                type="search"
                placeholder="Найти продукт"
                prop:value=move || query.get()
                on:input=move |ev| query.set(event_target_value(&ev))
            />
        </label>
        <div class="pick-list">
            {move || {
                let q = query.get().to_lowercase();
                all.iter()
                    .filter(|(_, v)| q.is_empty() || v.name.to_lowercase().contains(&q))
                    .map(|(r, v)| {
                        let r = r.clone();
                        let on_pick = on_pick.clone();
                        view! {
                            <button type="button" class="pick" on:click=move |_| on_pick(r.clone())>
                                {icon(v.cat.icon())}
                                <span class="grow">{v.name.clone()}</span>
                                <span class="mut">{format!("{} ккал · Б {}", v.kcal.round(), crate::fmt::trim0(v.protein))}</span>
                            </button>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
