import { test as base, type PlaywrightOptions } from '@rstest/playwright';
import { devices } from 'playwright';

export { expect } from './expect';

export const test = base.extend({
  playwright: {
    contextOptions: devices['Desktop Chrome'],
    trace: 'on-first-retry',
  } satisfies PlaywrightOptions,
  browserTimeouts: [
    async ({ context }, use) => {
      // Let the test timeout bound actions and navigation, as Playwright Test did.
      context.setDefaultTimeout(0);
      context.setDefaultNavigationTimeout(0);
      await use(undefined);
    },
    { auto: true },
  ],
});
