/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      chunks: 'all',
      minSize: 1,
      // maxInitialRequests would normally block splitting, but
      // enforceSizeThreshold overrides it when module group size exceeds the threshold
      maxInitialRequests: 1,
      enforceSizeThreshold: 10,
      cacheGroups: {
        vendors: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          priority: 10,
        },
      },
    },
  },
};
