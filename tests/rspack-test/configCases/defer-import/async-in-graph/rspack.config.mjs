import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
