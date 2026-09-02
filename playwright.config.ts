import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  testIgnore: '**/hosted-auth.spec.ts',
  timeout: 30_000,
  fullyParallel: false,
  workers: 1,
  reporter: 'line',
  use: {
    baseURL: 'http://127.0.0.1:4178',
    trace: 'retain-on-failure',
  },
  projects: [
    { name: 'mobile-chromium', use: { ...devices['Desktop Chrome'], viewport: { width: 390, height: 844 } } },
  ],
  webServer: {
    command: 'AUTH_MODE=local PORT=4178 DATABASE_PATH=target/stock-promise-e2e.db FRONTEND_DIR=dist cargo run --quiet',
    url: 'http://127.0.0.1:4178/health',
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
