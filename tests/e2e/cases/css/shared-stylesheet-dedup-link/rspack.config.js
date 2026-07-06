const { rspack } = require('@rspack/core');

/** @type {import('@rspack/core').RspackOptions} */
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: ['./src/index.css', './src/index.js'],
  },
  output: {
    cssFilename: 'static/style.css',
  },
  devServer: {
    hot: true,
  },
  experiments: {
    css: true,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        parser: {
          exportType: 'link',
        },
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
