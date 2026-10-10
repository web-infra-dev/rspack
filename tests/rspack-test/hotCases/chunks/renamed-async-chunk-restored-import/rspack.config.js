/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  optimization: {
    splitChunks: {
      cacheGroups: {
        hooks: {
          test: /hooks\.js$/,
          name: 'hooks',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
};
