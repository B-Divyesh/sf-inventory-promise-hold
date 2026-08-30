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
  await page.goto('/privacy');
  await expect(page.locator('h1')).toHaveCount(1);
  await expect(page).toHaveTitle('Privacy — Stock Promise');
  await expect(page.getByRole('heading', { name: 'Privacy for Stock Promise' })).toBeVisible();
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  await page.getByRole('button', { name: 'Return home' }).click();
  await expect(page).toHaveURL('/');
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
  expect(script).toBe('http://127.0.0.1:4178/sw.js');
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByText(/Demo.*sample data/i)).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Promise desk' })).toBeVisible();
  await context.close();
});

test('@claim:demo-isolated sample changes never call a live write API', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const writes: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('/api/') && !['GET', 'HEAD'].includes(request.method())) writes.push(request.url());
  });
  await page.goto('/demo', { waitUntil: 'networkidle' });
  await expect(page.getByText('Demo — sample data, nothing is saved.')).toBeVisible();
  await page.getByRole('button', { name: 'Create hold' }).first().click();
  await page.getByLabel('Quantity').fill('1');
  await page.getByLabel('Customer or order reference').fill('Demo counter order 102');
  await page.getByLabel('Your name').fill('Demo staff');
  await page.getByRole('button', { name: 'Hold VALVE-24' }).click();
  await expect(page.getByText(/Sample hold created for Demo counter order 102/)).toBeVisible();
  expect(writes).toEqual([]);
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
  await page.reload({ waitUntil: 'networkidle' });
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByText('Demo — sample data, nothing is saved.')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Promise desk' })).toBeVisible();
  await context.close();
});
