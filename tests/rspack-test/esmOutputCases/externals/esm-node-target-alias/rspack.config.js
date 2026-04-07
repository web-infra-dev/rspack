const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    'node:fs': 'node:path',
  },
  plugins: [
    new RslibPlugin({
      externalEsmNodeBuiltin: true,
    }),
  ],
};
