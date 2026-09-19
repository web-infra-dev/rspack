import { defineConfig } from '@rspack/cli';

import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  mode: 'development',
  target: 'web',
  experiments: {
    css: false,
  },
  output: {
    publicPath: '/',
    chunkLoading: 'import',
    filename: '[name].js',
    chunkFilename: '[id].js',
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
      chunkFilename: '[id].css',
    }),
  ],
});
