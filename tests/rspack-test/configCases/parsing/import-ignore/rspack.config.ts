import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    bundle0: './index.js',
    bundle1: './other.js',
  },
  output: {
    filename: '[name].js',
  },
  node: {
    __dirname: false,
  },
});
