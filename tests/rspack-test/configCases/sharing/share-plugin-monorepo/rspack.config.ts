import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { SharePlugin } = sharing;

export default defineConfig({
  context: `${import.meta.dirname}/app1`,
  plugins: [
    new SharePlugin({
      shared: {
        lib1: {},
        lib2: {
          singleton: true,
        },
      },
    }),
  ],
});
