import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    publicPath: 'auto',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      favicon: 'favicon.ico',
    }),
  ],
};
