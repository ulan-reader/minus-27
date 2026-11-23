use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use minus27_core::{
    foods::{Cat, CustomFood},
    profile::Sex,
    state::Reminder,
};
use minus27_data::Snapshot;

use crate::{
    api,
    app::Store,
    components::{chips, num, sheet},
    fmt, icons::icon, js,
    screens::onboarding::profile_form,
};

#[derive(Clone, Copy, PartialEq)]
enum Sheet {
    Edit,
    Food,
    Export,
    Import,
}

#[component]
pub fn ProfileScreen() -> impl IntoView {
    let store = expect_context::<Store>();
    let open = RwSignal::new(None::<Sheet>);
    let close = move || open.set(None);

    let me = move || {
        let p = store.settings.with(|s| s.profile.clone())?;
        let t = store.targets()?;
        let age = js::year() - p.birth_year;
        Some(view! {
            <div class="card row">
                <span class="avatar">{p.name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_else(|| "М".into())}</span>
                <div class="grow">
                    <b>{if p.name.is_empty() { "Без имени".to_string() } else { p.name.clone() }}</b>
                    <div class="mut">
                        {format!(
                            "{}, {age} лет, {} см · цель {} кг",
                            if p.sex == Sex::Male { "мужчина" } else { "женщина" },
                            fmt::trim0(p.height_cm),
                            fmt::trim0(p.goal_weight)
                        )}
                    </div>
                </div>
                <button type="button" class="icon-btn" aria-label="Изменить профиль" on:click=move |_| open.set(Some(Sheet::Edit))>
                    {icon("edit")}
                </button>
            </div>
            <div class="card targets">
                <div class="t3">
                    <div><b>{fmt::int(t.kcal)}</b><span>"ккал"</span></div>
                    <div><b>{format!("{} г", fmt::int(t.protein))}</b><span>"белка"</span></div>
                    <div><b>{format!("{} л", fmt::liters(t.water_ml))}</b><span>"воды"</span></div>
                </div>
                <p class="hint">
                    {format!(
                        "Основной обмен {} ккал, расход с активностью {} ккал. Норма пересчитывается по среднему весу за неделю.",
                        fmt::int(t.bmr),
                        fmt::int(t.tdee)
                    )}
                </p>
            </div>
        })
    };

    view! {
        <header class="top">
            <p class="mut">"Настройки и данные"</p>
            <h1>"Профиль"</h1>
        </header>
        {me}
        <h2>"Синхронизация"</h2>
        <SyncCard/>
        {api::is_app().then(|| view! { <h2>"Напоминания"</h2> <Reminders/> })}
        <h2>"Свои продукты"</h2>
        <CustomFoods open/>
        <h2>"Данные"</h2>
        <div class="card list">
            <button type="button" class="opt" on:click=move |_| open.set(Some(Sheet::Export))>
                {icon("download")}
                <span class="grow">"Экспорт"</span>
                {icon("chevron-right")}
            </button>
            <button type="button" class="opt" on:click=move |_| open.set(Some(Sheet::Import))>
                {icon("upload")}
                <span class="grow">"Импорт"</span>
                {icon("chevron-right")}
            </button>
        </div>
        <h2>"О приложении"</h2>
        <div class="card about">
            <p>
                <b>{format!("Минус 27 · версия {}", env!("CARGO_PKG_VERSION"))}</b>
            </p>
            <p class="mut">"Открытый проект под лицензией MIT: github.com/ulan-reader/minus-27"</p>
            <p class="mut">
                "Калории и нормы приблизительные и не заменяют консультацию врача. При заболеваниях, беременности и для подростков подбирай питание со специалистом."
            </p>
        </div>
        {move || {
            let s = open.get()?;
            Some(match s {
                Sheet::Edit => {
                    let p = store.settings.with_untracked(|s| s.profile.clone()).unwrap_or_default();
                    sheet(
                        "Профиль".into(),
                        close,
                        profile_form(p, "Сохранить", move |p| {
                            store.set(|s| s.profile = Some(p));
                            store.toast("Норма пересчитана");
                            close();
                        })
                        .into_any(),
                    )
                    .into_any()
                }
                Sheet::Food => sheet("Свой продукт".into(), close, food_form(store, close).into_any()).into_any(),
                Sheet::Export => sheet("Экспорт".into(), close, export_view(store).into_any()).into_any(),
                Sheet::Import => sheet("Импорт".into(), close, import_view(store, close).into_any()).into_any(),
            })
        }}
    }
}

