import { container, sharing } from '@rspack/core';

const { ContainerReferencePlugin } = container;
const { ProvideSharedPlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ContainerReferencePlugin({
      remoteType: 'var',
      remotes: {
        abc: 'ABC',
      },
    }),
    new ProvideSharedPlugin({
      provides: {
        './new-test': {
          shareKey: 'test',
          version: false,
        },
      },
    }),
  ],
};
