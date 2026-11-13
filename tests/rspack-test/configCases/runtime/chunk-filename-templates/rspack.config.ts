import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    chunkFilename: '[id].[contenthash].js',
    cssChunkFilename: '[id].[contenthash].css',
  },
  optimization: { minimize: false, chunkIds: 'named' },
  module: {
    rules: [{ test: /\.css$/, type: 'css/auto' }],
  },
});
