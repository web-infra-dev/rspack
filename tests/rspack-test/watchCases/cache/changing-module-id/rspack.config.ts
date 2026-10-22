import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  cache: true,
  optimization: {
    sideEffects: false,
    providedExports: false,
  },
  module: {
    rules: [
      {
        test: /other-layer/,
        layer: 'other-layer',
      },
    ],
  },
  experiments: {
    // cacheUnaffected: true,
  },
});
