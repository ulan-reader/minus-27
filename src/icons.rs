//! Иконки: контурные SVG 24×24, цвет берут из текста (currentColor).
//! Свои, без внешних шрифтов: работают офлайн и в приложении, и на сайте.

use leptos::prelude::*;

fn path(name: &str) -> &'static str {
    match name {
        "home" => "M3 11l9-8 9 8M5 10v10h14V10M10 20v-6h4v6",
        "utensils" => "M7 3v18M4 3v5a3 3 0 0 0 6 0V3M17 21V3c-2 2-3 5-3 8h3",
        "cart" => "M3 4h2l2.4 11h11l2-8H6.2M9 20.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2zM18 20.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2z",
        "chart" => "M4 19h16M5 15l4-5 4 3 6-7",
        "user" => "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM5 21a7 7 0 0 1 14 0",
        "plus" => "M12 5v14M5 12h14",
        "minus" => "M5 12h14",
        "camera" => "M4 8h3l2-3h6l2 3h3v11H4zM12 17a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
        "droplet" => "M12 3l5.5 7.5a6.5 6.5 0 1 1-11 0z",
        "scale" => "M5 4h14a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1zM8 10a4 4 0 0 1 8 0M12 10l2-2",
        "check" => "M5 12l5 5L20 7",
        "check-circle" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM8 12l3 3 5-6",
        "circle" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z",
        "x" => "M6 6l12 12M18 6L6 18",
        "trash" => "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3",
        "edit" => "M4 20h4L19 9l-4-4L4 16zM13.5 6.5l4 4",
        "swap" => "M4 8h13l-3-3M20 16H7l3 3",
        "chevron-left" => "M15 6l-6 6 6 6",
        "chevron-right" => "M9 6l6 6-6 6",
        "refresh" => "M20 11a8 8 0 1 0-2.3 5.7M20 4v7h-7",
        "bell" => "M6 17v-6a6 6 0 0 1 12 0v6l2 2H4zM10 21h4",
        "ruler" => "M3 17L17 3l4 4L7 21zM7 13l2 2M10 10l2 2M13 7l2 2",
        "sunrise" => "M3 18h18M7 18a5 5 0 0 1 10 0M12 4v4M5 9l1.5 1.5M19 9l-1.5 1.5",
        "sun" => "M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM12 2v2M12 20v2M4 12H2M22 12h-2M5 5l1.5 1.5M17.5 17.5L19 19M5 19l1.5-1.5M17.5 6.5L19 5",
        "moon" => "M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z",
        "apple" => "M12 7c-2-2-7-1.5-7 4 0 5 3 9 5 9 1 0 1.5-.5 2-.5s1 .5 2 .5c2 0 5-4 5-9 0-5.5-5-6-7-4zM12 7c0-2 1-4 3-4",
        "meat" => "M14 4a6 6 0 0 1 4 10l-6 6a3 3 0 0 1-4-4l-2-2a3 3 0 0 1 4-4z",
        "milk" => "M8 3h8v3l2 3v12H6V9l2-3zM6 12h12",
        "grain" => "M12 21V9M12 9c-3 0-4-3-4-5 3 0 4 3 4 5zM12 9c3 0 4-3 4-5-3 0-4 3-4 5zM12 15c-3 0-4-3-4-5 3 0 4 3 4 5zM12 15c3 0 4-3 4-5-3 0-4 3-4 5z",
        "leaf" => "M5 19c0-9 6-14 15-14 0 9-5 15-14 15zM5 19l8-8",
        "nut" => "M12 3c4 0 7 3 7 7s-3 11-7 11-7-7-7-11 3-7 7-7zM8 9h8",
        "copy" => "M9 9h11v11H9zM5 15H4V4h11v1",
        "download" => "M12 4v12M7 11l5 5 5-5M5 20h14",
        "upload" => "M12 20V8M7 13l5-5 5 5M5 4h14",
        "logout" => "M15 4h4v16h-4M10 8l-4 4 4 4M6 12h10",
        "sparkles" => "M12 3l1.8 4.2L18 9l-4.2 1.8L12 15l-1.8-4.2L6 9l4.2-1.8zM18 15l.9 2.1L21 18l-2.1.9L18 21l-.9-2.1L15 18l2.1-.9z",
        "flame" => "M12 21a6 6 0 0 0 6-6c0-4-3-6-4-10-2 2-3 4-3 6-1-1-2-2-2-3-2 2-3 4-3 7a6 6 0 0 0 6 6z",
        "target" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM12 12h.01",
        "info" => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM12 11v5M12 8h.01",
        "external" => "M14 4h6v6M20 4l-9 9M18 14v6H4V6h6",
        "search" => "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14zM20 20l-4-4",
        "cloud" => "M7 18a5 5 0 1 1 1-9.9A6 6 0 0 1 19 10a4 4 0 0 1 0 8z",
        _ => "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z",
    }
}

/// Иконка по имени. Размер задаёт CSS (.ic, по умолчанию 1.25em).
pub fn icon(name: &str) -> impl IntoView + use<> {
    let svg = format!(
        r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="{}"/></svg>"#,
        path(name)
    );
    view! { <span class="ic" inner_html=svg></span> }
}
