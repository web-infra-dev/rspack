import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig, devices } from '@playwright/test';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const hostDir = __dirname;
const remoteDir = path.resolve(__dirname, '..', 'rsc-mf-remote');

export default defineConfig({
  testDir: path.join(__dirname, 'e2e'),
  timeout: 120 * 1000,
  expect: {
    timeout: 20 * 1000,
  },
  fullyParallel: false,
  workers: 1,
  reporter: 'list',
  use: {
    ...devices['Desktop Chrome'],
    baseURL: 'http://localhost:1716',
    trace: 'retain-on-failure',
  },
  webServer: [
    {
      command: 'rm -f todos.json && NO_CSP=true node server.js',
      cwd: remoteDir,
      url: 'http://localhost:1717/remoteEntry.cjs',
      timeout: 180 * 1000,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'rm -f todos.json && NO_CSP=true node server.js',
      cwd: hostDir,
      url: 'http://localhost:1716/',
      timeout: 180 * 1000,
      reuseExistingServer: !process.env.CI,
    },
  ],
});
