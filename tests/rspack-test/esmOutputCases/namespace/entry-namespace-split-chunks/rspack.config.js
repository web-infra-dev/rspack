module.exports = {
  optimization: {
    splitChunks: {
      cacheGroups: {
        beta: {
          test: /(?:beta-target|alpha-shared)\.js$/,
          name: 'beta',
          chunks: 'all',
          minSize: 0,
          priority: 30,
        },
        moved: {
          test: /moved-target\.js$/,
          name: 'moved',
          chunks: 'all',
          minSize: 0,
          priority: 20,
        },
        merged: {
          test: /(?:merged-target|unrelated)\.js$/,
          name: 'merged',
          chunks: 'all',
          minSize: 0,
          priority: 10,
        },
        leaky: {
          test: /(?:leaky-target|leaky-shared)\.js$/,
          name: 'leaky',
          chunks: 'all',
          minSize: 0,
          priority: 5,
        },
      },
    },
  },
};
