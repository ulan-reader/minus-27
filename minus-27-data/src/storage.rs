//! Хранение данных: в приложении JSON-файл (`JsonFile`), на сайте localStorage (`web::LocalStore`).
//! Чтобы перейти, например, на SQLite, достаточно новой реализации `Store`.

use std::{
    io,
    sync::{Mutex, MutexGuard},
};

use minus27_core::{
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
    pub date: String,
    pub op: Op,
    pub t: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Up,
    Del,
}

impl Data {
    pub fn mark(&mut self, date: &str, op: Op, t: i64) {
        self.pending.retain(|p| p.date != date);
        self.pending.push(Pending { date: date.to_string(), op, t });
    }
}

/// То, что UI получает при старте.
#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub settings: Settings,
    pub entries: Vec<Entry>,
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
        self.read(|d| Snapshot { settings: d.settings.clone(), entries: d.entries.clone() })
    }

    pub fn save_settings(&self, settings: Settings) -> Result<(), String> {
        self.write(|d| d.settings = settings).map_err(|e| e.to_string())
    }

    /// Сохраняет запись (новую или поверх старой за ту же дату) и возвращает все записи.
    pub fn save_entry(&self, entry: Entry) -> Result<Vec<Entry>, String> {
        weight::check(&entry, None)?;
        let t = now_ms();
        self.write(|d| {
            d.mark(&entry.date, Op::Up, t);
            weight::upsert(&mut d.entries, Entry { updated_ms: t, ..entry });
            d.entries.clone()
        })
        .map_err(|e| e.to_string())
    }

    pub fn delete_entry(&self, date: &str) -> Result<Vec<Entry>, String> {
        let t = now_ms();
        self.write(|d| {
            if weight::remove(&mut d.entries, date) {
                d.mark(date, Op::Del, t);
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
