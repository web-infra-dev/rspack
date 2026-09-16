import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.IgnorePlugin({
      resourceRegExp: /^\.\/b$/,
    }),
    new rspack.IgnorePlugin({
      resourceRegExp: /^\.\/c$/,
      contextRegExp: /moment$/,
    }),
    new rspack.IgnorePlugin({
      resourceRegExp: /^\.\/d$/,
      contextRegExp: /test-ignore$/,
    }),
  ],
};
