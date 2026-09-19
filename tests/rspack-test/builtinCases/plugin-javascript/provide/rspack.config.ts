import { defineConfig } from '@rspack/cli';

import { ProvidePlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    new ProvidePlugin({
      process: ['./process.js'],
      name: ['./name.js'],
    }),
  ],
});
