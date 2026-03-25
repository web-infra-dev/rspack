module.exports = {
  optimization: {
    splitChunks: {
      cacheGroups: {
        broken: {
          test: /broken\.js$/,
          name: "broken-chunk",
          chunks: "all",
        },
        other: {
          test: /other\.js$/,
          name: "other-chunk",
          chunks: "all",
        },
      },
    },
  },
};
