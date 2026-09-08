const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: ['./src/index.css', './src/index.js'],
  },
  devServer: {
    hot: true,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
    }),
    // A hashed filename is the whole point of this case: the runtime's cached
    // `miniCssF` literal goes stale on every edit unless the fresh href is
    // threaded through the hot update instead.
    new rspack.CssExtractRspackPlugin({
      filename: '[name].[contenthash:8].css',
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
};
