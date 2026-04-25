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
          minChunks: 2,
          test() {
            throw new Error(
              'TEST_SHOULD_NOT_RUN_WHEN_MIN_CHUNKS_IS_IMPOSSIBLE',
            );
          },
        },
      },
    },
  },
};
