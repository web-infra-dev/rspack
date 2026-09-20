import { defineConfig } from '@rspack/cli';
import { CopyRspackPlugin } from '@rspack/core';

export default defineConfig({
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
});
