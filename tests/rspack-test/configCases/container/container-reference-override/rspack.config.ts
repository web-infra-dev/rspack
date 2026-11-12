import { defineConfig } from '@rspack/cli';
import { container, sharing } from '@rspack/core';

const { ContainerReferencePlugin } = container;
const { ProvideSharedPlugin } = sharing;

export default defineConfig({
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
});
