import fs from 'node:fs';
import { defineConfig, devices } from '@playwright/test';

const chromePath = '/usr/bin/google-chrome';
const hasSystemChrome = fs.existsSync(chromePath);

export default defineConfig({
  testDir: './e2e',
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  fullyParallel: false,
  reporter: 'line',
  use: {
    baseURL: 'http://localhost:3330',
    trace: 'on-first-retry',
  },
  webServer: {
    command: 'pnpm run dev:e2e',
    cwd: import.meta.dirname,
    url: 'http://localhost:3330',
    reuseExistingServer: false,
    timeout: 120_000,
  },
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: hasSystemChrome
          ? {
              executablePath: chromePath,
              args: ['--no-sandbox'],
            }
          : {
              args: ['--no-sandbox'],
            },
      },
    },
  ],
});
