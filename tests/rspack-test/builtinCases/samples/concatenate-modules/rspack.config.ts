import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  optimization: {
    concatenateModules: true,
    sideEffects: true,
    providedExports: true,
    usedExports: 'global',
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
});
