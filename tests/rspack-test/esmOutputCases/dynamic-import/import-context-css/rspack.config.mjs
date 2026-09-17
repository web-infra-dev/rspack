import { CssExtractRspackPlugin } from '@rspack/core';
export default {
  target: 'web',
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
      chunkFilename: '[name].css',
    }),
  ],
};
