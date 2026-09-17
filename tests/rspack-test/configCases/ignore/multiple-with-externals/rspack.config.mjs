import { IgnorePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
