import { container } from '@rspack/core';

const { ContainerReferencePlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ContainerReferencePlugin({
      remoteType: 'var',
      remotes: {
        abc: 'ABC',
        def: 'DEF',
      },
    }),
  ],
};
