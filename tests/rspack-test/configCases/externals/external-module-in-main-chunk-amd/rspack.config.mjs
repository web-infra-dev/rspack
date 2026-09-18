import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'amd' },
  },
  externals: {
    external: 'external',
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner: 'function define(deps, fn) { fn(); }\n',
    }),
  ],
};
