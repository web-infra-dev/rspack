import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin, EsmLibraryPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
const baseConfig = (i, mjs = false) => ({
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

export default [
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
];
