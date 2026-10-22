import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /side-effect\.js/,
        sideEffects: false,
      },
    ],
  },
  optimization: {
    sideEffects: false,
  },
});
