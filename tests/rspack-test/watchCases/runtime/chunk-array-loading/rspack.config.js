module.exports = {
  mode: 'production',
  target: 'node',
  experiments: { chunkArrayLoading: true },
  optimization: {
    minimize: false,
    concatenateModules: true,
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        dep: { test: /dep\.js$/, name: 'dep', chunks: 'async', enforce: true },
      },
    },
  },
};
