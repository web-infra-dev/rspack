import { sharing } from '@rspack/core';

const { SharePlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  context: `${import.meta.dirname}/cjs`,
  plugins: [
    new SharePlugin({
      shared: {
        lib: {},
        transitive_lib: {},
      },
    }),
  ],
};
