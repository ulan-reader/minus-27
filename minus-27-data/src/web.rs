//! Хранение для сайта: весь `Data` одной JSON-строкой в localStorage.

use std::io;

use wasm_bindgen::prelude::*;

use crate::storage::{Data, Store};

const KEY: &str = "minus27.data";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = localStorage, js_name = getItem, catch)]
    fn get_item(key: &str) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(js_namespace = localStorage, js_name = setItem, catch)]
    fn set_item(key: &str, value: &str) -> Result<(), JsValue>;
}

pub struct LocalStore;

impl Store for LocalStore {
    fn load(&self) -> io::Result<Data> {
        // Нет данных, приватный режим или битый JSON: начинаем с пустого.
        let text = get_item(KEY).ok().flatten().unwrap_or_default();
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    fn save(&self, data: &Data) -> io::Result<()> {
        let text = serde_json::to_string(data)?;
        set_item(KEY, &text).map_err(|_| io::Error::other("Не удалось сохранить в браузере (нет места?)"))
    }
}
