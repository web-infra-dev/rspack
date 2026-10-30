import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'web',
  devtool: false,
  entry: {
    index: './index.js',
    other: './other.js',
  },
  output: {
    filename: '[name].js',
    cssFilename: '[name].[contenthash].css',
    cssChunkFilename: '[name].[contenthash].chunk.css',
    publicPath: 'https://test.cases/path/',
    chunkLoadingGlobal: 'issue16349Chunks',
  },
  module: {
    rules: [{ test: /\.css$/, type: 'css' }],
  },
  optimization: {
    minimize: false,
    runtimeChunk: true,
    splitChunks: { chunks: 'all', minSize: 0 },
  },
});
