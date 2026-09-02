import { defineConfig, devices } from '@playwright/test';

/**
 * This server deliberately keeps AUTH_MODE=ciam, matching the hosted access
 * contract. VITE_HOSTED_AUTH_FIXTURE only enables the browser-side identity
 * exchange fixture used by hosted-auth.spec.ts; release builds omit it.
 */
export default defineConfig({
  testDir: './tests/e2e',
  testMatch: 'hosted-auth.spec.ts',
  timeout: 30_000,
  fullyParallel: false,
  workers: 1,
  reporter: 'line',
  use: {
    baseURL: 'http://127.0.0.1:4179',
    trace: 'retain-on-failure',
  },
  projects: [
    { name: 'hosted-auth-chromium', use: { ...devices['Desktop Chrome'], viewport: { width: 390, height: 844 } } },
  ],
  webServer: {
    command: 'AUTH_MODE=ciam PORT=4179 DATABASE_PATH=target/stock-promise-hosted-auth.db FRONTEND_DIR=dist cargo run --quiet',
    url: 'http://127.0.0.1:4179/health',
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
