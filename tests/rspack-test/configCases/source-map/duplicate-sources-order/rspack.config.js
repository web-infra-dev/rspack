const { CssExtractRspackPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'web',
  devtool: 'source-map',
  node: false,
  output: {
    devtoolModuleFilenameTemplate: 'module://[resource-path]',
    devtoolFallbackModuleFilenameTemplate:
      'fallback://[all-loaders][resource-path]',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [
          CssExtractRspackPlugin.loader,
          {
            loader: 'css-loader',
            options: {
              sourceMap: true,
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: 'bundle0.css',
    }),
  ],
};
