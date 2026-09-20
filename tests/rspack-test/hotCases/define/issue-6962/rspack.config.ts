import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.DefinePlugin({
      DEFINE_PATH: JSON.stringify('./a'),
    }),
  ],
});
