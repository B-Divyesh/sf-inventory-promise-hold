import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('sets up a location and completes the hold lifecycle on a phone', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });

  await page.goto('/');
  await expect(page).toHaveTitle('Stock Promise — timed inventory holds');
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.locator('h1')).toHaveCount(1);
  const landingAccessibility = await new AxeBuilder({ page }).analyze();
  expect(landingAccessibility.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);

  await expect(page.getByRole('heading', { name: 'Hold scarce stock before it is promised twice.' })).toBeVisible();
  await page.getByRole('button', { name: 'Open the live desk' }).click();
  await page.getByLabel('Location name').fill('Test counter');
  await page.getByLabel(/Supervisor PIN/).fill('246810');
  await page.getByRole('button', { name: 'Open the promise desk' }).click();
  await expect(page.getByRole('heading', { name: 'No stock is listed yet' })).toBeVisible();

  const anonymousBootstrap = await page.request.get('/api/bootstrap');
  expect(anonymousBootstrap.status()).toBe(401);
  const anonymousHold = await page.request.post('/api/holds', {
    data: { inventory_id: 1, quantity: 1, customer: 'Intruder', operator_name: 'Unknown', duration_minutes: 30 },
  });
  expect(anonymousHold.status()).toBe(401);

  await page.getByRole('button', { name: 'Add first item' }).click();
  await page.getByLabel('SKU', { exact: true }).fill('FILTER-7');
  await page.getByLabel('Item name').fill('Water filter');
  await page.getByLabel('On-hand quantity').fill('5');
  await page.getByRole('button', { name: 'Add to stock list' }).click();
  await expect(page.getByText('5 available')).toBeVisible();

  await page.getByRole('button', { name: 'Create hold' }).click();
  await page.getByLabel('Quantity').fill('2');
  await page.getByLabel('Customer or order reference').fill('Northside Cafe');
  await page.getByLabel('Your name').fill('Mina');
  await page.getByRole('button', { name: 'Hold FILTER-7' }).click();
  await expect(page.getByText('3 available')).toBeVisible();
  await expect(page.getByText(/^For Northside Cafe/)).toBeVisible();

  page.once('dialog', (dialog) => dialog.accept());
  await page.getByRole('button', { name: 'Convert', exact: true }).click();
  await expect(page.getByText('No stock is tied up.')).toBeVisible();
  await page.getByRole('button', { name: 'Outcomes' }).click();
  await expect(page.getByText('converted', { exact: true })).toBeVisible();
  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export CSV' }).click();
  expect((await download).suggestedFilename()).toBe('stock-promise-holds.csv');
  await page.getByRole('button', { name: 'Stock & settings' }).click();
  await expect(page.getByText('hold converted', { exact: true })).toBeVisible();
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  const legalTargets = await page.locator('.site-footer nav a').evaluateAll((links) =>
    links.map((link) => ({ width: link.getBoundingClientRect().width, height: link.getBoundingClientRect().height })),
  );
  expect(legalTargets.every(({ width, height }) => width >= 44 && height >= 44)).toBe(true);

  await page.getByRole('button', { name: 'Lock supervisor' }).click();
  await expect(page.getByRole('heading', { name: 'Open the promise desk.' })).toBeVisible();
  await expect(page.getByText('Northside Cafe')).toHaveCount(0);
  await expect(consoleErrors).toEqual([]);
});

test('legal pages stay semantic and reachable', async ({ page }) => {
  for (const [route, title, heading] of [
    ['/privacy', 'Privacy — Stock Promise', 'Privacy for Stock Promise'],
    ['/terms', 'Terms — Stock Promise', 'Terms for temporary stock holds'],
  ] as const) {
    await page.goto(route);
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page).toHaveTitle(title);
    await expect(page.getByRole('heading', { name: heading })).toBeVisible();
    const accessibility = await new AxeBuilder({ page }).analyze();
    expect(accessibility.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  }
  await page.getByRole('button', { name: 'Return home' }).click();
  await expect(page).toHaveURL('/');
});

