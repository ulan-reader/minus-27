-- Схема базы «Минус 27».
-- Выполнить: Supabase → SQL Editor → вставить весь файл → Run.
-- Можно выполнять повторно: всё создаётся через "if not exists", данные не теряются.
-- Каждый пользователь видит и меняет только свои строки (RLS).

-- ---------- Вес и замеры, одна строка на день ----------
create table if not exists public.weight_entries (
  user_id    uuid        not null default auth.uid() references auth.users on delete cascade,
  entry_date date        not null,
  weight_kg  numeric(5,1) not null check (weight_kg between 30 and 300),
  waist_cm   numeric(5,1) check (waist_cm between 40 and 250),
  steps      integer     check (steps between 0 and 100000),
  updated_at timestamptz not null default now(),
  primary key (user_id, entry_date)
);
alter table public.weight_entries add column if not exists hips_cm  numeric(5,1) check (hips_cm between 40 and 250);
alter table public.weight_entries add column if not exists chest_cm numeric(5,1) check (chest_cm between 40 and 250);

-- ---------- Дневник питания и вода, один день целиком ----------
create table if not exists public.day_logs (
  user_id    uuid        not null default auth.uid() references auth.users on delete cascade,
  log_date   date        not null,
  data       jsonb       not null default '{}'::jsonb,
  updated_at timestamptz not null default now(),
  primary key (user_id, log_date)
);

-- ---------- Профиль и настройки, одна строка на человека ----------
create table if not exists public.profiles (
  user_id    uuid        primary key default auth.uid() references auth.users on delete cascade,
  data       jsonb       not null default '{}'::jsonb,
  updated_at timestamptz not null default now()
);

-- ---------- RLS: только свои строки ----------
do $$
declare t text;
begin
  foreach t in array array['weight_entries', 'day_logs', 'profiles'] loop
    execute format('alter table public.%I enable row level security', t);
    execute format('drop policy if exists "own: select" on public.%I', t);
    execute format('drop policy if exists "own: insert" on public.%I', t);
    execute format('drop policy if exists "own: update" on public.%I', t);
    execute format('drop policy if exists "own: delete" on public.%I', t);
    execute format('create policy "own: select" on public.%I for select to authenticated using ((select auth.uid()) = user_id)', t);
    execute format('create policy "own: insert" on public.%I for insert to authenticated with check ((select auth.uid()) = user_id)', t);
    execute format('create policy "own: update" on public.%I for update to authenticated using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id)', t);
    execute format('create policy "own: delete" on public.%I for delete to authenticated using ((select auth.uid()) = user_id)', t);
  end loop;
end $$;

-- Политики из первой версии схемы
drop policy if exists "weight: select own" on public.weight_entries;
drop policy if exists "weight: insert own" on public.weight_entries;
drop policy if exists "weight: update own" on public.weight_entries;
drop policy if exists "weight: delete own" on public.weight_entries;

-- ---------- Лимит распознаваний фото ----------
-- Счётчик на человека в день. Напрямую из приложения недоступен (RLS без политик),
-- меняется только функцией ai_take, которую вызывает Edge Function estimate-food.
create table if not exists public.ai_usage (
  user_id  uuid not null references auth.users on delete cascade,
  day      date not null default current_date,
  count    int  not null default 0,
  primary key (user_id, day)
);
alter table public.ai_usage enable row level security;

-- Взять одно распознавание. Возвращает, сколько осталось после этого, или -1, если лимит исчерпан.
create or replace function public.ai_take(p_limit int)
returns int
language plpgsql
security definer
set search_path = ''
as $$
declare used int;
begin
  if auth.uid() is null then
    raise exception 'not authenticated';
  end if;
  insert into public.ai_usage (user_id, day, count)
  values (auth.uid(), current_date, 1)
  on conflict (user_id, day) do update set count = public.ai_usage.count + 1
  returning count into used;
  if used > p_limit then
    return -1;
  end if;
  return p_limit - used;
end $$;

revoke all on function public.ai_take(int) from public, anon;
grant execute on function public.ai_take(int) to authenticated;
