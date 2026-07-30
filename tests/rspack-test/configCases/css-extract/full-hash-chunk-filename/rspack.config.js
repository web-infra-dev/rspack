const { CssExtractRspackPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  mode: 'development',
  target: 'web',
  devtool: false,
  output: {
    filename: 'main.js',
    chunkFilename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  optimization: {
    minimize: false,
    splitChunks: false,
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
      chunkFilename: '[name].[fullhash].css',
    }),
  ],
};
