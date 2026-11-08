import { defineConfig } from '@rspack/cli';
import { HtmlRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    index: {
      import: ['./index.js'],
    },
  },
  plugins: [
    new HtmlRspackPlugin({
      filename: 'output.html',
      template: 'input.html',
      inject: 'head',
      scriptLoading: 'blocking',
    }),
  ],
});
