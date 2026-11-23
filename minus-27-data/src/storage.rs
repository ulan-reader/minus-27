//! Хранение данных: в приложении JSON-файл (`JsonFile`), на сайте localStorage (`web::LocalStore`).
//! Чтобы перейти, например, на SQLite, достаточно новой реализации `Store`.

use std::{
    io,
    sync::{Mutex, MutexGuard},
};

use minus27_core::{
    diary::{self, DayLog},
    state::Settings,
    weight::{self, Entry},
};
use serde::{Deserialize, Serialize};

use crate::sync::Session;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Data {
    pub settings: Settings,
    pub entries: Vec<Entry>,
    /// Дневник питания и вода, по дням.
    pub days: Vec<DayLog>,
    /// Изменения, ещё не отправленные на сервер. Копятся всегда,
    /// чтобы при включении синка ничего не потерялось.
    pub pending: Vec<Pending>,
    /// Вход в Supabase, если выполнен.
    pub session: Option<Session>,
    /// Под каким пользователем уже был синк. Новый пользователь получает все локальные записи.
    pub linked: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Pending {
    /// Что изменилось. В первой версии было только «вес», поэтому по умолчанию он.
    #[serde(default)]
    pub kind: Kind,
    /// Дата для веса и дней, пусто для настроек.
    pub date: String,
    pub op: Op,
    pub t: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Weight,
    Day,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Up,
    Del,
}

impl Data {
    pub fn mark(&mut self, kind: Kind, date: &str, op: Op, t: i64) {
        self.pending.retain(|p| !(p.kind == kind && p.date == date));
        self.pending.push(Pending { kind, date: date.to_string(), op, t });
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot { settings: self.settings.clone(), entries: self.entries.clone(), days: self.days.clone() }
    }
}

/// Все данные для UI: при старте и после синка.
#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub settings: Settings,
    pub entries: Vec<Entry>,
    #[serde(default)]
    pub days: Vec<DayLog>,
}

pub trait Store: Send + Sync {
    fn load(&self) -> io::Result<Data>;
    fn save(&self, data: &Data) -> io::Result<()>;
}

#[cfg(not(target_arch = "wasm32"))]
pub use file::JsonFile;

#[cfg(not(target_arch = "wasm32"))]
mod file {
    use std::{fs, io, path::PathBuf};

    use super::{Data, Store};

    /// `data.json` в каталоге данных приложения.
    pub struct JsonFile {
        path: PathBuf,
    }

    impl JsonFile {
        pub fn new(path: PathBuf) -> Self {
            Self { path }
        }
    }

    impl Store for JsonFile {
        fn load(&self) -> io::Result<Data> {
            let text = match fs::read_to_string(&self.path) {
                Ok(t) => t,
                Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Data::default()),
                Err(e) => return Err(e),
            };
            match serde_json::from_str(&text) {
                Ok(d) => Ok(d),
                Err(_) => {
                    // Битый файл не затираем, а откладываем рядом и начинаем с пустого.
                    fs::rename(&self.path, self.path.with_extension("json.bad"))?;
                    Ok(Data::default())
                }
            }
        }

        fn save(&self, data: &Data) -> io::Result<()> {
            if let Some(dir) = self.path.parent() {
                fs::create_dir_all(dir)?;
            }
            // Сначала во временный файл, потом rename: при сбое старые данные останутся целыми.
            let tmp = self.path.with_extension("json.tmp");
            fs::write(&tmp, serde_json::to_vec_pretty(data)?)?;
            fs::rename(&tmp, &self.path)
        }
    }
}

/// Данные в памяти плюс место, куда их сохранять.
pub struct Db {
    store: Box<dyn Store>,
    data: Mutex<Data>,
}

impl Db {
    pub fn open(store: Box<dyn Store>) -> io::Result<Self> {
        let data = store.load()?;
        Ok(Self { store, data: Mutex::new(data) })
    }

