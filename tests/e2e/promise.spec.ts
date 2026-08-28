import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('sets up a location and completes the hold lifecycle on a phone', async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') consoleErrors.push(message.text()); });

  await page.goto('/');
  await expect(page).toHaveTitle(/Stock Promise/);
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.locator('h1')).toHaveCount(1);

  await page.getByLabel('Location name').fill('Test counter');
  await page.getByLabel(/Supervisor PIN/).fill('246810');
  await page.getByRole('button', { name: 'Open the promise desk' }).click();
  await expect(page.getByRole('heading', { name: 'No stock is listed yet' })).toBeVisible();

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
  await expect(consoleErrors).toEqual([]);
});

test('legal pages stay semantic and reachable', async ({ page }) => {
  await page.goto('/privacy');
  await expect(page.locator('h1')).toHaveCount(1);
  await expect(page.getByRole('heading', { name: /Privacy, kept close/ })).toBeVisible();
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations.filter((item) => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  await page.getByRole('button', { name: 'Return to the promise desk' }).click();
  await expect(page).toHaveURL('/');
});
