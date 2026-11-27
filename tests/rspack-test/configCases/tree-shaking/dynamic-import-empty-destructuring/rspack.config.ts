import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  experiments: {
    sourceImport: true,
  },
  module: {
    rules: [
      { test: /(?:declared-side-effects-free|eager)\.js$/, sideEffects: false },
      { test: /context[\\/]effectful\.js$/, sideEffects: false },
    ],
  },
  optimization: {
    minimize: true,
    usedExports: true,
    sideEffects: true,
  },
});
