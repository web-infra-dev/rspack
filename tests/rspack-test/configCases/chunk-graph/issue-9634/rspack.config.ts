import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    b: './entry-b',
    a: './entry-a',
  },
  output: {
    filename: '[name].js',
  },
});
