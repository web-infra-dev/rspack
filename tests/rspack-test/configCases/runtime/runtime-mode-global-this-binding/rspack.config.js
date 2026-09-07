const { BannerPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
};
