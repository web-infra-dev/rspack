import { defineConfig, type RstestConfig } from '@rstest/core';

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
  },
  env: {
    RUST_BACKTRACE: 'full',
  },
  ...(wasmConfig || {}),
});