#[component]
fn SyncCard() -> impl IntoView {
    let store = expect_context::<Store>();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let err = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let on_login = move |ev: SubmitEvent| {
        ev.prevent_default();
        err.set(String::new());
        busy.set(true);
        spawn_local(async move {
            match api::sign_in(&email.get_untracked(), &password.get_untracked()).await {
                Ok(info) => {
                    password.set(String::new());
                    store.sync.set(info);
                    store.sync();
                }
                Err(e) => err.set(e),
            }
            busy.set(false);
        });
    };

    let on_logout = move |_: leptos::ev::MouseEvent| {
        spawn_local(async move {
            if let Ok(info) = api::sign_out().await {
                store.sync.set(info);
                store.sync_status.set(String::new());
            }
        });
    };

    move || {
        let info = store.sync.get();
        if !info.enabled {
            return view! {
                <div class="card row">
                    <span class="badge">{icon("cloud")}</span>
                    <div class="grow">
                        <b>"Данные на этом устройстве"</b>
                        <div class="mut">"Синхронизация не настроена в этой сборке."</div>
                    </div>
                </div>
            }
            .into_any();
        }
        if let Some(who) = info.account {
            return view! {
                <div class="card">
                    <div class="row">
                        <span class="badge green">{icon("cloud")}</span>
                        <div class="grow">
                            <b>{who}</b>
                            <div class="mut">{move || store.sync_status.get()}</div>
                        </div>
                        <button type="button" class="icon-btn" aria-label="Синхронизировать" on:click=move |_| store.sync()>
                            {icon("refresh")}
                        </button>
                        <button type="button" class="icon-btn" aria-label="Выйти" on:click=on_logout>
                            {icon("logout")}
                        </button>
                    </div>
                </div>
            }
            .into_any();
        }
        view! {
            <form class="card" on:submit=on_login>
                <p class="mut">"Вход нужен для синхронизации между устройствами и распознавания фото. Аккаунт выдаёт владелец приложения."</p>
                <input
                    type="email"
                    autocomplete="username"
                    required
                    placeholder="email"
                    prop:value=move || email.get()
                    on:input=move |ev| email.set(event_target_value(&ev))
                />
                <input
                    type="password"
                    autocomplete="current-password"
                    required
                    placeholder="пароль"
                    prop:value=move || password.get()
                    on:input=move |ev| password.set(event_target_value(&ev))
                />
                <button class="btn pri wide" type="submit" disabled=move || busy.get()>
                    {move || if busy.get() { "Вход…" } else { "Войти" }}
                </button>
                <p class="err">{move || err.get()}</p>
            </form>
        }
        .into_any()
    }
}

#[component]
fn Reminders() -> impl IntoView {
    let store = expect_context::<Store>();
    let save = move |list: Vec<Reminder>| {
        store.set(|s| s.reminders = list.clone());
        spawn_local(async move {
            match api::set_reminders(&list).await {
                Ok(true) => {}
                Ok(false) if list.iter().any(|r| r.on) => store.toast("Разреши уведомления в настройках телефона"),
                Ok(false) => {}
                Err(e) => store.toast(e),
            }
        });
    };
    move || {
        let list = store.settings.with(|s| s.reminders.clone());
        view! {
            <div class="card list">
                {list
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        let (l1, l2) = (list.clone(), list.clone());
                        view! {
                            <div class="opt">
                                {icon("bell")}
                                <span class="grow">{r.kind.title()}</span>
                                <input
                                    type="time"
                                    class="time"
                                    prop:value=format!("{:02}:{:02}", r.hour, r.minute)
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        let mut l = l1.clone();
                                        if let Some((h, m)) = v.split_once(':') && let (Ok(h), Ok(m)) = (h.parse(), m.parse()) {
                                            l[i].hour = h;
                                            l[i].minute = m;
                                            save(l);
                                        }
                                    }
                                />
                                <button
                                    type="button"
                                    class=if r.on { "switch on" } else { "switch" }
                                    aria-label="Включить"
                                    on:click=move |_| {
                                        let mut l = l2.clone();
                                        l[i].on = !l[i].on;
                                        save(l);
                                    }
                                >
                                    <i></i>
                                </button>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        }
    }
}

#[component]
fn CustomFoods(open: RwSignal<Option<Sheet>>) -> impl IntoView {
    let store = expect_context::<Store>();
    move || {
        let foods = store.settings.with(|s| s.custom_foods.clone());
        view! {
            <div class="card list">
                {foods
                    .into_iter()
                    .map(|f| {
                        let id = f.id.clone();
                        view! {
                            <div class="opt">
                                {icon(f.cat.icon())}
                                <span class="grow">{f.name}</span>
                                <span class="mut">{format!("{} ккал · Б {}", fmt::int(f.kcal), fmt::trim0(f.protein))}</span>
                                <button
                                    type="button"
                                    class="icon-btn sm"
                                    aria-label="Удалить"
                                    on:click=move |_| {
                                        let id = id.clone();
                                        store.set(|s| s.custom_foods.retain(|c| c.id != id));
                                    }
                                >
                                    {icon("trash")}
                                </button>
                            </div>
                        }
                    })
                    .collect_view()}
                <button type="button" class="opt" on:click=move |_| open.set(Some(Sheet::Food))>
                    {icon("plus")}
                    <span class="grow">"Добавить продукт"</span>
                </button>
            </div>
            <p class="hint">"Свои продукты появятся в поиске: в плане, при замене и в дневнике. Значения на 100 г — с упаковки."</p>
        }
    }
}

fn food_form(store: Store, close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let kcal = RwSignal::new(String::new());
    let protein = RwSignal::new(String::new());
    let cat = RwSignal::new(Cat::Other);
    let err = RwSignal::new(String::new());
    view! {
        <label class="f">
            <span>"Название"</span>
            <input type="text" placeholder="Протеиновый батончик" prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev))/>
        </label>
        <div class="grid2">
            <label class="f">
                <span>"Ккал на 100 г"</span>
                <input type="text" inputmode="decimal" prop:value=move || kcal.get() on:input=move |ev| kcal.set(event_target_value(&ev))/>
            </label>
            <label class="f">
                <span>"Белок на 100 г"</span>
                <input type="text" inputmode="decimal" prop:value=move || protein.get() on:input=move |ev| protein.set(event_target_value(&ev))/>
            </label>
        </div>
        {chips(Cat::ALL.into_iter().map(|c| (c, c.title().to_string())).collect(), move |c| cat.get() == c, move |c| cat.set(c), "")}
        <button
            type="button"
            class="btn pri wide"
            on:click=move |_| {
                let n = name.get_untracked().trim().to_string();
                let (Some(k), Some(p)) = (num(&kcal.get_untracked()), num(&protein.get_untracked()).or(Some(0.0))) else {
                    err.set("Укажи калории".into());
                    return;
                };
                if n.is_empty() || !(0.0..=950.0).contains(&k) || !(0.0..=100.0).contains(&p) {
                    err.set("Проверь название и числа".into());
                    return;
                }
                let food = CustomFood { id: format!("c{:x}", js::now_ms()), name: n, kcal: k, protein: p, cat: cat.get_untracked() };
                store.set(|s| s.custom_foods.push(food));
                close();
            }
        >
            "Добавить"
        </button>
        <p class="err">{move || err.get()}</p>
    }
}

