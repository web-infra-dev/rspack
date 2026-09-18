import { CopyRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  output: {
    library: { type: 'umd' },
  },
  externals: {
    lodash: {
      root: './lodash.js',
      commonjs: './lodash.js',
      commonjs2: './lodash.js',
    },
  },
  plugins: [
    new CopyRspackPlugin({
      patterns: ['./lodash.js'],
    }),
  ],
};
