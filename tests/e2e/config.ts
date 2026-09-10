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
  const collectedFiles: TestFileInfo[] = [];
  return {
    // These tests run in Node; do not inherit the browser app's Rsbuild config.
    extends: {},
    include: ['cases/**/*.{test,spec}.?(c|m)[jt]s?(x)'],
    // Also used by the shared expect fixture for browser assertion timeouts.
    testTimeout: timeout,
    hookTimeout: timeout,
    expect: { poll: { timeout, interval: 100 } },
    pool: { maxWorkers: process.env.CI ? ciWorkers : '50%' },
    slowTestThreshold: 300_000,
    reporters: [
      'default',
      ['json', { outputPath: 'test-results/results.json' }],
      ...(process.env.CI
        ? [
            {
              onTestFileReady: (file: TestFileInfo) => {
                collectedFiles.push(file);
              },
              onTestRunEnd: () => {
                for (const { tests } of collectedFiles) checkOnly(tests);
              },
            },
          ]
        : []),
    ],
  };
}