fn export_view(store: Store) -> impl IntoView {
    let snap = Snapshot {
        settings: store.settings.get_untracked(),
        entries: store.entries.get_untracked(),
        days: store.days.get_untracked(),
    };
    let json = serde_json::to_string_pretty(&snap).unwrap_or_default();
    let csv = std::iter::once("date,weight_kg,waist_cm,hips_cm,chest_cm,steps".to_string())
        .chain(snap.entries.iter().map(|e| {
            let o = |v: Option<f64>| v.map(|x| x.to_string()).unwrap_or_default();
            format!("{},{},{},{},{},{}", e.date, e.weight, o(e.waist), o(e.hips), o(e.chest), e.steps.map(|s| s.to_string()).unwrap_or_default())
        }))
        .collect::<Vec<_>>()
        .join("\n");
    let (j, c) = (json.clone(), csv.clone());
    view! {
        <p class="hint">"Резервная копия всех данных. Скопируй и сохрани в заметки или файл: её можно вставить в «Импорт» на любом устройстве."</p>
        <div class="grid2">
            <button
                type="button"
                class="btn pri"
                on:click=move |_| {
                    let j = j.clone();
                    spawn_local(async move { store.toast(if js::copy(&j).await { "JSON скопирован" } else { "Выдели текст ниже вручную" }) });
                }
            >
                {icon("copy")}
                "JSON"
            </button>
            <button
                type="button"
                class="btn"
                on:click=move |_| {
                    let c = c.clone();
                    spawn_local(async move { store.toast(if js::copy(&c).await { "CSV веса скопирован" } else { "Не удалось скопировать" }) });
                }
            >
                {icon("copy")}
                "CSV веса"
            </button>
        </div>
        <textarea class="code" readonly rows="8">{json}</textarea>
    }
}

fn import_view(store: Store, close: impl Fn() + Copy + Send + Sync + 'static) -> impl IntoView {
    let text = RwSignal::new(String::new());
    let err = RwSignal::new(String::new());
    view! {
        <p class="hint">"Вставь JSON из «Экспорта». Текущие данные на устройстве заменятся."</p>
        <textarea class="code" rows="8" placeholder="{ \"settings\": … }" prop:value=move || text.get() on:input=move |ev| text.set(event_target_value(&ev))></textarea>
        <button
            type="button"
            class="btn pri wide"
            on:click=move |_| {
                let snap: Snapshot = match serde_json::from_str(&text.get_untracked()) {
                    Ok(s) => s,
                    Err(_) => {
                        err.set("Это не похоже на экспорт из приложения".into());
                        return;
                    }
                };
                spawn_local(async move {
                    match api::import_data(&snap).await {
                        Ok(s) => {
                            store.apply(s);
                            store.sync();
                            store.toast("Данные загружены");
                            close();
                        }
                        Err(e) => err.set(e),
                    }
                });
            }
        >
            "Загрузить"
        </button>
        <p class="err">{move || err.get()}</p>
    }
}
