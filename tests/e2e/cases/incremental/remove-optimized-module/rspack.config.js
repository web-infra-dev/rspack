import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  entry: './index.js',
  cache: true,
  experiments: {
    cache: true,
  },
  plugins: [new rspack.HtmlRspackPlugin()],
};
