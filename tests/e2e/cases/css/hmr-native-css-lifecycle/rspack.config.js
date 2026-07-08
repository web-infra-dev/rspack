const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  output: {
    cssFilename: '[name].css',
    cssChunkFilename: '[name].css',
  },
  devServer: {
    hot: true,
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
        type: 'css',
      },
    ],
  },
  watchOptions: {
    poll: 1000,
  },
  experiments: {
    css: true,
  },
};
