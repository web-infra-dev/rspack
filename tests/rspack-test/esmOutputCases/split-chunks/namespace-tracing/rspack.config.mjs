export default {
  optimization: {
    splitChunks: {
      cacheGroups: {
        broken: {
          test: /broken\.js$/,
          name: 'broken-chunk',
          chunks: 'all',
        },
      },
    },
  },
};
