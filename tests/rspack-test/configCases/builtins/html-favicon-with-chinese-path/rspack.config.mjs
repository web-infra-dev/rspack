import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.HtmlRspackPlugin({
      publicPath: '/',
      favicon: './资源/favicon-图标.ico',
    }),
  ],
};
