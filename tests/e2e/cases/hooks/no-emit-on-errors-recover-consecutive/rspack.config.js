const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './src/index.js',
  context: __dirname,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  optimization: {
    emitOnErrors: false,
  },
  devServer: {
    hot: true,
  },
};