    fn lock(&self) -> MutexGuard<'_, Data> {
        self.data.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn read<R>(&self, f: impl FnOnce(&Data) -> R) -> R {
        f(&self.lock())
    }

    /// Изменить данные и сразу сохранить.
    pub fn write<R>(&self, f: impl FnOnce(&mut Data) -> R) -> io::Result<R> {
        let mut d = self.lock();
        let r = f(&mut d);
        self.store.save(&d)?;
        Ok(r)
    }

    pub fn snapshot(&self) -> Snapshot {
        self.read(Data::snapshot)
    }

    /// Сохраняет настройки. Синк запускается, только если поменялось что-то кроме
    /// вкладки и выбранного дня: их переключают постоянно, на сервер они не нужны.
    pub fn save_settings(&self, mut settings: Settings) -> Result<Settings, String> {
        let t = now_ms();
        self.write(|d| {
            let ui_only = Settings { tab: d.settings.tab, day: d.settings.day, updated_ms: d.settings.updated_ms, ..settings.clone() };
            if ui_only != d.settings {
                settings.updated_ms = t;
                d.mark(Kind::Settings, "", Op::Up, t);
            } else {
                settings.updated_ms = d.settings.updated_ms;
            }
            d.settings = settings;
            d.settings.clone()
        })
        .map_err(|e| e.to_string())
    }

    /// Сохраняет день дневника целиком и возвращает все дни.
    pub fn save_day(&self, day: DayLog) -> Result<Vec<DayLog>, String> {
        weight::day_num(&day.date).ok_or("Неверная дата")?;
        let t = now_ms();
        self.write(|d| {
            d.mark(Kind::Day, &day.date, Op::Up, t);
            diary::upsert(&mut d.days, DayLog { updated_ms: t, ..day });
            d.days.clone()
        })
        .map_err(|e| e.to_string())
    }

    /// Заменить все данные (импорт из файла). Всё помечается к отправке на сервер.
    pub fn import(&self, snap: Snapshot) -> Result<Snapshot, String> {
        for e in &snap.entries {
            weight::check(e, None).map_err(|m| format!("{}: {m}", e.date))?;
        }
        let t = now_ms();
        self.write(|d| {
            let (tab, session, linked) = (d.settings.tab, d.session.take(), d.linked.take());
            *d = Data { settings: Settings { tab, updated_ms: t, ..snap.settings }, entries: snap.entries, days: snap.days, ..Data::default() };
            d.session = session;
            d.linked = linked;
            d.mark(Kind::Settings, "", Op::Up, t);
            let dates: Vec<(Kind, String)> = d
                .entries
                .iter()
                .map(|e| (Kind::Weight, e.date.clone()))
                .chain(d.days.iter().map(|x| (Kind::Day, x.date.clone())))
                .collect();
            for (k, date) in dates {
                d.mark(k, &date, Op::Up, t);
            }
            d.snapshot()
        })
        .map_err(|e| e.to_string())
    }

    /// Сохраняет запись (новую или поверх старой за ту же дату) и возвращает все записи.
    pub fn save_entry(&self, entry: Entry) -> Result<Vec<Entry>, String> {
        weight::check(&entry, None)?;
        let t = now_ms();
        self.write(|d| {
            d.mark(Kind::Weight, &entry.date, Op::Up, t);
            weight::upsert(&mut d.entries, Entry { updated_ms: t, ..entry });
            d.entries.clone()
        })
        .map_err(|e| e.to_string())
    }

    pub fn delete_entry(&self, date: &str) -> Result<Vec<Entry>, String> {
        let t = now_ms();
        self.write(|d| {
            if weight::remove(&mut d.entries, date) {
                d.mark(Kind::Weight, date, Op::Del, t);
            }
            d.entries.clone()
        })
        .map_err(|e| e.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// На wasm32-unknown-unknown SystemTime::now() паникует, время берём у браузера.
#[cfg(target_arch = "wasm32")]
pub fn now_ms() -> i64 {
    js_sys::Date::now() as i64
}
