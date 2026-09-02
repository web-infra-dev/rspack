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
    // TODO webpack 6 remove, `css-url` is the old spelling of `asset-url`
    'css-css-url': 'css-url https://example.test/css-css-url.png',
    'js-asset': 'asset https://example.test/js-asset.png',
    'js-asset-url': 'asset-url https://example.test/js-asset-url.png',
    'js-css-url': 'css-url https://example.test/js-css-url.png',
  },
};
