module.exports = {
  optimization: {
    splitChunks: {
      cacheGroups: {
        shared: {
          test: /shared\.js$/,
          name: 'shared-chunk',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
};
