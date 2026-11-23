use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use minus27_core::{
    diary::{self, DayLog},
    plan::PlanDay,
    profile::Targets,
    state::{Settings, SyncInfo, Tab},
    weight::{self, Entry},
};
use minus27_data::Snapshot;

use crate::{
    api, icons::icon, js,
    screens::{onboarding::Onboarding, plan::PlanScreen, profile::ProfileScreen, progress::ProgressScreen, shop::ShopScreen, today::TodayScreen},
};

/// Общее состояние приложения. Copy, поэтому его можно свободно таскать в замыкания.
#[derive(Clone, Copy)]
pub struct Store {
    pub settings: RwSignal<Settings>,
    pub entries: RwSignal<Vec<Entry>>,
    pub days: RwSignal<Vec<DayLog>>,
    pub sync: RwSignal<SyncInfo>,
    /// Строка в профиле: «Синхронизировано в 14:05» или ошибка.
    pub sync_status: RwSignal<String>,
    pub loaded: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    /// Короткое сообщение внизу экрана.
    pub toast: RwSignal<Option<String>>,
}

impl Store {
    /// Изменить настройки и сохранить их. Если поменялось что-то кроме вкладки и дня — синк.
    pub fn set(&self, f: impl FnOnce(&mut Settings)) {
        self.settings.update(f);
        let s = self.settings.get_untracked();
        let st = *self;
        spawn_local(async move {
            match api::save_settings(&s).await {
                Ok(saved) => {
                    let changed = saved.updated_ms != s.updated_ms;
                    st.settings.update(|cur| cur.updated_ms = saved.updated_ms);
                    if changed {
                        st.sync();
                    }
                }
                Err(e) => st.error.set(Some(e)),
            }
        });
    }

    pub fn apply(&self, snap: Snapshot) {
        self.settings.set(snap.settings);
        self.entries.set(snap.entries);
        self.days.set(snap.days);
    }

    /// Изменить день дневника: сразу на экране, потом сохранить и синхронизировать.
    pub fn update_day(&self, date: &str, f: impl FnOnce(&mut DayLog)) {
        let mut day = self.days.with_untracked(|ds| ds.iter().find(|d| d.date == date).cloned()).unwrap_or_else(|| DayLog::new(date));
        f(&mut day);
        self.days.update(|ds| diary::upsert(ds, day.clone()));
        let st = *self;
        spawn_local(async move {
            match api::save_day(&day).await {
                Ok(list) => {
                    st.days.set(list);
                    st.sync();
                }
                Err(e) => st.toast(e),
            }
        });
    }

    pub fn today(&self) -> DayLog {
        let date = js::today_iso();
        self.days.with(|ds| ds.iter().find(|d| d.date == date).cloned()).unwrap_or_else(|| DayLog::new(&date))
    }

    /// Текущий вес: среднее за последние 7 дней.
    pub fn weight_now(&self) -> Option<f64> {
        self.entries.with(|e| weight::rolling(&weight::points(e)).last().copied())
    }

    pub fn targets(&self) -> Option<Targets> {
        let w = self.weight_now();
        self.settings.with(|s| s.targets(w, js::year()))
    }

    pub fn plan(&self) -> Vec<PlanDay> {
        let w = self.weight_now();
        self.settings.with(|s| s.plan(w, js::year()))
    }

    pub fn toast(&self, msg: impl Into<String>) {
        self.toast.set(Some(msg.into()));
        let t = self.toast;
        set_timeout(move || t.set(None), Duration::from_millis(2600));
    }

    /// Синхронизировать с Supabase, если он включён и выполнен вход.
    pub fn sync(&self) {
        if !self.sync.with_untracked(|s| s.enabled && s.account.is_some()) {
            return;
        }
        let st = *self;
        st.sync_status.set("Синхронизация…".into());
        spawn_local(async move {
            match api::sync_now().await {
                Ok(snap) => {
                    st.apply(snap);
                    st.sync_status.set(format!("Синхронизировано в {}", js::time_hm()));
                }
                Err(e) => {
                    // Сессия могла слететь: обновить карточку входа.
                    if let Ok(info) = api::sync_info().await {
                        st.sync.set(info);
                    }
                    st.sync_status.set(format!("Ошибка: {e}"));
                }
            }
        });
    }
}

#[component]
pub fn App() -> impl IntoView {
    let store = Store {
        settings: RwSignal::new(Settings::default()),
        entries: RwSignal::new(Vec::new()),
        days: RwSignal::new(Vec::new()),
        sync: RwSignal::new(SyncInfo::default()),
        sync_status: RwSignal::new(String::new()),
        loaded: RwSignal::new(false),
        error: RwSignal::new(None),
        toast: RwSignal::new(None),
    };
    provide_context(store);

    spawn_local(async move {
        match api::load().await {
            Ok(snap) => {
                store.apply(snap);
                store.loaded.set(true);
            }
            Err(e) => store.error.set(Some(e)),
        }
        if let Ok(info) = api::sync_info().await {
            store.sync.set(info);
            store.sync();
        }
    });

    let tab = move || store.settings.with(|s| s.tab);
    let go = move |t: Tab| {
        store.set(|s| s.tab = t);
        js::scroll_top();
    };

    view! {
        <div class="app">
            {move || store.error.get().map(|e| view! { <div class="card err">{e}</div> })}
            {move || {
                if !store.loaded.get() {
                    return view! { <div class="loading">"Загрузка…"</div> }.into_any();
                }
                if store.settings.with(|s| s.profile.is_none()) {
                    return view! { <Onboarding/> }.into_any();
                }
                view! {
                    <main class="screen">
                        {move || match tab() {
                            Tab::Today => view! { <TodayScreen/> }.into_any(),
                            Tab::Plan => view! { <PlanScreen/> }.into_any(),
                            Tab::Shop => view! { <ShopScreen/> }.into_any(),
                            Tab::Progress => view! { <ProgressScreen/> }.into_any(),
                            Tab::Profile => view! { <ProfileScreen/> }.into_any(),
                        }}
                    </main>
                    <nav class="tabbar">
                        {Tab::ALL
                            .into_iter()
                            .map(|(t, label, ic)| {
                                view! {
                                    <button
                                        type="button"
                                        class=move || if tab() == t { "on" } else { "" }
                                        on:click=move |_| go(t)
                                    >
                                        {icon(ic)}
                                        <span>{label}</span>
                                    </button>
                                }
                            })
                            .collect_view()}
                    </nav>
                }
                .into_any()
            }}
            {move || store.toast.get().map(|t| view! { <div class="toast">{t}</div> })}
        </div>
    }
}
