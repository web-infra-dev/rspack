const path = require('path');

/**
 * @param {string} name name
 * @param {string} devtool devtool
 * @returns {import("@rspack/core").Configuration} configuration
 */
const base = (name, devtool) => ({
  mode: 'production',
  devtool,
  module: {
    rules: [
      {
        test: /chunk/,
        loader: 'babel-loader',
        options: {},
      },
    ],
  },
  stats: {
    assets: true,
    modules: true,
    relatedAssets: true,
  },
  entry: {
    main: {
      import: './index',
      layer: 'my-layer',
    },
  },
  context: path.resolve(__dirname, name),
  output: {
    filename: '[name]-[chunkhash].js',
    // Keep the emitted/cached asset status deterministic: all six child
    // compilations share one dist directory and the a/b pairs produce
    // identical files, so content comparison would race between "[emitted]"
    // and "[cached]".
    compareBeforeEmit: false,
  },
  resolve: {
    alias: {
      c: [
        path.resolve(__dirname, name, 'c'),
        path.resolve(__dirname, name, 'cc'),
      ],
    },
  },
});

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  base('a', 'source-map'),
  base('b', 'source-map'),
  base('a', 'eval-source-map'),
  base('b', 'eval-source-map'),
  base('a', 'eval'),
  base('b', 'eval'),
];
