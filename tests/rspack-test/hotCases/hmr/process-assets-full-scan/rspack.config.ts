import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: './index.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: false,
  },
  incremental: {
    chunkAsset: false,
  },
});