test('public routes keep one route-specific canonical and description', async ({ page }) => {
  const routes = [
    ['/demo', 'Demo — Stock Promise', 'https://inventory-promise-hold.sociobot.in/demo'],
    ['/privacy', 'Privacy — Stock Promise', 'https://inventory-promise-hold.sociobot.in/privacy'],
    ['/terms', 'Terms — Stock Promise', 'https://inventory-promise-hold.sociobot.in/terms'],
  ] as const;
  for (const [route, title, canonical] of routes) {
    await page.goto(route, { waitUntil: 'networkidle' });
    await expect(page).toHaveTitle(title);
    await expect(page.locator('link[rel="canonical"]')).toHaveCount(1);
    await expect(page.locator('meta[name="description"]')).toHaveCount(1);
    await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', canonical);
    await expect(page.locator('meta[name="description"]')).not.toHaveAttribute('content', 'Create timed, shared inventory holds so scarce stock is not promised twice.');
  }
  await page.goto('/does-not-exist');
  await expect(page).toHaveTitle('Page not found — Stock Promise');
  await expect(page.locator('link[rel="canonical"]')).toHaveCount(1);
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://inventory-promise-hold.sociobot.in/404');
  await expect(page.locator('meta[name="description"]')).toHaveAttribute('content', 'Return to Stock Promise or try the sample stockroom.');
});

test('direct routes, cache policy, metadata, and security headers are production-safe', async ({ request }) => {
  for (const route of ['/privacy', '/terms', '/demo']) {
    const response = await request.head(route);
    expect(response.status()).toBe(200);
    expect(response.headers()['cache-control']).toContain('no-cache');
  }
  const home = await request.get('/');
  expect((await home.text())).toContain('og:image');
  const assetPath = (await home.text()).match(/\/assets\/index-[^"']+\.js/)?.[0];
  expect(assetPath).toBeTruthy();
  const asset = await request.get(assetPath!);
  expect(asset.headers()['cache-control']).toBe('public, max-age=31536000, immutable');
  const worker = await request.get('/sw.js');
  expect(worker.headers()['cache-control']).toBe('no-cache, no-store, must-revalidate');
  const health = await request.get('/health');
  expect(health.headers()['cache-control']).toBe('no-store');
  expect(health.headers()['strict-transport-security']).toContain('max-age=31536000');
  expect(health.headers()['permissions-policy']).toContain('camera=()');
  expect((await request.get('/robots.txt')).status()).toBe(200);
  expect((await request.get('/sitemap.xml')).status()).toBe(200);
  expect((await request.get('/does-not-exist')).status()).toBe(404);
});

test('desktop keyboard access gate recovers from an invalid PIN', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  await page.goto('/');
  await page.keyboard.press('Tab');
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/#main$/);
  await page.getByRole('button', { name: 'Open the live desk' }).click();
  await expect(page.getByRole('heading', { name: 'Open the promise desk.' })).toBeVisible();

  await page.getByLabel('Supervisor PIN').fill('000000');
  await page.getByRole('button', { name: 'Open promise desk' }).click();
  await expect(page.getByRole('alert')).toContainText('not correct');
  await page.getByLabel('Supervisor PIN').fill('246810');
  await page.getByRole('button', { name: 'Open promise desk' }).click();
  await expect(page.getByRole('heading', { name: 'Promise desk' })).toBeVisible();
  const deskAccessibility = await new AxeBuilder({ page }).analyze();
  expect(deskAccessibility.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  await context.close();
});

test('Escape closes a hold dialog and returns focus to its opener', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  await page.goto('/demo', { waitUntil: 'networkidle' });
  const opener = page.getByRole('button', { name: 'Create hold' }).first();
  await opener.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('dialog')).toBeVisible();
  expect(await page.evaluate(() => document.querySelector('dialog')?.contains(document.activeElement))).toBe(true);
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect(opener).toBeFocused();
  await context.close();
});

