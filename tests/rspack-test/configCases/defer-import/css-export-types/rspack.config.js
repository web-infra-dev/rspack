'use strict';

/** @type {import("../../../../").Configuration} */
module.exports = {
  target: 'web',
  mode: 'development',
  devtool: false,
  optimization: {
    concatenateModules: false,
  },
  module: {
    rules: [
      {
        test: /style-text\.css/,
        type: 'css/auto',
        parser: { exportType: 'text' },
      },
      {
        test: /style-stylesheet\.css/,
        type: 'css/auto',
        parser: { exportType: 'css-style-sheet' },
      },
      {
        test: /style-attr\.css/,
        type: 'css/auto',
        parser: { exportType: 'css-style-sheet' },
      },
    ],
  },
  experiments: {
    deferImport: true,
    css: true,
  },
};
