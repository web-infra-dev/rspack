import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    cssChunkFilename: () => '[id].[contenthash].css',
  },
  optimization: {
    minimize: false,
    chunkIds: 'named',
  },
  module: {
    rules: [{ test: /\.css$/, type: 'css/auto' }],
  },
});
