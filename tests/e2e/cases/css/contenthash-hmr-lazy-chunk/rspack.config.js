const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  devServer: {
    hot: true,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
    }),
    // A hashed filename is the whole point of this case: without it, the stale
    // `miniCssF` literal a lazy chunk falls back to happens to still be correct
    // (it never changes), so it wouldn't exercise the bug.
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
