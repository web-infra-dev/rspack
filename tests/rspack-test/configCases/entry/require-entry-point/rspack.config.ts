import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    bundle0: './require-entry-point',
    a: './entry-point',
    b: ['./entry-point2'],
  },
  output: {
    filename: '[name].js',
  },
});
