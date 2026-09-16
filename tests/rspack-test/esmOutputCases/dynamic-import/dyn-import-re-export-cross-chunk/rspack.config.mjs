export default {
  optimization: {
    splitChunks: {
      cacheGroups: {
        shared: {
          test: /shared\.js$/,
          name: 'shared-chunk',
        },
      },
    },
  },
};
