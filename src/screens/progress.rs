use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use minus27_core::weight::{self, Entry, Eta};

use crate::{api, app::Store, chart, fmt, icons::icon, js};

/// Поле формы → число. Пусто = `Ok(None)`, мусор = `Err`.
fn parse(s: &str) -> Result<Option<f64>, ()> {
    let s = s.trim().replace(',', ".");
    if s.is_empty() {
        return Ok(None);
    }
    s.parse::<f64>().ok().filter(|v| v.is_finite()).map(Some).ok_or(())
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

#[component]
pub fn ProgressScreen() -> impl IntoView {
    let store = expect_context::<Store>();
    // Форма общая: её заполняет и сама форма, и кнопка правки в таблице записей.
    provide_context(Form {
        date: RwSignal::new(js::today_iso()),
        weight: RwSignal::new(String::new()),
        waist: RwSignal::new(String::new()),
        hips: RwSignal::new(String::new()),
        chest: RwSignal::new(String::new()),
        steps: RwSignal::new(String::new()),
    });
    let goal = move || store.settings.with(|s| s.profile.as_ref().map(|p| p.goal_weight).unwrap_or(s.goal));
    view! {
        <header class="top">
            <p class="mut">{move || format!("Цель {} кг", fmt::trim0(goal()))}</p>
            <h1>"Прогресс"</h1>
        </header>
        <EntryForm/>
        {move || {
            if store.entries.with(|e| e.is_empty()) {
                view! { <div class="card empty">"Записей пока нет. Первая запись — твоя точка отсчёта."</div> }.into_any()
            } else {
                view! { <Analytics/> }.into_any()
            }
        }}
    }
}

/// Поля формы записи, строками как в input.
#[derive(Clone, Copy)]
struct Form {
    date: RwSignal<String>,
    weight: RwSignal<String>,
    waist: RwSignal<String>,
    hips: RwSignal<String>,
    chest: RwSignal<String>,
    steps: RwSignal<String>,
}

impl Form {
    fn fill(&self, date: &str, entries: &[Entry]) {
        self.date.set(date.to_string());
        match entries.iter().find(|e| e.date == date) {
            Some(e) => {
                self.weight.set(e.weight.to_string());
                self.waist.set(e.waist.map(|v| v.to_string()).unwrap_or_default());
                self.hips.set(e.hips.map(|v| v.to_string()).unwrap_or_default());
                self.chest.set(e.chest.map(|v| v.to_string()).unwrap_or_default());
                self.steps.set(e.steps.map(|v| v.to_string()).unwrap_or_default());
            }
            None => {
                self.weight.set(String::new());
                self.waist.set(String::new());
                self.hips.set(String::new());
                self.chest.set(String::new());
                self.steps.set(String::new());
            }
        }
    }
}

#[component]
fn EntryForm() -> impl IntoView {
    let store = expect_context::<Store>();
    let form = expect_context::<Form>();
    let err = RwSignal::new(String::new());

    let exists = move || {
        let d = form.date.get();
        store.entries.with(|es| es.iter().any(|e| e.date == d))
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        err.set(String::new());
        let (Ok(Some(w)), Ok(waist), Ok(hips), Ok(chest), Ok(steps)) = (
            parse(&form.weight.get_untracked()),
            parse(&form.waist.get_untracked()),
            parse(&form.hips.get_untracked()),
            parse(&form.chest.get_untracked()),
            parse(&form.steps.get_untracked()),
        ) else {
            err.set("Проверь числа: вес обязателен, остальное можно оставить пустым".into());
            return;
        };
        if steps.is_some_and(|s| s < 0.0) {
            err.set("Шаги: от 0 до 100 000".into());
            return;
        }
        let entry = Entry {
            date: form.date.get_untracked(),
            weight: round1(w),
            waist: waist.map(round1),
            hips: hips.map(round1),
            chest: chest.map(round1),
            steps: steps.map(|s| s.round() as u32),
            updated_ms: 0,
        };
        if let Err(e) = weight::check(&entry, Some(js::today_num())) {
            err.set(e.into());
            return;
        }
        spawn_local(async move {
            match api::save_entry(&entry).await {
                Ok(list) => {
                    store.entries.set(list);
                    form.fill(&js::today_iso(), &[]);
                    store.sync();
                }
                Err(e) => err.set(e),
            }
        });
    };

    let field = move |label: &'static str, sig: RwSignal<String>, unit: &'static str, ph: &'static str| {
        view! {
            <label class="f">
                <span>{label}</span>
                <div class="unit">
                    <input
                        type="text"
                        inputmode="decimal"
                        placeholder=ph
                        prop:value=move || sig.get()
                        on:input=move |ev| sig.set(event_target_value(&ev))
                    />
                    <em>{unit}</em>
                </div>
            </label>
        }
    };

    view! {
        <form class="card" on:submit=on_submit>
            <div class="row">
                <span class="badge green">{icon("scale")}</span>
                <b class="grow">"Запись"</b>
                <input
                    type="date"
                    class="date"
                    required
                    prop:value=move || form.date.get()
                    on:change=move |ev| {
                        let d = event_target_value(&ev);
                        store.entries.with_untracked(|es| form.fill(&d, es));
                    }
                />
            </div>
            <div class="grid2">
                {field("Вес", form.weight, "кг", "97,0")}
                {field("Шаги", form.steps, "", "необяз.")}
            </div>
            <details class="more">
                <summary>{icon("ruler")} "Замеры тела"</summary>
                <div class="grid3">
                    {field("Талия", form.waist, "см", "")}
                    {field("Бёдра", form.hips, "см", "")}
                    {field("Грудь", form.chest, "см", "")}
                </div>
            </details>
            <button type="submit" class="btn pri wide">{move || if exists() { "Обновить" } else { "Сохранить" }}</button>
            <p class="err">{move || err.get()}</p>
            <p class="hint">
                "Взвешивайся утром: после туалета, до еды и воды, на одних весах. Замеры — раз в неделю, на выдохе."
            </p>
        </form>
    }
}

