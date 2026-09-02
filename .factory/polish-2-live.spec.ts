import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const base = 'https://inventory-promise-hold.sociobot.in';
const expectedSha = process.env.EXPECTED_BUILD_SHA || '';

async function expectAccessible(page: import('@playwright/test').Page) {
  const scan = await new AxeBuilder({ page }).analyze();
  expect(scan.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
}

test('cold mobile and desktop first screens state the job and action', async ({ browser }) => {
  for (const viewport of [{ width: 390, height: 844 }, { width: 1440, height: 1000 }]) {
    const context = await browser.newContext({ viewport, reducedMotion: 'reduce' });
    const page = await context.newPage();
    const consoleErrors: string[] = [];
    page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
    await page.goto(base, { waitUntil: 'networkidle' });
    await expect(page).toHaveTitle('Stock Promise — timed inventory holds');
    await expect(page.getByRole('heading', { name: 'Hold scarce stock before it is promised twice.' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'Try it with sample data' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Open inventory holds' })).toBeVisible();
    await expect(page.locator('.connection')).toHaveCount(0);
    const facts = await page.locator('.plain-facts').boundingBox();
    if (viewport.width === 390) expect(facts!.y + facts!.height).toBeLessThanOrEqual(844);
    await expectAccessible(page);
    expect(consoleErrors).toEqual([]);
    await page.screenshot({
      path: `/tmp/polish-2-live-${viewport.width === 390 ? 'mobile-first-screen' : 'desktop-first-screen'}.png`,
      fullPage: false,
    });
    await context.close();
  }
});

test('one-click demo invalidates a delayed live license response', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  await context.addInitScript(() => {
    localStorage.setItem('stock-promise:operator', 'Real operator');
    localStorage.setItem('sb_license:inventory-promise-hold', 'real-live-license');
    localStorage.setItem('sb_license:inventory-promise-hold:verdict', JSON.stringify({ valid: true, checked: 1 }));
  });
  const page = await context.newPage();
  let started = () => {};
  const verificationStarted = new Promise<void>((resolve) => { started = resolve; });
  let release = () => {};
  const verificationReleased = new Promise<void>((resolve) => { release = resolve; });
  await page.route('https://api.sociobot.in/api/v1/products/inventory-promise-hold/verify?license=*', async (route) => {
    started();
    await verificationReleased;
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ valid: false, reason: 'revoked' }) }).catch(() => {});
  });
  await page.goto(base, { waitUntil: 'domcontentloaded' });
  await verificationStarted;
  const before = await page.evaluate(() => JSON.stringify(Object.fromEntries(Object.keys(localStorage).sort().map((key) => [key, localStorage.getItem(key)]))));
  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await expect(page).toHaveURL(`${base}/?demo=1`);
  await expect(page.getByText('Demo — sample data, nothing is saved.')).toBeVisible();
  release();
  await page.waitForTimeout(100);
  const after = await page.evaluate(() => JSON.stringify(
    Object.fromEntries(Object.keys(localStorage).sort().map((key) => [key, localStorage.getItem(key)])),
  ));
  expect(after).toBe(before);
  await expect(page.locator('.inventory-list > li')).toHaveCount(3);
  await expect(page.locator('.hold-list > li')).toHaveCount(1);
  await expect(page.getByRole('heading', { name: 'Manage sample inventory holds' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Leave demo' })).toBeVisible();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  expect(await page.evaluate(() => Object.keys(sessionStorage).filter((key) => key.startsWith('demo:stock-promise:')))).toEqual([]);
  await expectAccessible(page);
  await page.screenshot({ path: '/tmp/polish-2-live-demo.png', fullPage: true });
  await context.close();
});

test('legal routes, history focus, and 404 keep the complete shell', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await page.goto(base, { waitUntil: 'networkidle' });
  await page.locator('.top-nav').getByRole('link', { name: 'Privacy' }).click();
  await expect(page).toHaveTitle('Privacy — Stock Promise');
  await expect(page.getByRole('heading', { name: 'Privacy for Stock Promise' })).toBeFocused();
  await expect(page.getByRole('link', { name: 'Return home' })).toHaveAttribute('href', '/');
  await expectAccessible(page);
  await page.goBack();
  await expect(page.getByRole('heading', { name: 'Hold scarce stock before it is promised twice.' })).toBeFocused();
  await page.goForward();
  await expect(page.getByRole('heading', { name: 'Privacy for Stock Promise' })).toBeFocused();

  const missing = await page.goto(`${base}/not-a-real-route`, { waitUntil: 'networkidle' });
  expect(missing?.status()).toBe(404);
  await expect(page.getByRole('heading', { name: 'Page not found' })).toBeVisible();
  await expect(page.locator('header nav').getByRole('link', { name: 'Demo' })).toBeVisible();
  await expect(page.locator('footer').getByRole('link', { name: 'Terms' })).toBeVisible();
  await expect(page.locator('footer')).toContainText(`build ${expectedSha.slice(0, 12)}`);
  await expectAccessible(page);
  await page.screenshot({ path: '/tmp/polish-2-live-404.png', fullPage: true });
  await context.close();
});

test('public routes, headers, offline demo, and rate limit work cold', async ({ browser, request }) => {
  const health = await request.get(`${base}/health`);
  expect(health.status()).toBe(200);
  expect((await health.json()).build_sha).toBe(expectedSha);
  expect(health.headers()['cache-control']).toBe('no-store');
  expect(health.headers()['strict-transport-security']).toContain('max-age=31536000');
  for (const route of ['/', '/?demo=1', '/demo', '/privacy', '/terms', '/robots.txt', '/sitemap.xml']) {
    expect((await request.get(`${base}${route}`)).status(), route).toBe(200);
  }
  const forwarded = `198.51.100.${Math.floor(Math.random() * 180) + 20}`;
  let limited: import('@playwright/test').APIResponse | null = null;
  for (let index = 0; index < 90; index += 1) {
    const response = await request.get(`${base}/api/status`, { headers: { 'x-forwarded-for': forwarded } });
    if (response.status() === 429) { limited = response; break; }
  }
  expect(limited).not.toBeNull();
  expect(Number(limited!.headers()['retry-after'])).toBeGreaterThan(0);

  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, serviceWorkers: 'allow' });
  const page = await context.newPage();
  await page.goto(`${base}/?demo=1`, { waitUntil: 'networkidle' });
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload({ waitUntil: 'networkidle' });
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByRole('heading', { name: 'Manage sample inventory holds' })).toBeVisible();
  await context.close();
});
