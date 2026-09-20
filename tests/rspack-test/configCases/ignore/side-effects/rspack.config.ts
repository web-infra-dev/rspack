import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: './test.js',
  resolve: {
    alias: {
      'ignored-module': false,
      './ignored-module': false,
    },
  },
  plugins: [new rspack.IgnorePlugin({ resourceRegExp: /(b\.js|b)$/ })],
  optimization: {
    sideEffects: true,
  },
});
