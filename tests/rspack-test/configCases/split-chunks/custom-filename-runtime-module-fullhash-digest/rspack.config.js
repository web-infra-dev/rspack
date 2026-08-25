/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  output: {
    filename: 'main.js',
    chunkFilename: 'fallback-[id].js',
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        encoded: {
          chunks: 'async',
          test: /async\.js$/,
          name: 'encoded',
          filename: 'split-[name]-[fullhash:base64:4].js',
          enforce: true,
        },
      },
    },
  },
};
