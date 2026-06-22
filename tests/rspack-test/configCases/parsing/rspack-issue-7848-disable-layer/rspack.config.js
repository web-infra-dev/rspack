const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
