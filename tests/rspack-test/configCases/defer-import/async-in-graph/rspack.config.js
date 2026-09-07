'use strict';

const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: [`async-node${process.versions.node.split('.').map(Number)[0]}`],
  mode: 'none',
  experiments: {
    deferImport: true,
  },
  plugins: [
    new rspack.ProvidePlugin({
      providedAsyncValue: ['./async.js', 'value'],
    }),
  ],
};
