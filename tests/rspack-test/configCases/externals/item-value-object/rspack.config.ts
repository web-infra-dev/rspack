import { defineConfig } from '@rspack/cli';

import { CopyRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: './index.js',
  output: {
    library: {
      type: 'commonjs',
    },
  },
  externals: {
    lodash: {
      commonjs: './lodash.js',
    },
  },
  plugins: [
    new CopyRspackPlugin({
      patterns: ['./lodash.js'],
    }),
  ],
});
