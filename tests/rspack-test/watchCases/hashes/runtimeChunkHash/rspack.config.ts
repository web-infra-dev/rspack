import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: './a/index.js',
    b: './b/index.js',
    main: './main/index.js',
  },
  output: {
    clean: false,
    filename: '[name].[chunkhash].[contenthash].js',
    chunkFilename: '[name].[chunkhash].[contenthash].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
});
