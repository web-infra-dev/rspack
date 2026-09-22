import { CssExtractRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    a: './a.js',
    b: { import: './b.js', dependOn: 'a' },
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        loader: CssExtractRspackPlugin.loader,
      },
    ],
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        styles: {
          type: 'css/mini-extract',
          enforce: true,
        },
      },
    },
  },

  target: 'web',
  plugins: [
    new CssExtractRspackPlugin({
      experimentalUseImportModule: true,
    }),
  ],
  experiments: {
    css: false,
  },
};
