import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
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
});
