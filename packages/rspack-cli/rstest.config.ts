import path from 'node:path';
import { defineConfig, type RstestConfig } from '@rstest/core';
import { StreamedTextReporter } from '../../scripts/test/streamed-reporter';

const wasmConfig: RstestConfig | undefined = process.env.WASM
  ? {
      exclude: [
        '**/*/profile.test.ts', // Skip due to lack of system api support
      ],
      pool: {
        maxWorkers: 1,
      },
      maxConcurrency: 1,
    }
  : undefined;

const streamedReporter = new StreamedTextReporter(
  path.join(__dirname, '../../rspack-cli-streamed-report.txt'),
);

export default defineConfig({
  testEnvironment: 'node',
  globals: true,
  testTimeout: process.env.CI ? 200000 : 30000,
  include: ['tests/**/*.test.{ts,js,cts}'],
  source: {
    tsconfigPath: 'tests/tsconfig.json',
  },
  output: {
    externals: [/@rspack\/core/, /api-wrapper/],
    module: false,
  },
  env: {
    RUST_BACKTRACE: 'full',
  },
  chaiConfig: process.env.CI
    ? {
        // show all info on CI
        truncateThreshold: 5000,
      }
    : undefined,
  reporters: ['default', streamedReporter],
  ...(wasmConfig || {}),
});
