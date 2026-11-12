import type { PlaywrightOptions } from '@rstest/playwright';
import { devices } from 'playwright';
import { test as base } from '../e2e/fixtures/base';

export { expect } from '../e2e/fixtures/base';

export const test = base.extend({
  playwright: {
    contextOptions: {
      ...devices['Desktop Chrome'],
      baseURL: 'http://localhost:8900',
    },
    trace: 'on-first-retry',
  } satisfies PlaywrightOptions,
});
