import { defineConfig } from '@rspack/cli';

import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  entry: './index.js',
  output: {
    filename: '[name].js',
  },
  plugins: [new webpack.optimize.LimitChunkCountPlugin({ maxChunks: 1 })],
});