test('browser Back and Forward focus and announce each route heading', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  await page.goto('/');
  await page.locator('.top-nav').getByRole('link', { name: 'Privacy' }).click();
  await expect(page.getByRole('heading', { name: 'Privacy for Stock Promise' })).toBeFocused();
  await expect(page.locator('.live-region')).toHaveText('Opened Privacy for Stock Promise.');

  await page.goBack();
  await expect(page).toHaveURL('/');
  await expect(page.getByRole('heading', { name: 'Hold scarce stock before it is promised twice.' })).toBeFocused();
  await expect(page.locator('.live-region')).toHaveText('Opened Hold scarce stock before it is promised twice.');

  await page.goForward();
  await expect(page).toHaveURL('/privacy');
  await expect(page.getByRole('heading', { name: 'Privacy for Stock Promise' })).toBeFocused();
  await expect(page.locator('.live-region')).toHaveText('Opened Privacy for Stock Promise.');
  await context.close();
});

test('390px reduced-motion shell updates and explains an offline reload', async ({ browser }) => {
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    reducedMotion: 'reduce',
    serviceWorkers: 'allow',
  });
  const page = await context.newPage();
  await page.goto('/demo', { waitUntil: 'networkidle' });
  expect(await page.evaluate(() => matchMedia('(prefers-reduced-motion: reduce)').matches)).toBe(true);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload({ waitUntil: 'networkidle' });
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
  const script = await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    await registration.update();
    return registration.active?.scriptURL;
  });
  expect(script).toMatch(/^http:\/\/127\.0\.0\.1:4178\/sw\.js\?v=/);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByText(/Demo.*sample data/i)).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Promise desk' })).toBeVisible();
  await context.close();
});

test('390px targets are at least 44px and 200% text does not widen the demo', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, bypassCSP: true });
  const page = await context.newPage();
  await page.goto('/demo', { waitUntil: 'networkidle' });
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  const undersized = await page.locator('a, button, input, select, textarea').evaluateAll((elements) => elements
    .filter((element) => {
      const style = getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return style.visibility !== 'hidden' && style.display !== 'none' && rect.width > 0 && rect.height > 0;
    })
    .map((element) => {
      const rect = element.getBoundingClientRect();
      return { label: (element.textContent || element.getAttribute('aria-label') || element.tagName).trim(), width: rect.width, height: rect.height };
    })
    .filter((target) => target.width < 44 || target.height < 44),
  );
  expect(undersized).toEqual([]);

  await page.evaluate(() => { document.documentElement.style.fontSize = '32px'; });
  const reflow = await page.evaluate(() => {
    const width = document.documentElement.clientWidth;
    return {
      viewport: width,
      document: document.documentElement.scrollWidth,
      offenders: [...document.querySelectorAll<HTMLElement>('*')]
        .map((element) => ({ tag: element.tagName, className: element.className, right: element.getBoundingClientRect().right }))
        .filter((element) => element.right > width + 1)
        .slice(0, 8),
    };
  });
  expect(reflow.document, JSON.stringify(reflow.offenders)).toBeLessThanOrEqual(reflow.viewport);
  await context.close();
});

