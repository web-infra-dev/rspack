import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  externals: {
    path: "require('path')",
    fs: "require('fs')",
  },
  node: {
    __dirname: false,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      filename: 'main_page/index.html',
    }),
  ],
};
