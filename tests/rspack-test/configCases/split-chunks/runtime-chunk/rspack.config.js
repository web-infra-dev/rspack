const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './dep-shared_js.js': 'commonjs ./dep-shared_js.js',
  },
  entry: {
    a: './a',
    b: './b',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        dep: {
          chunks: 'all',
          test: path.resolve(__dirname, 'shared.js'),
          enforce: true,
        },
      },
    },
  },
};
