import { CopyRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
