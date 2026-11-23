//! Хранение данных на устройстве.
//! Сейчас это один JSON-файл. Чтобы перейти на SQLite, достаточно новой реализации `Store`.

use std::{
    fs, io,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
    time::{SystemTime, UNIX_EPOCH},
};

use minus27_core::{state::Settings, weight::Entry};
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

pub trait Store: Send + Sync {
    fn load(&self) -> io::Result<Data>;
    fn save(&self, data: &Data) -> io::Result<()>;
}

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

/// Данные в памяти плюс место, куда их сохранять. Лежит в состоянии Tauri.
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
}

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}