test('@claim:demo-isolated demo storage and requests never cross into the real workspace', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  await context.addInitScript(() => {
    localStorage.setItem('stock-promise:operator', 'Real workspace operator');
    localStorage.setItem('stock-promise:supervisor-name', 'Real supervisor');
    localStorage.setItem('stock-promise:profiles', JSON.stringify(['Real profile']));
    localStorage.setItem('stock-promise:reminders', 'true');
    localStorage.setItem('sb_license:inventory-promise-hold', 'real-cached-license');
    localStorage.setItem('sb_license:inventory-promise-hold:verdict', JSON.stringify({ valid: true, checked: 1 }));
    sessionStorage.setItem('stock-promise:supervisor-session', 'real-session-token');
  });
  const page = await context.newPage();
  const productApiRequests: string[] = [];
  const licenseRequests: string[] = [];
  page.on('request', (request) => {
    if (request.url().startsWith('http://127.0.0.1:4178/api/')) productApiRequests.push(`${request.method()} ${request.url()}`);
  });
  await page.route('https://api.sociobot.in/api/v1/products/inventory-promise-hold/verify?license=*', async (route) => {
    licenseRequests.push(route.request().url());
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok' }) });
  });
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await expect(page.getByText('Demo — sample data, nothing is saved.')).toBeVisible();
  await page.getByRole('button', { name: 'Stock & settings' }).click();
  await expect(page.getByRole('heading', { name: 'Pro reminders & profiles' })).toBeVisible();
  await page.getByRole('button', { name: 'Import CSV' }).click();
  await page.getByLabel('Choose CSV file').setInputFiles({
    name: 'demo-stock.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from('sku,name,on_hand\nDEMO-ONLY,Demo-only item,2\n'),
  });
  await expect(page.getByText('1 item imported.')).toBeVisible();
  await page.getByRole('button', { name: 'Done' }).click();
  await page.getByRole('button', { name: 'Live desk' }).click();
  await page.getByRole('button', { name: 'Create hold' }).first().click();
  await expect(page.getByLabel('Your name')).toHaveValue('');
  await page.getByLabel('Quantity').fill('1');
  await page.getByLabel('Customer or order reference').fill('Demo counter order 102');
  await page.getByLabel('Your name').fill('Demo staff');
  await page.getByRole('button', { name: 'Hold VALVE-24' }).click();
  await expect(page.getByText(/Sample hold created for Demo counter order 102/)).toBeVisible();

  const beforeReset = await page.evaluate(() => ({
    local: Object.fromEntries(Object.keys(localStorage).sort().map((key) => [key, localStorage.getItem(key)])),
    session: Object.fromEntries(Object.keys(sessionStorage).sort().map((key) => [key, sessionStorage.getItem(key)])),
  }));
  expect(beforeReset.local).toEqual({
    'sb_license:inventory-promise-hold': 'real-cached-license',
    'sb_license:inventory-promise-hold:verdict': JSON.stringify({ valid: true, checked: 1 }),
    'stock-promise:operator': 'Real workspace operator',
    'stock-promise:profiles': JSON.stringify(['Real profile']),
    'stock-promise:reminders': 'true',
    'stock-promise:supervisor-name': 'Real supervisor',
  });
  expect(beforeReset.session['stock-promise:supervisor-session']).toBe('real-session-token');
  expect(beforeReset.session['demo:stock-promise:operator']).toBe('Demo staff');
  expect(beforeReset.session['demo:stock-promise:state']).toContain('Demo counter order 102');

  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.locator('.live-region')).toHaveText('Demo reset to the shipped sample data.');
  const afterReset = await page.evaluate(() => ({
    local: Object.fromEntries(Object.keys(localStorage).sort().map((key) => [key, localStorage.getItem(key)])),
    session: Object.fromEntries(Object.keys(sessionStorage).sort().map((key) => [key, sessionStorage.getItem(key)])),
  }));
  expect(afterReset.local).toEqual(beforeReset.local);
  expect(afterReset.session).toEqual({ 'stock-promise:supervisor-session': 'real-session-token' });
  expect(productApiRequests).toEqual([]);
  expect(licenseRequests).toEqual([]);
  await context.close();
});

test('@claim:demo-seed-reset demo starts with three SKUs and one hold, then Reset restores them', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await expect(page.locator('.inventory-list > li')).toHaveCount(3);
  await expect(page.locator('.hold-list > li')).toHaveCount(1);
  await expect(page.getByText('Northline Plumbing order 418')).toBeVisible();

  await page.getByRole('button', { name: 'Create hold' }).first().click();
  await page.getByLabel('Quantity').fill('1');
  await page.getByLabel('Customer or order reference').fill('Temporary reset check');
  await page.getByLabel('Your name').fill('Demo operator');
  await page.getByRole('button', { name: 'Hold VALVE-24' }).click();
  await expect(page.locator('.hold-list > li')).toHaveCount(2);
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.locator('.inventory-list > li')).toHaveCount(3);
  await expect(page.locator('.hold-list > li')).toHaveCount(1);
  await expect(page.getByText('Temporary reset check')).toHaveCount(0);
  expect(await page.evaluate(() => Object.keys(sessionStorage).filter((key) => key.startsWith('demo:stock-promise:')))).toEqual([]);
  await context.close();
});

