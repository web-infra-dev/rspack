export default {
  optimization: {
    splitChunks: {
      cacheGroups: {
        ab: {
          test: /[ab]\.js$/,
          name: 'ab-chunk',
          chunks: 'all',
        },
      },
    },
  },
};
