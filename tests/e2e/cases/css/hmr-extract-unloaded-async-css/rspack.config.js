const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
    }),
    new rspack.CssExtractRspackPlugin(),
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
