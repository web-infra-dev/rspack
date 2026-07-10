const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: '[name].css',
    }),
  ],
};
