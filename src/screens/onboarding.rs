use leptos::prelude::*;
use minus27_core::profile::{Activity, Pace, Profile, Sex};

use crate::{
    app::Store,
    components::{chips, num},
    fmt, icons::icon, js,
};

/// Анкета при первом запуске.
#[component]
pub fn Onboarding() -> impl IntoView {
    let store = expect_context::<Store>();
    // Подсказки из первой версии: цель и последний вес, если уже были записи.
    let mut initial = Profile::default();
    initial.goal_weight = store.settings.with_untracked(|s| s.goal);
    if let Some(w) = store.entries.with_untracked(|e| e.last().map(|x| x.weight)) {
        initial.start_weight = w;
    }
    view! {
        <div class="screen onboarding">
            <div class="hero">
                {icon("flame")}
                <h1>"Минус 27"</h1>
                <p>"План питания, дневник и прогресс в одном месте. Расскажи о себе, и я посчитаю твою норму."</p>
            </div>
            {profile_form(initial, "Начать", move |p| store.set(|s| s.profile = Some(p)))}
        </div>
    }
}

/// Форма профиля с живым расчётом нормы. Используется в анкете и в «Профиле».
pub fn profile_form(initial: Profile, submit: &'static str, on_save: impl Fn(Profile) + Copy + Send + Sync + 'static) -> impl IntoView {
    let year = js::year();
    let name = RwSignal::new(initial.name.clone());
    let sex = RwSignal::new(initial.sex);
    let age = RwSignal::new((year - initial.birth_year).to_string());
    let height = RwSignal::new(fmt::trim0(initial.height_cm));
    let weight = RwSignal::new(fmt::trim0(initial.start_weight));
    let goal = RwSignal::new(fmt::trim0(initial.goal_weight));
    let activity = RwSignal::new(initial.activity);
    let pace = RwSignal::new(initial.pace);
    let err = RwSignal::new(String::new());

    let build = move || -> Result<Profile, &'static str> {
        let age = num(&age.get()).ok_or("Укажи возраст")?.round() as i32;
        let p = Profile {
            name: name.get().trim().to_string(),
            sex: sex.get(),
            birth_year: year - age,
            height_cm: num(&height.get()).ok_or("Укажи рост")?,
            start_weight: num(&weight.get()).ok_or("Укажи вес")?,
            goal_weight: num(&goal.get()).ok_or("Укажи цель")?,
            activity: activity.get(),
            pace: pace.get(),
        };
        p.check(year)?;
        Ok(p)
    };

    let field = |label: &'static str, sig: RwSignal<String>, unit: &'static str| {
        view! {
            <label class="f">
                <span>{label}</span>
                <div class="unit">
                    <input
                        type="text"
                        inputmode="decimal"
                        prop:value=move || sig.get()
                        on:input=move |ev| sig.set(event_target_value(&ev))
                    />
                    <em>{unit}</em>
                </div>
            </label>
        }
    };

    view! {
        <div class="card">
            <label class="f">
                <span>"Имя"</span>
                <input
                    type="text"
                    placeholder="Как к тебе обращаться"
                    prop:value=move || name.get()
                    on:input=move |ev| name.set(event_target_value(&ev))
                />
            </label>
            {chips(
                vec![(Sex::Male, "Мужчина".to_string()), (Sex::Female, "Женщина".to_string())],
                move |v| sex.get() == v,
                move |v| sex.set(v),
                "seg",
            )}
            <div class="grid3">
                {field("Возраст", age, "лет")}
                {field("Рост", height, "см")}
                {field("Вес", weight, "кг")}
            </div>
            {field("Цель", goal, "кг")}
        </div>
        <h2>"Активность"</h2>
        <div class="card list">
            {Activity::ALL
                .into_iter()
                .map(|a| {
                    view! {
                        <button
                            type="button"
                            class=move || if activity.get() == a { "opt on" } else { "opt" }
                            on:click=move |_| activity.set(a)
                        >
                            {move || icon(if activity.get() == a { "check-circle" } else { "circle" })}
                            <span>{a.title()}</span>
                        </button>
                    }
                })
                .collect_view()}
        </div>
        <h2>"Темп"</h2>
        {chips(Pace::ALL.into_iter().map(|p| (p, p.title().to_string())).collect(), move |v| pace.get() == v, move |v| pace.set(v), "stack")}
        {move || match build() {
            Ok(p) => {
                let t = p.targets(p.start_weight, year);
                let weeks = ((p.start_weight - p.goal_weight).max(0.0) / p.pace.kg_per_week()).round();
                view! {
                    <div class="card targets">
                        <div class="t3">
                            <div><b>{fmt::int(t.kcal)}</b><span>"ккал в день"</span></div>
                            <div><b>{format!("{} г", fmt::int(t.protein))}</b><span>"белка"</span></div>
                            <div><b>{format!("{} л", fmt::liters(t.water_ml))}</b><span>"воды"</span></div>
                        </div>
                        <p class="hint">
                            {format!(
                                "Расход ≈ {} ккал. {}",
                                fmt::int(t.tdee),
                                if weeks > 0.0 { format!("До цели ≈ {weeks:.0} нед. при выбранном темпе.") } else { "Цель достигнута, держим вес.".into() }
                            )}
                        </p>
                    </div>
                }
                .into_any()
            }
            Err(e) => view! { <p class="hint">{e}</p> }.into_any(),
        }}
        <button
            type="button"
            class="btn pri wide"
            on:click=move |_| match build() {
                Ok(p) => on_save(p),
                Err(e) => err.set(e.into()),
            }
        >
            {submit}
        </button>
        <p class="err">{move || err.get()}</p>
        <p class="hint center">"Расчёт по формуле Миффлина — Сан Жеора. Это ориентир, а не медицинская рекомендация."</p>
    }
}
