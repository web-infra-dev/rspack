import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  optimization: {
    sideEffects: false,
  },
  module: {
    parser: {
      javascript: {
        exportsPresence: 'auto',
      },
    },
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
});
