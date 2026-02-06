import path from 'node:path';
import { defineConfig, defineProject, type ProjectConfig } from '@rstest/core';

const root = path.resolve(__dirname, "../../");

process.env.NO_COLOR = '1';

const setupFilesAfterEnv = [
  "@rspack/test-tools/setup-env",
  "@rspack/test-tools/setup-expect",
  "./expects/stats-string-comparator.js",
];

const wasmConfig = process.env.WASM && defineProject({
  setupFiles: [...setupFilesAfterEnv, "@rspack/test-tools/setup-wasm"],
  exclude: [
    // Skip because they rely on snapshots
    "Diagnostics.test.js",
    "Error.test.js",
    "StatsAPI.test.js",
    "StatsOutput.test.js",
    // Skip because the loader can not be loaded in CI
    "Hot*.test.js",

    // Skip temporarily and should investigate in the future
    "Cache.test.js",
    "Compiler.test.js",
    "MultiCompiler.test.js",
    "Serial.test.js",
    "Defaults.test.js",
    "Example.test.js",
    "Incremental-*.test.js",
    "NativeWatcher*.test.js",
  ],
  maxConcurrency: 1,
});

const testFilter = process.argv.includes("--test") || process.argv.includes("-t")
  ? process.argv[
  (process.argv.includes("-t")
    ? process.argv.indexOf("-t")
    : process.argv.indexOf("--test")) + 1
  ]
  : undefined;

const sharedConfig = defineProject({
  setupFiles: setupFilesAfterEnv,
  testTimeout: process.env.CI ? 60000 : 30000,
  include: [
    "*.test.js",
  ],
  slowTestThreshold: 5000,
  // Retry on CI to reduce flakes
  retry: process.env.CI ? 3 : 0,
  resolve: {
    alias: {
      // Fixed jest-serialize-path not working when non-ascii code contains.
      slash: path.join(__dirname, "../../scripts/test/slash.cjs"),
      // disable sourcemap remapping for ts file
      "source-map-support/register": "identity-obj-proxy"
    }
  },
  source: {
    exclude: [root],
  },
  disableConsoleIntercept: true,
  globals: true,
  output: {
    externals: [/.*/],
  },
  passWithNoTests: true,
  hideSkippedTests: true,
  hideSkippedTestFiles: true,
  snapshotFormat: {
    escapeString: true,
    printBasicPrototype: true
  },
  chaiConfig: process.env.CI ? {
    // show all info on CI
    truncateThreshold: 5000,
  } : undefined,
  env: {
    RUST_BACKTRACE: 'full',
    updateSnapshot:
      process.argv.includes("-u") || process.argv.includes("--updateSnapshot") ? 'true' : 'false',
    RSPACK_DEV: 'false',
    RSPACK_EXPERIMENTAL: 'true',
    RSPACK_CONFIG_VALIDATE: "strict",
    testFilter,
    printLogger: process.env.DEBUG === "test" ? 'true' : 'false',
    __TEST_PATH__: __dirname,
    __TEST_FIXTURES_PATH__: path.resolve(__dirname, "fixtures"),
    __TEST_DIST_PATH__: path.resolve(__dirname, "js"),
    __ROOT_PATH__: root,
    DEFAULT_MAX_CONCURRENT: process.argv.includes("--maxConcurrency")
      ? process.argv[
      process.argv.indexOf("--maxConcurrency") + 1
      ]
      : undefined,
    __RSPACK_PATH__: path.resolve(root, "packages/rspack"),
    __RSPACK_TEST_TOOLS_PATH__: path.resolve(root, "packages/rspack-test-tools"),
    __DEBUG__: process.env.DEBUG === "test" ? 'true' : 'false',
  },
  ...(wasmConfig || {}),
}) as ProjectConfig;

export default defineConfig({
  projects: [{
    extends: sharedConfig,
    name: 'base',
  }, {
    extends: sharedConfig,
    name: 'hottest',
    include: process.env.WASM ? [] : ["<rootDir>/*.hottest.js"],
    env: {
      RSPACK_HOT_TEST: 'true',
    },
  }],
  reporters: testFilter ? ['verbose'] : ['default'],
  pool: {
    maxWorkers: process.env.WASM ? 1 : "80%",
    execArgv: ['--no-warnings', '--expose-gc', '--max-old-space-size=8192', '--experimental-vm-modules'],
  },
});

