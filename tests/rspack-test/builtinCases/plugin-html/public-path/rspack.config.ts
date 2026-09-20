import { defineConfig } from '@rspack/cli';
import { HtmlRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    index: {
      import: ['./index.js'],
    },
  },
  output: {
    publicPath: '/base',
  },
  plugins: [
    new HtmlRspackPlugin({
      favicon: 'favicon.ico',
    }),
  ],
});
