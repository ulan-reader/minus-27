//! Немного браузерного API, без лишних фич web-sys.

use minus27_core::weight;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["navigator", "clipboard"], js_name = writeText, catch)]
    async fn write_text(text: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = scrollTo)]
    fn scroll_to(x: f64, y: f64);
}

pub async fn copy(text: &str) -> bool {
    write_text(text).await.is_ok()
}

pub fn scroll_top() {
    scroll_to(0.0, 0.0);
}

/// Сегодня по местному времени, "2026-11-23".
pub fn today_iso() -> String {
    let d = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
}

/// Текущее время "14:05".
pub fn time_hm() -> String {
    let d = js_sys::Date::new_0();
    format!("{:02}:{:02}", d.get_hours(), d.get_minutes())
}

pub fn today_num() -> i32 {
    weight::day_num(&today_iso()).unwrap_or(i32::MAX)
}
