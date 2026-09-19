import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig([
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
]);
