//! Подключение к Supabase. Значения подставляет build.rs из supabase/supabase.env
//! (файл не в git, образец: supabase.env.example) или из переменных окружения.
//! Пусто = синк выключен, данные только на устройстве.
//! Ключ публичный (anon / publishable), чужие данные защищает RLS из supabase/schema.sql.
//! Секретный ключ (sb_secret_… / service_role) сюда не вставлять никогда.

pub const SUPABASE_URL: &str = env!("SUPABASE_URL");
pub const SUPABASE_ANON_KEY: &str = env!("SUPABASE_ANON_KEY");
