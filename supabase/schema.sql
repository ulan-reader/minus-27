-- Таблица записей веса для «Минус 27».
-- Выполнить один раз: Supabase → SQL Editor → вставить → Run.
-- Каждый пользователь видит и меняет только свои строки (RLS).

create table if not exists public.weight_entries (
  user_id    uuid        not null default auth.uid() references auth.users on delete cascade,
  entry_date date        not null,
  weight_kg  numeric(5,1) not null check (weight_kg between 30 and 300),
  waist_cm   numeric(5,1) check (waist_cm between 40 and 250),
  steps      integer     check (steps between 0 and 100000),
  updated_at timestamptz not null default now(),
  primary key (user_id, entry_date)
);

alter table public.weight_entries enable row level security;

drop policy if exists "weight: select own" on public.weight_entries;
drop policy if exists "weight: insert own" on public.weight_entries;
drop policy if exists "weight: update own" on public.weight_entries;
drop policy if exists "weight: delete own" on public.weight_entries;

create policy "weight: select own" on public.weight_entries
  for select to authenticated using ((select auth.uid()) = user_id);
create policy "weight: insert own" on public.weight_entries
  for insert to authenticated with check ((select auth.uid()) = user_id);
create policy "weight: update own" on public.weight_entries
  for update to authenticated using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id);
create policy "weight: delete own" on public.weight_entries
  for delete to authenticated using ((select auth.uid()) = user_id);
