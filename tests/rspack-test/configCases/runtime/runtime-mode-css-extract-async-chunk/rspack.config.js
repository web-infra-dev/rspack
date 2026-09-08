const { CssExtractRspackPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  experiments: {
    runtimeMode: 'rspack',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [new CssExtractRspackPlugin()],
};
