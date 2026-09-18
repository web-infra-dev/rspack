import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'amd' },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner: 'function define(fn) { fn(); }\n',
    }),
  ],
};
