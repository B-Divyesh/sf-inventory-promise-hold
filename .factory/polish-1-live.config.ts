import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  testMatch: 'polish-1-live.spec.ts',
  timeout: 60_000,
  workers: 1,
  reporter: 'line',
  use: { trace: 'retain-on-failure' },
});