test('@claim:no-tracking normal home, privacy, and demo use no tracking requests or cookies', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await page.goto('/', { waitUntil: 'networkidle' });
  await page.locator('.top-nav').getByRole('link', { name: 'Privacy' }).click();
  await page.getByRole('link', { name: 'Demo' }).click();
  await page.getByRole('button', { name: 'Create hold' }).first().click();
  await page.getByLabel('Quantity').fill('1');
  await page.getByLabel('Customer or order reference').fill('Private demo order');
  await page.getByLabel('Your name').fill('Demo operator');
  await page.getByRole('button', { name: 'Hold VALVE-24' }).click();
  expect(requests.every((url) => new URL(url).origin === 'http://127.0.0.1:4178')).toBe(true);
  expect(await context.cookies()).toEqual([]);
  expect(await page.evaluate(() => localStorage.length)).toBe(0);
  expect(await page.evaluate(() => sessionStorage.getItem('demo:stock-promise:operator'))).toBe('Demo operator');
  await context.close();
});

test('@claim:browser-storage live access and preferences use their documented browser stores', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  await context.addInitScript(() => {
    class TestNotification {
      static permission = 'granted';
      static requestPermission = async () => 'granted';
      constructor() { /* The storage assertion does not need a delivered reminder. */ }
    }
    Object.defineProperty(window, 'Notification', { configurable: true, value: TestNotification });
  });
  const page = await context.newPage();
  await page.route('https://api.sociobot.in/api/v1/products/inventory-promise-hold/verify?license=browser-storage-license', async (route) => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok' }) });
  });
  await page.goto('/');
  await page.getByRole('button', { name: 'Open the live desk' }).click();
  await expect(page.getByLabel(/Supervisor PIN/)).toBeVisible();
  const needsSetup = Boolean(await page.getByLabel('Location name').count());

  if (needsSetup) {
    await page.getByLabel('Location name').fill('Browser storage test');
    await page.getByLabel(/Supervisor PIN/).fill('246810');
    await page.getByRole('button', { name: 'Open the promise desk' }).click();
  } else {
    await page.getByLabel('Supervisor PIN').fill('246810');
    await page.getByRole('button', { name: 'Open promise desk' }).click();
  }
  await expect(page.getByRole('heading', { name: 'Promise desk' })).toBeVisible();

  if (needsSetup) {
    const addFirst = page.getByRole('button', { name: 'Add first item' });
    await expect(addFirst).toBeVisible();
    await addFirst.click();
    await page.getByLabel('SKU', { exact: true }).fill('STORE-1');
    await page.getByLabel('Item name').fill('Storage test item');
    await page.getByLabel('On-hand quantity').fill('3');
    await page.getByRole('button', { name: 'Add to stock list' }).click();
  }
  const createHold = page.getByRole('button', { name: 'Create hold' }).first();
  await expect(createHold).toBeVisible();
  await createHold.click();
  await page.getByLabel('Quantity').fill('1');
  await page.getByLabel('Customer or order reference').fill('Browser storage order');
  await page.getByLabel('Your name').fill('Browser operator');
  await page.getByRole('button', { name: /^Hold / }).click();
  await expect(page.getByText(/Hold created for Browser storage order/)).toBeVisible();

  await page.getByRole('button', { name: 'Stock & settings' }).click();
  await page.getByLabel('Have a license? Paste it here').fill('browser-storage-license');
  await page.getByRole('button', { name: 'Verify license' }).click();
  await expect(page.getByRole('heading', { name: 'Stock Promise Pro is active' })).toBeVisible();
  await page.getByRole('button', { name: 'Save profile' }).click();
  await page.getByRole('button', { name: 'Enable 5-minute reminders' }).click();

  expect(await page.evaluate(() => sessionStorage.getItem('stock-promise:supervisor-session'))).toBeTruthy();
  expect(await page.evaluate(() => localStorage.getItem('stock-promise:operator'))).toBe('Browser operator');
  expect(await page.evaluate(() => localStorage.getItem('stock-promise:profiles'))).toBe(JSON.stringify(['Browser operator']));
  expect(await page.evaluate(() => localStorage.getItem('stock-promise:reminders'))).toBe('true');
  expect(await page.evaluate(() => localStorage.getItem('sb_license:inventory-promise-hold'))).toBe('browser-storage-license');
  expect(await page.evaluate(() => Object.keys(sessionStorage).some((key) => key.startsWith('demo:stock-promise:')))).toBe(false);
  await context.close();
});

