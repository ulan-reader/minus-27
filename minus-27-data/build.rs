use std::{env, fs};

/// Ключи Supabase берутся из supabase/supabase.env (не в git) или из переменных окружения
/// и вшиваются в код через env! (см. src/config.rs).
const KEYS: [&str; 2] = ["SUPABASE_URL", "SUPABASE_ANON_KEY"];
const FILE: &str = "../supabase/supabase.env";

fn main() {
    println!("cargo:rerun-if-changed={FILE}");
    let file = fs::read_to_string(FILE).unwrap_or_default();
    for key in KEYS {
        println!("cargo:rerun-if-env-changed={key}");
        let from_file = file.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
        });
        let value = env::var(key).ok().filter(|v| !v.is_empty()).or(from_file).unwrap_or_default();
        println!("cargo:rustc-env={key}={value}");
    }
}
