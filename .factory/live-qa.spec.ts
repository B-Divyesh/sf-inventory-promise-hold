import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const base = process.env.QA_BASE_URL || 'https://inventory-promise-hold.sociobot.in';

for (const profile of [
  { name: 'desktop', width: 1440, height: 1000 },
  { name: 'mobile-390', width: 390, height: 844 },
]) {
  test(`${profile.name}: live semantics, privacy, and accessibility`, async ({ browser }) => {
    const context = await browser.newContext({ viewport: profile, reducedMotion: 'reduce' });
    const page = await context.newPage();
    const consoleErrors: string[] = [];
    const pageErrors: string[] = [];
    const failedRequests: string[] = [];
    const origins = new Set<string>();
    page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
    page.on('pageerror', (error) => pageErrors.push(error.message));
    page.on('requestfailed', (request) => failedRequests.push(`${request.method()} ${request.url()}: ${request.failure()?.errorText}`));
    page.on('request', (request) => origins.add(new URL(request.url()).origin));

    await page.goto(base, { waitUntil: 'networkidle' });
    await expect(page).toHaveTitle('Stock Promise — live inventory holds');
    await expect(page.locator('html')).toHaveAttribute('lang', 'en');
    await expect(page.locator('main')).toHaveCount(1);
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page.getByText('QA candidate e826a45')).toBeVisible();

    const accessibility = await new AxeBuilder({ page }).analyze();
    expect(accessibility.violations.filter((v) => ['serious', 'critical'].includes(v.impact || ''))).toEqual([]);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
    expect(await page.evaluate(() => matchMedia('(prefers-reduced-motion: reduce)').matches)).toBe(true);
    expect([...origins]).toEqual([base]);
    expect(consoleErrors).toEqual([]);
    expect(pageErrors).toEqual([]);
    expect(failedRequests).toEqual([]);
    await page.screenshot({ path: `/tmp/stock-promise-${profile.name}.png`, fullPage: true });
    await context.close();
  });
}

test('desktop keyboard lifecycle, invalid PIN recovery, and CSV', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  const consoleErrors: string[] = [];
  const pageErrors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  page.on('pageerror', (error) => pageErrors.push(error.message));
  await page.goto(base, { waitUntil: 'networkidle' });

  await page.keyboard.press('Tab');
  expect(await page.evaluate(() => (document.activeElement as HTMLElement)?.textContent?.trim())).toBe('Skip to promise desk');
  const skipOutline = await page.locator('.skip-link').evaluate((el) => getComputedStyle(el).outlineStyle);
  expect(skipOutline).not.toBe('none');
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(`${base}/#main`);

  await page.getByRole('button', { name: 'Supervisor unlock' }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('dialog')).toBeVisible();
  expect(await page.evaluate(() => document.querySelector('dialog')?.contains(document.activeElement))).toBe(true);
  await page.getByLabel('Supervisor PIN').fill('000000');
  await page.keyboard.press('Enter');
  await expect(page.getByRole('alert')).toContainText('not correct');
  await page.getByLabel('Supervisor PIN').fill('864209');
  await page.keyboard.press('Enter');
  await expect(page.getByRole('button', { name: 'Lock supervisor' })).toBeVisible();
  await expect(page.locator('.live-region')).toHaveText('Supervisor controls unlocked for this tab.');
  consoleErrors.length = 0;

  await page.getByPlaceholder('Search SKU or item').fill('does-not-exist');
  await expect(page.getByRole('heading', { name: /No items match/ })).toBeVisible();
  await page.getByRole('button', { name: 'Clear search' }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByText('5 available')).toBeVisible();

  const normalRow = page.getByRole('listitem').filter({ hasText: 'QA-NORMAL' });
  await normalRow.getByRole('button', { name: 'Create hold' }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('dialog')).toBeVisible();
  expect(await page.evaluate(() => document.querySelector('dialog')?.contains(document.activeElement))).toBe(true);
  await page.getByLabel('Quantity').fill('2');
  await page.getByLabel('Customer or order reference').fill('Keyboard Customer');
  await page.getByLabel('Your name').fill('QA Operator');
  await page.getByLabel(/Order note/).fill('Candidate live verification, commas included');
  await page.getByRole('button', { name: 'Hold QA-NORMAL' }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByText('3 available')).toBeVisible();
  await expect(page.getByText(/For Keyboard Customer/)).toBeVisible();

  page.once('dialog', (dialog) => dialog.accept());
  const heldRow = page.locator('.hold-list li').filter({ hasText: 'Keyboard Customer' });
  await heldRow.getByRole('button', { name: 'Convert', exact: true }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByText('No stock is tied up.')).not.toBeVisible();
  await expect(page.getByText('3 available')).toBeVisible();

  await page.getByRole('button', { name: 'Outcomes' }).click();
  await expect(page.getByText('converted', { exact: true })).toBeVisible();
  const downloadEvent = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export CSV' }).click();
  const download = await downloadEvent;
  expect(download.suggestedFilename()).toBe('stock-promise-holds.csv');
  const stream = await download.createReadStream();
  let csv = '';
  for await (const chunk of stream!) csv += chunk.toString();
  expect(csv).toContain('hold_id,sku,item,quantity,customer');
  expect(csv).toContain('QA-NORMAL');
  expect(csv).toContain('Keyboard Customer');

  expect(consoleErrors).toEqual([]);
  expect(pageErrors).toEqual([]);
  await context.close();
});

test('PWA controls the page and reloads offline', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, serviceWorkers: 'allow' });
  const page = await context.newPage();
  await page.goto(base, { waitUntil: 'networkidle' });
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload({ waitUntil: 'networkidle' });
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
  const registration = await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    await registration.update();
    return { scope: registration.scope, script: registration.active?.scriptURL };
  });
  expect(registration.scope).toBe(`${base}/`);
  expect(registration.script).toBe(`${base}/sw.js`);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByText(/You’re offline/)).toBeVisible();
  await expect(page.getByRole('heading', { name: /promise desk can’t open yet/ })).toBeVisible();
  await context.close();
});
