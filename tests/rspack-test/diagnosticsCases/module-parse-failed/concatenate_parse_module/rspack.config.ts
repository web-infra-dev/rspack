import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.DefinePlugin({
      DEFINE_VAR: '1 2 3',
    }),
  ],
  optimization: {
    concatenateModules: true,
  },
});