test('@claim:csv-export demo exports a header and sample outcome', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await page.getByRole('button', { name: 'Outcomes' }).click();
  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export CSV' }).click();
  const stream = await (await download).createReadStream();
  let csv = ''; for await (const chunk of stream!) csv += chunk.toString();
  expect(csv).toContain('hold_id,sku,item,quantity,customer');
  expect(csv).toContain('Tideway Maintenance order 771');
  await context.close();
});

test('@claim:offline-demo sample opens offline after first visit', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, serviceWorkers: 'allow' });
  const page = await context.newPage();
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await page.evaluate(() => navigator.serviceWorker.ready);
  const cachedAssets = await page.evaluate(async () => {
    const keys = await caches.keys();
    const requests = (await Promise.all(keys.map(async (key) => (await (await caches.open(key)).keys()).map((request) => request.url)))).flat();
    return requests.filter((url) => /\/assets\/index-.*\.(?:js|css)$/.test(url));
  });
  expect(cachedAssets.some((url) => url.endsWith('.js'))).toBe(true);
  expect(cachedAssets.some((url) => url.endsWith('.css'))).toBe(true);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByText('Demo — sample data, nothing is saved.')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Promise desk' })).toBeVisible();
  await context.close();
});

test('@claim:pro-profiles-reminders a verified license saves profiles and sends an on-device expiry reminder', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  await context.addInitScript(() => {
    sessionStorage.setItem('demo:stock-promise:license', 'verified-test-license');
    sessionStorage.setItem('demo:stock-promise:license:verdict', JSON.stringify({ valid: true, checked: Date.now() }));
    class TestNotification {
      static permission = 'granted';
      static requestPermission = async () => 'granted';
      constructor(title: string, options?: NotificationOptions) {
        (window as Window & { __stockPromiseNotifications?: Array<{ title: string; body?: string }> }).__stockPromiseNotifications ??= [];
        (window as Window & { __stockPromiseNotifications: Array<{ title: string; body?: string }> }).__stockPromiseNotifications.push({ title, body: options?.body });
      }
    }
    Object.defineProperty(window, 'Notification', { configurable: true, value: TestNotification });
  });
  const page = await context.newPage();
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await page.evaluate(() => {
    const key = 'demo:stock-promise:state';
    const now = Math.floor(Date.now() / 1000);
    const data = {
      setup_required: false, location_name: 'Harbor Parts — sample', server_time: now, role: 'supervisor',
      inventory: [{ id: 1, sku: 'VALVE-24', name: 'Brass isolation valve', on_hand: 12, held: 3, available: 9 }],
      active_holds: [{ id: 'demo-due-soon', inventory_id: 1, sku: 'VALVE-24', item_name: 'Brass isolation valve', quantity: 3, customer: 'Northline Plumbing order 418', order_note: 'Counter pickup', operator_name: 'Mina', status: 'active', created_at: now - 300, expires_at: now + 120, resolved_at: null, resolved_by: null }],
      recent_outcomes: [],
    };
    sessionStorage.setItem(key, JSON.stringify(data));
  });
  await page.reload({ waitUntil: 'networkidle' });
  await page.getByRole('button', { name: 'Stock & settings' }).click();
  await expect(page.getByRole('heading', { name: 'Stock Promise Pro is active' })).toBeVisible();
  await page.getByLabel('Operator profile name').fill('Mina');
  await page.getByRole('button', { name: 'Save profile' }).click();
  await expect(page.getByRole('button', { name: 'Mina' }).first()).toBeVisible();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:stock-promise:profiles'))).toBe(JSON.stringify(['Mina']));
  await page.getByRole('button', { name: 'Enable 5-minute reminders' }).click();
  await expect(page.getByRole('button', { name: 'Reminders enabled' })).toBeVisible();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:stock-promise:reminders'))).toBe('true');
  await expect.poll(() => page.evaluate(() => (window as Window & { __stockPromiseNotifications?: unknown[] }).__stockPromiseNotifications?.length || 0)).toBe(1);
  await page.getByRole('button', { name: 'Live desk' }).click();
  await page.getByRole('button', { name: 'Create hold' }).first().click();
  await page.getByRole('button', { name: 'Mina' }).click();
  await expect(page.getByLabel('Your name')).toHaveValue('Mina');
  await context.close();
});

