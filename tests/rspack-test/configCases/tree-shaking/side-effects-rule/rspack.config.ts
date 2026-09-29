import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /package/,
        sideEffects: false,
      },
    ],
  },

  optimization: {
    sideEffects: true,
  },
  externalsPresets: {
    node: true,
  },
});
