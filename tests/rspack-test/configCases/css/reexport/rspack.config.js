'use strict';

/** @type {import("@rspack/core").Configuration[]} */
module.exports = ['development', 'production'].map((mode, idx) => ({
  name: mode,
  devtool: false,
  entry: './index.js',
  mode,
  target: 'web',
  output: {
    filename: `bundle${idx}.js`,
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  optimization: {
    minimize: false,
  },
  experiments: {
    css: true,
  },
}));
