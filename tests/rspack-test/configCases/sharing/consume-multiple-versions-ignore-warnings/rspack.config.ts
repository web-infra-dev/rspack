import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

// eslint-disable-next-line node/no-unpublished-require
const { ConsumeSharedPlugin } = sharing;

export default defineConfig({
  output: {
    // @ts-expect-error Rspack does not support ignoreBrowserWarnings yet.
    ignoreBrowserWarnings: true,
  },
  plugins: [
    new ConsumeSharedPlugin({
      consumes: {
        shared: {
          import: false,
          strictVersion: true,
        },
        shared2: {
          import: false,
        },
      },
    }),
  ],
});
