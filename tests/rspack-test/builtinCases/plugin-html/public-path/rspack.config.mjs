import { HtmlRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
