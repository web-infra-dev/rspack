import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    output: {
      library: { type: 'commonjs2' },
    },
    externals: {
      external: ['@rspack/core', 'version'],
    },
    plugins: [
      new rspack.DefinePlugin({
        EXPECTED: JSON.stringify(rspack.version),
      }),
    ],
  },
  {
    externals: {
      external: ['Array', 'isArray'],
    },
    plugins: [
      new rspack.DefinePlugin({
        EXPECTED: 'Array.isArray',
      }),
    ],
  },
];
