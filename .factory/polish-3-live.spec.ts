import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const base = process.env.QA_BASE_URL || 'https://inventory-promise-hold.sociobot.in';
const expectedSha = process.env.EXPECTED_BUILD_SHA || '';
const artifacts = '.factory/qa-artifacts';

async function expectNoSeriousAxe(page: import('@playwright/test').Page) {
  const scan = await new AxeBuilder({ page }).analyze();
  expect(scan.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
}

test('round 3 live first screen has all facts and the data-backed sample preview', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 }, reducedMotion: 'reduce' });
  const page = await context.newPage();
  const consoleErrors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });
  await page.goto(base, { waitUntil: 'networkidle' });
  await expect(page).toHaveTitle('Stock Promise — timed inventory holds');
  await expect(page.getByRole('heading', { name: 'Hold scarce stock before it is promised twice.' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Try it with sample data' })).toBeVisible();
  await expect(page.locator('.plain-facts')).toContainText('The sample never changes a live stockroom.');
  await expect(page.locator('.plain-facts')).toContainText('The sample opens offline after your first visit.');
  await expect(page.locator('.plain-facts')).toContainText('Paid upgrades are temporarily unavailable.');
  const facts = await page.locator('.plain-facts').boundingBox();
  expect(facts!.y + facts!.height).toBeLessThanOrEqual(844);
  const preview = page.locator('.sample-preview');
  await expect(preview.getByRole('heading', { name: 'Preview sample inventory holds' })).toBeVisible();
  await expect(preview.locator('.preview-stock > li')).toHaveCount(3);
  await expect(preview).toContainText('Northline Plumbing order 418');
  await expect(page.getByRole('heading', { name: 'Pro profiles and reminders' })).toHaveCount(0);
  await expectNoSeriousAxe(page);
  expect(consoleErrors).toEqual([]);
  await page.screenshot({ path: `${artifacts}/polish-3-live-first-screen.png`, fullPage: false });
  await context.close();
});

test('round 3 live sample, privacy, access, legal, and 404 routes stay accurate', async ({ browser, request }) => {
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await context.newPage();
  await page.goto(`${base}/?demo=1`, { waitUntil: 'networkidle' });
  await expect(page.getByText('Demo — sample data, nothing is saved.')).toBeVisible();
  await expect(page.locator('.inventory-list > li')).toHaveCount(3);
  await expect(page.locator('.hold-list > li')).toHaveCount(1);
  await expectNoSeriousAxe(page);
  await page.screenshot({ path: `${artifacts}/polish-3-live-demo.png`, fullPage: true });

  await page.goto(`${base}/privacy`, { waitUntil: 'networkidle' });
  await expect(page.getByText('Sociobot sign-in tokens stay in the current browser session.')).toBeVisible();
  await expect(page.getByText('local storage.')).toBeVisible();
  await expectNoSeriousAxe(page);

  await page.goto(base, { waitUntil: 'networkidle' });
  await page.getByRole('button', { name: 'Open inventory holds' }).click();
  await expect(page.getByRole('heading', { name: 'Open inventory holds.' })).toBeVisible();
  await expect(page.getByText('Sign in to view this location’s stock and customer references.')).toBeVisible();
  await expectNoSeriousAxe(page);

  await page.goto(`${base}/terms`, { waitUntil: 'networkidle' });
  await expect(page.getByRole('heading', { name: 'Existing Pro licenses' })).toBeVisible();
  await expect(page.getByText('New purchases are temporarily unavailable.')).toBeVisible();
  await expectNoSeriousAxe(page);

  const missing = await page.goto(`${base}/round-3-not-found`, { waitUntil: 'networkidle' });
  expect(missing?.status()).toBe(404);
  await expect(page.getByRole('heading', { name: 'Page not found' })).toBeVisible();
  await expect(page.locator('footer')).toContainText(`build ${expectedSha.slice(0, 12)}`);
  await expectNoSeriousAxe(page);
  await page.screenshot({ path: `${artifacts}/polish-3-live-404.png`, fullPage: true });

  const health = await request.get(`${base}/health`);
  expect(health.status()).toBe(200);
  if (expectedSha) expect((await health.json()).build_sha).toBe(expectedSha);
  expect((await request.get(`${base}/api/bootstrap`)).status()).toBe(401);
  await context.close();
});
