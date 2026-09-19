import { defineConfig } from '@rspack/cli';

import { IgnorePlugin } from '@rspack/core';

export default defineConfig({
  entry: './test.js',
  externals: {
    './normal-module': '{}',
  },
  plugins: [
    new IgnorePlugin({
      resourceRegExp: /ignored-module1/,
    }),
    new IgnorePlugin({
      resourceRegExp: /ignored-module2/,
    }),
  ],
});
