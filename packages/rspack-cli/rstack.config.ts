import { fileURLToPath } from 'node:url';
import { define } from 'rstack';
import type { RstestConfig } from 'rstack/test';
import packageJson from './package.json' with { type: 'json' };

define.lib({
  lib: [
    {
      format: 'esm',
      syntax: ['es2023'],
      dts: {
        bundle: true,
        tsgo: true,
        typescriptPath: fileURLToPath(
          import.meta.resolve('@typescript/native'),
        ),
      },
    },
  ],
  output: {
    externals: [
      ({ request }, callback) => {
        if (request === 'jiti') {
          return callback(undefined, '../compiled/jiti/index.js');
        }
        return callback();
      },
    ],
  },
  source: {
    tsconfigPath: './tsconfig.build.json',
    define: {
      RSPACK_CLI_VERSION: JSON.stringify(packageJson.version),
    },
  },
});

define.test(() => {
  const wasmConfig: RstestConfig | undefined = process.env.WASM
    ? {
        exclude: [
          '**/*/profile.test.ts', // Skip due to lack of system api support
        ],
      }
    : undefined;

  return {
    // Keep tests independent from the Rslib build configuration.
    extends: {},
    name: 'rspack-cli',
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
    ...(wasmConfig || {}),
  };
});
