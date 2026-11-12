import { defineConfig } from '@rspack/cli';
import { HotModuleReplacementPlugin } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  mode: 'development',
  plugins: [new HotModuleReplacementPlugin()],
  module: {
    rules: [
      {
        loader: './loader.mjs',
      },
    ],
  },
});
