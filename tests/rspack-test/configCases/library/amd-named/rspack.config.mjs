import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'amd', name: 'NamedLibrary' },
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner: 'function define(name, deps, fn) { fn(); }\n',
    }),
  ],
};
