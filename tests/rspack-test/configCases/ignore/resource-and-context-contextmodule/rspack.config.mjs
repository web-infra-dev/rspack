import { IgnorePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './test.js',
  plugins: [
    new IgnorePlugin({
      resourceRegExp: /ignored-module/,
      contextRegExp: /folder-b/,
    }),
  ],
};
