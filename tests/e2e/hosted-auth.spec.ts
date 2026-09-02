import { expect, test } from '@playwright/test';

test('@claim:hosted-token-storage hosted Sociobot callback keeps the MSAL token cache in session storage', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const bootstrapAuthorization: string[] = [];

  await page.route('**/api/bootstrap', async (route) => {
    bootstrapAuthorization.push(route.request().headers().authorization || '');
    await route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'Fixture callback ends before live data access.' }),
    });
  });
  await page.goto('/auth/callback?test-hosted-auth=1', { waitUntil: 'networkidle' });
  await expect(page.getByRole('heading', { name: 'Open inventory holds.' })).toBeVisible();

  const stores = await page.evaluate(() => ({
    local: Object.entries(localStorage),
    session: Object.entries(sessionStorage),
  }));
  const sessionCache = stores.session.filter(([key, value]) => /msal/i.test(key) || value.includes('hosted-fixture'));
  const localCache = stores.local.filter(([key, value]) => /msal/i.test(key) || value.includes('hosted-fixture'));
  expect(sessionCache.length).toBeGreaterThan(0);
  expect(sessionCache.some(([, value]) => value.includes('hosted-fixture-access-token') || value.includes('hosted-fixture-user'))).toBe(true);
  expect(localCache).toEqual([]);
  expect(bootstrapAuthorization).toEqual(['Bearer hosted-fixture-access-token']);
  await context.close();
});
