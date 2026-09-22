import { sharing } from '@rspack/core';

const { SharePlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
