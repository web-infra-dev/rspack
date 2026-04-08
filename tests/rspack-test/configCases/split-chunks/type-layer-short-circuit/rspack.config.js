/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    main: './index',
  },
  target: 'async-node',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        skipped: {
          type: 'css',
          layer() {
            throw new Error('LAYER_SHOULD_NOT_RUN_WHEN_TYPE_IS_FALSE');
          },
        },
      },
    },
  },
};
