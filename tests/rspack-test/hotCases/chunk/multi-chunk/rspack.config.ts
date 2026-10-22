import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: './a/index.js',
    b: './b/index.js',
    main: './main/index.js',
  },
  output: {
    filename: '[name].js',
  },
});
