/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  entry: { main: './index.js', other: './other.js' },
  output: { filename: '[name].js' },
  optimization: {
    splitChunks: {
      cacheGroups: {
        hooks: {
          test: /hooks\.js$/,
          name: 'hooks',
          chunks: 'all',
          enforce: true,
        },
        shared: {
          test: /[\\/]shared[\\/]/,
          name: 'shared',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
};
