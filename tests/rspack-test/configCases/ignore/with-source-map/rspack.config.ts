import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  resolve: {
    alias: {
      './ignored-module': false,
    },
  },
  devtool: 'source-map',
});
