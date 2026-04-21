const path = require('path');
const { CssExtractRspackPlugin } = require('@rspack/core');

/**@type {import('@rspack/core').Configuration} */
module.exports = {
  entry: './src/index.js',
  experiments: {
    css: false,
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
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
    }),
  ],
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(__dirname, 'src'),
    },
  },
};
