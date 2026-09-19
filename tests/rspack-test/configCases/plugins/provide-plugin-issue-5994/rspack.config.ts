import { defineConfig } from '@rspack/cli';

import { ProvidePlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    new ProvidePlugin({
      Mod: ['./esm', 'default'],
      Def: ['./esm', 'default'],
    }),
  ],
});
