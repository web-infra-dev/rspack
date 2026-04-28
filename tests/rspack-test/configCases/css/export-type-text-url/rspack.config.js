'use strict';

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    name: 'js-at-root',
    devtool: false,
    target: 'web',
    mode: 'development',
    output: {
      filename: 'bundle0.js',
      cssFilename: 'css/[name].css',
      cssChunkFilename: 'css/[name].css',
      publicPath: 'auto',
      assetModuleFilename: 'images/[name][ext]',
    },
    module: {
      rules: [
        {
          test: /style\.css$/,
          type: 'css/auto',
          parser: { exportType: 'text' },
        },
        {
          test: /style-for-sheet\.css$/,
          type: 'css/auto',
          parser: { exportType: 'css-style-sheet' },
        },
        {
          test: /style-for-inject\.css$/,
          type: 'css/auto',
          parser: { exportType: 'style' },
        },
      ],
    },
  },
  {
    name: 'js-in-subdir',
    devtool: false,
    target: 'web',
    mode: 'development',
    output: {
      filename: 'js/bundle1.js',
      cssFilename: 'css/[name].css',
      cssChunkFilename: 'css/[name].css',
      publicPath: 'auto',
      assetModuleFilename: 'images/[name][ext]',
    },
    module: {
      rules: [
        {
          test: /style\.css$/,
          type: 'css/auto',
          parser: { exportType: 'text' },
        },
        {
          test: /style-for-sheet\.css$/,
          type: 'css/auto',
          parser: { exportType: 'css-style-sheet' },
        },
        {
          test: /style-for-inject\.css$/,
          type: 'css/auto',
          parser: { exportType: 'style' },
        },
      ],
    },
  },
];
