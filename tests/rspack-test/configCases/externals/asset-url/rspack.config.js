'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  experiments: {
    css: true,
  },
  optimization: {
    minimize: false,
  },
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
    'css-asset': 'asset https://example.test/css-asset.png',
    'css-asset-url': 'asset-url https://example.test/css-asset-url.png',
    'js-asset': 'asset https://example.test/js-asset.png',
    'js-asset-url': 'asset-url https://example.test/js-asset-url.png',
  },
};
