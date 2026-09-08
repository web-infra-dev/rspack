module.exports = {
  mode: 'production',
  target: 'webworker',
  experiments: { chunkArrayLoading: true },
  optimization: {
    minimize: false,
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        dep: { test: /dep\.js$/, name: 'dep', chunks: 'async', enforce: true },
      },
    },
  },
};
