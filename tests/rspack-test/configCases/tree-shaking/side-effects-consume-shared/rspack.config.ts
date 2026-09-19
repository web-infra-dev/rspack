import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  optimization: {
    sideEffects: true,
  },
  plugins: [
    new rspack.sharing.ConsumeSharedPlugin({
      consumes: {
        './lib/c.js': {
          singleton: true,
          eager: true,
        },
      },
    }),
  ],
});
