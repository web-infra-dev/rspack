import { definePlaywrightConfig } from '@rstest/playwright/config';
import type { RstestConfig, TestFileInfo, TestInfo } from 'rstack/test';

// Rstest does not yet expose Playwright Test's forbidOnly option.
function checkOnly(tests: TestInfo[]) {
  for (const test of tests) {
    if (test.runMode === 'only') {
      throw new Error(`Unexpected .only in CI: ${test.testPath}: ${test.name}`);
    }
    if (test.type === 'suite') {
      checkOnly(test.tests);
    }
  }
}

export function e2eConfig(timeout: number, ciWorkers: number): RstestConfig {
  return {
    // These tests run in Node; do not inherit the browser app's Rsbuild config.
    extends: definePlaywrightConfig({}),
    include: ['cases/**/*.{test,spec}.?(c|m)[jt]s?(x)'],
    testTimeout: timeout,
    hookTimeout: timeout,
    // Also controls Playwright locator and page assertion timeouts.
    expect: { poll: { timeout, interval: 100 } },
    pool: { maxWorkers: process.env.CI ? ciWorkers : '50%' },
    slowTestThreshold: 300_000,
    reporters: [
      'default',
      ['json', { outputPath: 'test-results/results.json' }],
      ...(process.env.CI
        ? [
            {
              onTestFileReady: ({ tests }: TestFileInfo) => checkOnly(tests),
            },
          ]
        : []),
    ],
  };
}
