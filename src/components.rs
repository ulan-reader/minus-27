use leptos::prelude::*;

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
