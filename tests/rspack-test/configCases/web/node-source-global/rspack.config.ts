import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  node: {
    global: true,
  },
  plugins: [
    new DefinePlugin({
      'global.test': "'test'",
    }),
  ],
});
