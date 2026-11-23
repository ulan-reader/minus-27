//! Данные «Минус 27»: хранение записей и синк с Supabase.
//! В приложении данные лежат в JSON-файле (src-tauri), на сайте в localStorage (`web`).

mod config;
pub mod storage;
pub mod sync;
pub mod web;

pub use storage::{Db, Snapshot};
pub use sync::Remote;
