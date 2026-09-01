const version = new URL(self.location.href).searchParams.get('v') || 'dev';
const CACHE = `stock-promise-shell-${version}`;
const SHELL = ['/', '/demo', '/mark.svg', '/manifest.webmanifest', '/icon-192.png', '/assets/stockroom-watch-640.webp'];
self.addEventListener('install', (event) => event.waitUntil((async () => {
  const cache = await caches.open(CACHE);
  await cache.addAll(SHELL);
  const demo = await cache.match('/demo');
  const html = await demo.text();
  const builtAssets = [...html.matchAll(/(?:src|href)="(\/assets\/index-[^"]+\.(?:js|css))"/g)].map((match) => match[1]);
  await cache.addAll(builtAssets);
  await self.skipWaiting();
})()));
self.addEventListener('activate', (event) => event.waitUntil(caches.keys().then((keys) => Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key)))).then(() => self.clients.claim())));
self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET' || new URL(event.request.url).pathname.startsWith('/api/')) return;
  event.respondWith(fetch(event.request).then((response) => {
    if (response.ok && new URL(event.request.url).origin === location.origin) caches.open(CACHE).then((cache) => cache.put(event.request, response.clone()));
    return response;
  }).catch(async () => {
    const cached = await caches.match(event.request);
    if (cached) return cached;
    if (event.request.mode === 'navigate') return caches.match('/');
    return Response.error();
  }));
});
