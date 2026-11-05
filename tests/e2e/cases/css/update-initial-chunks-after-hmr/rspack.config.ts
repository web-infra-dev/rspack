import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: ['./src/index.css', './src/index.js'],
  },
  devServer: {
    hot: true,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
    }),
    new rspack.CssExtractRspackPlugin({
      filename: 'static/style.css',
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        style: {
          name: 'style',
          test: /\.css$/,
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
  watchOptions: {
    poll: 1000,
  },
  experiments: {
    css: false,
  },
});
