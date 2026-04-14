import { defineConfig, devices } from '@playwright/test';

const TIMEOUT = 120 * 1000;

export default defineConfig({
  testDir: './cases',
  forbidOnly: !!process.env.CI,
  retries: 0,
  timeout: TIMEOUT,
  expect: {
    timeout: TIMEOUT,
  },
  workers: process.env.CI ? 2 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:8900',
    trace: 'on-first-retry',
    ...devices['Desktop Chrome'],
  },
  webServer: {
    command: 'npx rsbuild dev',
    port: 8900,
    reuseExistingServer: !process.env.CI,
    timeout: 60 * 1000,
  },
});
