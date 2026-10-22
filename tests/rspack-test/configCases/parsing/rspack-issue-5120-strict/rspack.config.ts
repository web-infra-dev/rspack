import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
    fail: './fail.js',
  },
  output: {
    filename: '[name].js',
  },
});
