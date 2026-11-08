import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'production',
  plugins: [
    new rspack.DefinePlugin({
      FALSY: JSON.stringify(false),
    }),
  ],
});
