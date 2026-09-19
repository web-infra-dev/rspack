import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.DefinePlugin({
      'typeof window': JSON.stringify('undefined'),
    }),
  ],
});