#[component]
fn Analytics() -> impl IntoView {
    let store = expect_context::<Store>();
    let form = expect_context::<Form>();
    // Дата, по которой один раз нажали ✕: второе нажатие удаляет.
    let confirm_del = RwSignal::new(None::<String>);
    let err = RwSignal::new(String::new());

    let delete = move |date: String| {
        if confirm_del.get_untracked().as_deref() != Some(date.as_str()) {
            confirm_del.set(Some(date));
            return;
        }
        confirm_del.set(None);
        spawn_local(async move {
            match api::delete_entry(&date).await {
                Ok(list) => {
                    store.entries.set(list);
                    store.sync();
                }
                Err(e) => err.set(e),
            }
        });
    };

    move || {
        let list = store.entries.with(|e| weight::points(e));
        let goal = store.settings.with(|s| s.profile.as_ref().map(|p| p.goal_weight).unwrap_or(s.goal));
        let roll = weight::rolling(&list);
        let weeks = weight::weekly_blocks(&list);
        let v = weight::verdict(&weeks);
        let pc = weight::pace(&weight::week_losses(&weeks));
        let (Some(&cur), Some(first_week), Some(last)) = (roll.last(), weeks.first(), list.last()) else {
            return ().into_any();
        };
        let start = first_week.avg;
        let start_cls = if start - cur > 0.0 { "dn" } else { "" };
        let left = cur - goal;
        let (eta, eta_sub) = match weight::eta(last.n, cur, goal, pc) {
            Eta::Reached => ("Готово".to_string(), "цель достигнута".to_string()),
            Eta::At { day, weeks } => {
                (fmt::month_year(day), format!("≈ {} нед. по текущему темпу, на деле дольше", weeks.round()))
            }
            Eta::Flat => ("—".to_string(), "темп пока около нуля".to_string()),
            Eta::NoData => ("—".to_string(), "нужно 2 недели данных".to_string()),
        };

        let week_rows = weeks
            .iter()
            .enumerate()
            .rev()
            .map(|(i, b)| {
                let d = (i > 0).then(|| b.avg - weeks[i - 1].avg);
                view! {
                    <tr>
                        <td>{format!("№{}{}", b.index + 1, if b.n < 3 { " *" } else { "" })}</td>
                        <td>{format!("{}–{}", fmt::day(b.from), fmt::day(b.to))}</td>
                        <td>{fmt::f1(b.avg)}</td>
                        <td class={fmt::delta_class(d)}>{d.map(fmt::signed).unwrap_or("—".into())}</td>
                        <td>{b.steps.map(|s| fmt::thousands(s.round() as i64)).unwrap_or("—".into())}</td>
                        <td>{b.waist.map(fmt::f1).unwrap_or("—".into())}</td>
                    </tr>
                }
            })
            .collect_view();

        let entry_rows = list
            .iter()
            .enumerate()
            .rev()
            .map(|(i, p)| {
                let e = p.entry.clone();
                let d = (i > 0).then(|| e.weight - list[i - 1].entry.weight);
                let (edit_date, del_date, cmp_date) = (e.date.clone(), e.date.clone(), e.date.clone());
                view! {
                    <tr>
                        <td>{fmt::date(&e.date)}</td>
                        <td>{fmt::f1(e.weight)}</td>
                        <td class={fmt::delta_class(d)}>{d.map(fmt::signed).unwrap_or("—".into())}</td>
                        <td>{e.waist.map(fmt::f1).unwrap_or("—".into())}</td>
                        <td>{e.steps.map(|s| fmt::thousands(s as i64)).unwrap_or("—".into())}</td>
                        <td>
                            <button
                                type="button"
                                class="icon-btn sm"
                                aria-label="Изменить"
                                on:click=move |_| {
                                    store.entries.with_untracked(|es| form.fill(&edit_date, es));
                                    js::scroll_top();
                                }
                            >
                                {icon("edit")}
                            </button>
                            <button
                                type="button"
                                class="icon-btn sm"
                                aria-label="Удалить"
                                on:click=move |_| delete(del_date.clone())
                            >
                                {move || {
                                    if confirm_del.get().as_deref() == Some(cmp_date.as_str()) {
                                        view! { <span class="danger-text">"Точно?"</span> }.into_any()
                                    } else {
                                        icon("trash").into_any()
                                    }
                                }}
                            </button>
                        </td>
                    </tr>
                }
            })
            .collect_view();

        let chart = match chart::svg(&list, &roll, goal) {
            Some(svg) => view! { <div inner_html=svg></div> }.into_any(),
            None => view! { <div class="empty">"График появится после двух записей"</div> }.into_any(),
        };

        view! {
            <div class="stats">
                <div class="stat">
                    <b>{format!("{} кг", fmt::f1(cur))}</b>
                    <span>"Сейчас (среднее за 7 дней)"</span>
                </div>
                <div class="stat">
                    <b class=start_cls>{format!("{} кг", fmt::signed(cur - start))}</b>
                    <span>{format!("От старта ({}, 1-я неделя)", fmt::f1(start))}</span>
                </div>
                <div class="stat">
                    <b>{if left > 0.0 { format!("{} кг", fmt::f1(left)) } else { "0".into() }}</b>
                    <span>{format!("До цели {} кг", fmt::trim0(goal))}</span>
                </div>
                <div class="stat">
                    <b>{pc.map(|p| format!("{} кг/нед", fmt::f1(p))).unwrap_or("—".into())}</b>
                    <span>"Темп (последние недели)"</span>
                </div>
                <div class="stat">
                    <b class="sm">{eta}</b>
                    <span>{eta_sub}</span>
                </div>
            </div>
            <div class={format!("card verdict {}", v.level.css())}>
                <b>{icon(match v.level { weight::Level::Ok => "check-circle", weight::Level::Warn => "flame", weight::Level::Info => "info" })} {v.title}</b>
                {v.text}
            </div>
            <div class="card">
                <b>"Вес"</b>
                <p class="hint">"Точки — каждый день, зелёная линия — среднее за 7 дней, пунктир — цель."</p>
                {chart}
            </div>
            {measures(&list)}
            <h2>"По неделям"</h2>
            <div class="card scroll">
                <table>
                    <thead>
                        <tr>
                            <th>"Неделя"</th>
                            <th>"Период"</th>
                            <th>"Ср. вес"</th>
                            <th>"Δ"</th>
                            <th>"Шаги"</th>
                            <th>"Талия"</th>
                        </tr>
                    </thead>
                    <tbody>{week_rows}</tbody>
                </table>
            </div>
            <p class="hint">"* меньше 3 взвешиваний: в выводы не идёт."</p>
            <h2>"Все записи"</h2>
            <span class="err">{move || err.get()}</span>
            <div class="card scroll">
                <table>
                    <thead>
                        <tr>
                            <th>"Дата"</th>
                            <th>"Вес"</th>
                            <th>"Δ"</th>
                            <th>"Талия"</th>
                            <th>"Шаги"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>{entry_rows}</tbody>
                </table>
            </div>
        }
        .into_any()
    }
}

/// Замеры тела: последнее значение и разница с первым замером.
fn measures(list: &[weight::Point]) -> impl IntoView + use<> {
    let rows: Vec<(&'static str, f64, f64)> = [
        ("Талия", list.iter().filter_map(|p| p.entry.waist).collect::<Vec<_>>()),
        ("Бёдра", list.iter().filter_map(|p| p.entry.hips).collect()),
        ("Грудь", list.iter().filter_map(|p| p.entry.chest).collect()),
    ]
    .into_iter()
    .filter_map(|(n, v)| Some((n, *v.last()?, v.last()? - v.first()?)))
    .collect();
    (!rows.is_empty()).then(|| {
        view! {
            <div class="card">
                <b>"Замеры"</b>
                <div class="t3">
                    {rows
                        .into_iter()
                        .map(|(n, last, d)| {
                            view! {
                                <div>
                                    <b>{format!("{} см", fmt::trim0(last))}</b>
                                    <span>{n}</span>
                                    <span class={fmt::delta_class(Some(d))}>{fmt::signed(d)}</span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        }
    })
}
