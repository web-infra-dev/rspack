import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.sharing.ProvideSharedPlugin({
      provides: ['./a/index.js'],
    }),
  ],
});
