import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    e1: './e1.js',
    e2: {
      import: './e2.js',
      runtime: false,
    },
  },
  mode: 'development',
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
});
