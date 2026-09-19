import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [new rspack.HotModuleReplacementPlugin()],
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /stylesheet\.js$/i,
        use: ['./a-pitching-loader.mjs'],
        type: 'asset/source',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
