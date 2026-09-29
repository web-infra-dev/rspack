import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
});
