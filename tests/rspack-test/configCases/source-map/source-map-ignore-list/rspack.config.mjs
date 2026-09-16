import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: false,
  plugins: [
    new rspack.SourceMapDevToolPlugin({
      filename: '[file].map',
      ignoreList: [/ignored\.js/],
    }),
  ],
};
