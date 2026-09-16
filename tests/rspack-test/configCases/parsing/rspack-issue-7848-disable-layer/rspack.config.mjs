import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    'node:fs': 'node-commonjs node:fs',
    'node:path': 'node-commonjs node:path',
  },
  entry: {
    bundle0: './index.js',
  },
  plugins: [
    new rspack.DefinePlugin({
      __RUNTIME_TYPE__: '__webpack_layer__',
    }),
  ],
};
