import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    global: true,
  },
  plugins: [
    new DefinePlugin({
      'global.test': "'test'",
    }),
  ],
};
