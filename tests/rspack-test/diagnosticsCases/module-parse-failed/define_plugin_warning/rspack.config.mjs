import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.DefinePlugin({
      __DEV__: '😄',
    }),
  ],
};
