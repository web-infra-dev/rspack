import { defineConfig } from '@rspack/cli';
import { type Configuration, rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

const baseConfig = (i: number, mjs = false): Configuration => ({
  entry: {
    index: {
      import: './index.js',
      filename: `bundle${i}${mjs ? '.mjs' : '.js'}`,
    },
  },
  target: 'node',
  node: {
    __filename: false,
    __dirname: false,
  },
});

export default defineConfig([
  // CJS output
  {
    ...baseConfig(0),
    output: {
      library: {
        type: 'commonjs',
      },
    },
    plugins: [new RslibPlugin()],
  },
  // ESM output
  {
    ...baseConfig(1, true),
    externals: {
      os: 'module os',
    },
    output: {
      module: true,
      library: {
        type: 'modern-module',
      },
    },
    plugins: [new RslibPlugin()],
  },
  // Test entry
  {
    entry: './test.js',
    target: 'node',
    node: {
      __filename: false,
      __dirname: false,
    },
  },
]);
