import { defineConfig } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
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
  plugins: [new CssExtractRspackPlugin()],
  experiments: {
    css: false,
  },
});
