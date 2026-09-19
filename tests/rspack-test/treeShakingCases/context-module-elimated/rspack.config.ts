import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  optimization: {
    sideEffects: true,
    // Avoid inlineExports causes resolve error and failed to create the context module
    inlineExports: false,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
});
