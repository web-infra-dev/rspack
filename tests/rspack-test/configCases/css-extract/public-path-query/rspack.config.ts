import { defineConfig } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  target: 'web',
  output: {
    publicPath: 'https://test.cases/path/asset?file=',
    filename: '[name].js',
    chunkFilename: '[name].js',
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
      chunkFilename: '[name].css',
    }),
  ],
});
