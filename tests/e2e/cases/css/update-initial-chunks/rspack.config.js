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
    new rspack.CssExtractRspackPlugin({
      filename: 'static/style.css',
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        style: {
          name: 'style',
          test: /\.css$/,
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
  watchOptions: {
    poll: 1000,
  },
};
