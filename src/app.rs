use leptos::prelude::*;
use leptos::task::spawn_local;
use minus27_core::{
    state::{Settings, SyncInfo, Tab},
    weight::Entry,
};

use crate::{
    api, js,
    tabs::{plan::PlanTab, shop::ShopTab, weight::WeightTab},
};

/// Общее состояние приложения. Copy, поэтому его можно свободно таскать в замыкания.
#[derive(Clone, Copy)]
pub struct Store {
    pub settings: RwSignal<Settings>,
    pub entries: RwSignal<Vec<Entry>>,
    pub sync: RwSignal<SyncInfo>,
    /// Строка под карточкой синка: «Синхронизировано в 14:05» или ошибка.
    pub sync_status: RwSignal<String>,
    pub loaded: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}

impl Store {
    /// Изменить настройки и сохранить их на устройстве.
    pub fn set(&self, f: impl FnOnce(&mut Settings)) {
        self.settings.update(f);
        let s = self.settings.get_untracked();
        let error = self.error;
        spawn_local(async move {
            if let Err(e) = api::save_settings(&s).await {
                error.set(Some(e));
            }
        });
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
                Ok(list) => {
                    st.entries.set(list);
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
        sync: RwSignal::new(SyncInfo::default()),
        sync_status: RwSignal::new(String::new()),
        loaded: RwSignal::new(false),
        error: RwSignal::new(None),
    };
    provide_context(store);

    spawn_local(async move {
        match api::load().await {
            Ok(snap) => {
                store.settings.set(snap.settings);
                store.entries.set(snap.entries);
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

    view! {
        <div class="wrap">
            <header>
                <h1>"Минус 27 🔥"</h1>
                <p>"97 → 70 кг. План, покупки и трекер веса."</p>
            </header>
            <nav>
                {Tab::ALL
                    .into_iter()
                    .map(|(t, label)| {
                        view! {
                            <button
                                type="button"
                                class=move || if tab() == t { "on" } else { "" }
                                on:click=move |_| store.set(|s| s.tab = t)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </nav>
            {move || store.error.get().map(|e| view! { <div class="card err">{e}</div> })}
            <main>
                {move || {
                    if !store.loaded.get() {
                        return view! { <div class="card empty">"Загрузка…"</div> }.into_any();
                    }
                    match tab() {
                        Tab::Plan => view! { <PlanTab/> }.into_any(),
                        Tab::Shop => view! { <ShopTab/> }.into_any(),
                        Tab::Weight => view! { <WeightTab/> }.into_any(),
                    }
                }}
            </main>
        </div>
    }
}
