import { defineConfig } from '@rspack/cli';

import { IgnorePlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    a: './intentionally-missing-module.js',
    b: ['./intentionally-missing-module.js'],
    bundle0: ['./index'],
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new IgnorePlugin({
      resourceRegExp: new RegExp(/intentionally-missing-module/),
    }),
  ],
});
