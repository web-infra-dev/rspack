const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    'node:fs': 'module node:path',
    'node:url': 'module-import node:url',
  },
  plugins: [
    new RslibPlugin({
      autoCjsNodeBuiltin: true,
    }),
  ],
};
