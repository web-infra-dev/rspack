import { defineConfig } from '@rspack/cli';

import { container } from '@rspack/core';

const { ContainerReferencePlugin } = container;

export default defineConfig({
  plugins: [
    new ContainerReferencePlugin({
      remoteType: 'var',
      remotes: {
        abc: 'ABC',
        def: 'DEF',
      },
    }),
  ],
});
