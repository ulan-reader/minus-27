use std::{env, fs};

/// Ключи Supabase берутся из supabase.env (не в git) или из переменных окружения
/// и вшиваются в бинарник через env! (см. src/config.rs).
const KEYS: [&str; 2] = ["SUPABASE_URL", "SUPABASE_ANON_KEY"];

fn main() {
    println!("cargo:rerun-if-changed=supabase.env");
    let file = fs::read_to_string("supabase.env").unwrap_or_default();
    for key in KEYS {
        println!("cargo:rerun-if-env-changed={key}");
        let from_file = file.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
        });
        let value = env::var(key).ok().or(from_file).unwrap_or_default();
        println!("cargo:rustc-env={key}={value}");
    }
    tauri_build::build()
}
