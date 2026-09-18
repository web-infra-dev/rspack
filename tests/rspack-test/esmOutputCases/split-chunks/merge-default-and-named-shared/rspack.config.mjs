export default {
  optimization: {
    splitChunks: {
      cacheGroups: {
        a: {
          test: /a\.js$/,
          name: 'a-chunk',
        },
        b: {
          test: /b\.js$/,
          name: 'b-chunk',
        },
      },
    },
  },
};
