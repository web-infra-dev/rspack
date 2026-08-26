/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  entry: {
    a: './a',
    b: './b',
  },
  output: {
    filename: 'c-[name].js',
    library: { type: 'commonjs2' },
    // TODO: Both webpack and Rspack fail to define __webpack_require__.h when
    // [hash] is used only in a splitChunks cache-group filename. Keep [hash]
    // here as a workaround.
    chunkFilename: '[hash].js',
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      cacheGroups: {
        shared: {
          chunks: 'all',
          test: /shared/,
          filename: 'shared-[name]-[hash:6].js',
          enforce: true,
        },
        common: {
          chunks: 'all',
          filename: 'common-[name]-[fullhash:4].js',
          test: /common/,
          enforce: true,
        },
        other: {
          chunks: 'all',
          test: /other/,
          enforce: true,
        },
      },
    },
  },
};
