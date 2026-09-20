import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  entry: './index.mjs',
  resolve: {
    alias: {
      './ignored-module': false,
    },
  },
  output: {
    iife: false,
  },
});
