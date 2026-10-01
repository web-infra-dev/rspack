import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
    chunkFilename: 'main.[contenthash:8].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
});