test('@claim:pro-license-restore verifies a pasted existing license without leaving the product', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const verificationRequests: string[] = [];
  await page.route('https://api.sociobot.in/api/v1/products/inventory-promise-hold/verify?license=restored-fixture', async (route) => {
    verificationRequests.push(route.request().url());
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok' }) });
  });
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await page.getByRole('button', { name: 'Stock & settings' }).click();
  await page.getByLabel('Have a license? Paste it here').fill('restored-fixture');
  await page.getByRole('button', { name: 'Verify license' }).click();
  await expect(page.getByRole('heading', { name: 'Stock Promise Pro is active' })).toBeVisible();
  expect(await page.evaluate(() => sessionStorage.getItem('demo:stock-promise:license'))).toBe('restored-fixture');
  expect(await page.evaluate(() => localStorage.getItem('sb_license:inventory-promise-hold'))).toBeNull();
  expect(verificationRequests).toEqual(['https://api.sociobot.in/api/v1/products/inventory-promise-hold/verify?license=restored-fixture']);
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.getByRole('heading', { name: 'Pro reminders & profiles' })).toBeVisible();
  expect(await page.evaluate(() => Object.keys(sessionStorage).filter((key) => key.startsWith('demo:stock-promise:')))).toEqual([]);
  await context.close();
});

test('@claim:pro-checkout-status does not offer the recorded unavailable checkout route', async ({ page }) => {
  await page.goto('/', { waitUntil: 'networkidle' });
  await expect(page.locator('.plain-facts').getByText('New Pro purchases are temporarily unavailable.')).toBeVisible();
  expect(await page.locator('a[href*="/checkout"]').count()).toBe(0);
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await page.getByRole('button', { name: 'Stock & settings' }).click();
  await expect(page.getByText('Existing license holders can restore a license below.')).toBeVisible();
  expect(await page.locator('a[href*="/checkout"]').count()).toBe(0);
});

test('@claim:core-features-no-pro creates a hold and exports CSV without a license', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await page.getByRole('button', { name: 'Create hold' }).first().click();
  await page.getByLabel('Quantity').fill('1');
  await page.getByLabel('Customer or order reference').fill('No Pro counter order');
  await page.getByLabel('Your name').fill('Free staff');
  await page.getByRole('button', { name: 'Hold VALVE-24' }).click();
  await expect(page.getByText('Sample hold created for No Pro counter order.')).toBeVisible();
  await page.getByRole('button', { name: 'Outcomes' }).click();
  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export CSV' }).click();
  expect((await download).suggestedFilename()).toBe('stock-promise-holds.csv');
  await page.getByRole('button', { name: 'Stock & settings' }).click();
  await expect(page.getByText('New Pro purchases are temporarily unavailable.')).toBeVisible();
  await context.close();
});
