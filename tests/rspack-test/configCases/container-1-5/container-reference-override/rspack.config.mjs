import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ModuleFederationPlugin({
      remoteType: 'var',
      remotes: {
        abc: 'ABC',
      },
      shared: {
        './new-test': {
          shareKey: 'test',
          version: false,
        },
      },
    }),
  ],
};
