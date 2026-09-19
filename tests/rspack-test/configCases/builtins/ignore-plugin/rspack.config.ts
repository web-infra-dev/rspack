import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
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
});
