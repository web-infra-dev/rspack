import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const { CssExtractRspackPlugin, HtmlRspackPlugin } = rspack;

export default defineConfig({
  output: {
    publicPath: 'http://cdn.com/',
    crossOriginLoading: 'anonymous',
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
  experiments: {
    css: false,
  },
  plugins: [
    new CssExtractRspackPlugin(),
    new HtmlRspackPlugin({
      minify: false,
      template: './index.html',
      title: 'i am title',
      meta: {
        'meta-name': 'meta-value',
      },
      inject: false,
      favicon: './favicon.ico',
      templateParameters: {
        foo: 'bar',
      },
    }),
  ],
});
