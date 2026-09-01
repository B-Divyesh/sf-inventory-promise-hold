import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const base = 'https://inventory-promise-hold.sociobot.in';
const expectedSha = process.env.EXPECTED_BUILD_SHA || '';

function collectFailures(page: import('@playwright/test').Page) {
  const consoleErrors: string[] = [];
  const pageErrors: string[] = [];
  const failedRequests: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', (error) => pageErrors.push(error.message));
  page.on('requestfailed', (request) => failedRequests.push(`${request.method()} ${request.url()}: ${request.failure()?.errorText}`));
  return { consoleErrors, pageErrors, failedRequests };
}

async function expectNoSeriousAxe(page: import('@playwright/test').Page) {
  const scan = await new AxeBuilder({ page }).analyze();
  expect(scan.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
}

test('cold mobile first screen and one-click query demo', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, reducedMotion: 'reduce' });
  const page = await context.newPage();
  const failures = collectFailures(page);
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await page.goto(base, { waitUntil: 'networkidle' });
  await expect(page).toHaveTitle('Stock Promise — timed inventory holds');
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.locator('h1')).toHaveCount(1);
  await expect(page.getByRole('heading', { name: 'Hold scarce stock before it is promised twice.' })).toBeVisible();
  await expect(page.getByText('Open a sample stockroom.')).toBeVisible();
  const facts = await page.locator('.plain-facts').boundingBox();
  expect(facts).not.toBeNull();
  expect(facts!.y + facts!.height).toBeLessThanOrEqual(844);
  await expectNoSeriousAxe(page);

  await page.getByRole('link', { name: 'Try it with sample data' }).click();
  await expect(page).toHaveURL(`${base}/?demo=1`);
  await expect(page).toHaveTitle('Demo — Stock Promise');
  await expect(page.locator('.demo-banner')).toContainText('Demo — sample data, nothing is saved.');
  await expect(page.locator('.inventory-list > li')).toHaveCount(3);
  await expect(page.locator('.hold-list > li')).toHaveCount(1);
  await expect(page.getByText('Northline Plumbing order 418')).toBeVisible();
  await expect(page.locator('.header-status')).not.toContainText('Shared live');
  await expect(page.getByRole('button', { name: 'Lock supervisor' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Reset demo' }).click();
  expect(await page.evaluate(() => Object.keys(sessionStorage).filter((key) => key.startsWith('demo:stock-promise:')))).toEqual([]);
  expect(requests.every((url) => new URL(url).origin === base)).toBe(true);
  expect(requests.some((url) => new URL(url).pathname.startsWith('/api/'))).toBe(false);
  await expectNoSeriousAxe(page);
  expect(failures).toEqual({ consoleErrors: [], pageErrors: [], failedRequests: [] });
  await context.close();
});

test('desktop demo labels isolated sample state', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  const failures = collectFailures(page);
  await page.goto(`${base}/?demo=1`, { waitUntil: 'networkidle' });
  await expect(page.locator('.header-status')).toHaveText('Sample data');
  await expect(page.locator('.header-status')).not.toContainText('Shared live');
  await expect(page.getByRole('button', { name: 'Lock supervisor' })).toHaveCount(0);
  expect(failures).toEqual({ consoleErrors: [], pageErrors: [], failedRequests: [] });
  await context.close();
});

test('route metadata, history focus, legal copy, and static 404 shell', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const failures = collectFailures(page);
  await page.goto(base, { waitUntil: 'networkidle' });
  await page.locator('.top-nav').getByRole('link', { name: 'Privacy' }).click();
  await expect(page).toHaveURL(`${base}/privacy`);
  await expect(page).toHaveTitle('Privacy — Stock Promise');
  await expect(page.getByRole('heading', { name: 'Privacy for Stock Promise' })).toBeFocused();
  await expect(page.getByText('The audit record keeps past changes and cannot be edited.')).toBeVisible();
  await expectNoSeriousAxe(page);
  await page.goBack();
  await expect(page.getByRole('heading', { name: 'Hold scarce stock before it is promised twice.' })).toBeFocused();
  await page.goForward();
  await expect(page.getByRole('heading', { name: 'Privacy for Stock Promise' })).toBeFocused();

  await page.goto(`${base}/terms`, { waitUntil: 'networkidle' });
  await expect(page).toHaveTitle('Terms — Stock Promise');
  await expect(page.getByText('Do not interfere with normal service use or present inaccurate stock availability to customers.')).toBeVisible();
  await expectNoSeriousAxe(page);

  const response = await page.goto(`${base}/not-a-real-route`, { waitUntil: 'networkidle' });
  expect(response?.status()).toBe(404);
  await expect(page).toHaveTitle('Page not found — Stock Promise');
  await expect(page.getByRole('heading', { name: 'Page not found' })).toBeVisible();
  await expect(page.locator('header .wordmark')).toBeVisible();
  await expect(page.locator('header nav').getByRole('link', { name: 'Demo' })).toBeVisible();
  await expect(page.locator('footer')).toBeVisible();
  await expect(page.locator('link[rel="icon"]')).toHaveCount(1);
  await expect(page.locator('link[rel="apple-touch-icon"]')).toHaveCount(1);
  await expectNoSeriousAxe(page);
  expect(failures.consoleErrors.filter((message) => !message.includes('server responded with a status of 404'))).toEqual([]);
  expect(failures.pageErrors).toEqual([]);
  expect(failures.failedRequests).toEqual([]);
  await context.close();
});

test('live identity, public files, links, headers, and rate allowance', async ({ request }) => {
  const health = await request.get(`${base}/health`);
  expect(health.status()).toBe(200);
  const identity = await health.json();
  if (expectedSha) expect(identity.build_sha).toBe(expectedSha);
  expect(health.headers()['cache-control']).toBe('no-store');
  expect(health.headers()['strict-transport-security']).toContain('max-age=31536000');
  expect(health.headers()['permissions-policy']).toContain('camera=()');

  for (const route of ['/', '/?demo=1', '/demo', '/privacy', '/terms', '/robots.txt', '/sitemap.xml', '/mark.svg', '/apple-touch-icon.png']) {
    expect((await request.get(`${base}${route}`)).status(), route).toBe(200);
  }
  expect((await request.get(`${base}/not-a-real-route`)).status()).toBe(404);

  const forwarded = `198.51.100.${Math.floor(Math.random() * 180) + 20}`;
  let limited: import('@playwright/test').APIResponse | null = null;
  for (let index = 0; index < 90; index += 1) {
    const response = await request.get(`${base}/api/status`, { headers: { 'x-forwarded-for': forwarded } });
    if (response.status() === 429) { limited = response; break; }
  }
  expect(limited).not.toBeNull();
  expect(Number(limited!.headers()['retry-after'])).toBeGreaterThan(0);
});

test('sample demo reloads offline after its first visit', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, serviceWorkers: 'allow' });
  const page = await context.newPage();
  await page.goto(`${base}/?demo=1`, { waitUntil: 'networkidle' });
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload({ waitUntil: 'networkidle' });
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.locator('.demo-banner')).toContainText('Demo — sample data, nothing is saved.');
  await expect(page.getByRole('heading', { name: 'Promise desk' })).toBeVisible();
  await context.close();
});
