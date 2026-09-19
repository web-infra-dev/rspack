import { defineConfig } from '@rspack/cli';

import { BannerPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  target: 'webworker',
  experiments: {
    runtimeMode: 'rspack',
  },
  module: {
    rules: [
      {
        test: /\.txt$/,
        type: 'asset/resource',
      },
    ],
  },
  output: {
    environment: {
      arrowFunction: true,
      globalThis: false,
    },
    publicPath: 'auto',
  },
  optimization: {
    concatenateModules: false,
    minimize: false,
  },
  plugins: [
    new BannerPlugin({
      banner: '"use strict";',
      raw: true,
    }),
  ],
});
