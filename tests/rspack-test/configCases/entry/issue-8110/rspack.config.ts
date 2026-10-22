import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    bundle0: './a',
    other: './b',
  },
  output: {
    filename: '[name].js',
  },
});
