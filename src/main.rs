mod api;
mod app;
mod chart;
mod components;
mod fmt;
mod icons;
mod js;
mod screens;

use app::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
