import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    sideEffects: true,
    usedExports: false,
    innerGraph: true,
  },
  module: {
    rules: [
      {
        test: /re-exports\.js$/,
        sideEffects: false,
      },
    ],
  },
});
