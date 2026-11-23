// Service worker сайта: чтобы приложение открывалось без сети.
// Сначала сеть (всегда свежая версия), при ошибке — последняя сохранённая копия.
// Запросы к Supabase (другой домен) не трогаем.
const CACHE = 'minus27-v1';

self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', e => e.waitUntil(self.clients.claim()));

self.addEventListener('fetch', e => {
  const req = e.request;
  if (req.method !== 'GET' || new URL(req.url).origin !== self.location.origin) return;
  e.respondWith(
    fetch(req)
      .then(res => {
        if (res.ok) {
          const copy = res.clone();
          caches.open(CACHE).then(c => c.put(req, copy));
        }
        return res;
      })
      .catch(async () => (await caches.match(req)) || (req.mode === 'navigate' ? caches.match('./') : Response.error()))
  );
});
