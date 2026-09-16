import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.DefinePlugin({
      'process.env.DIR': JSON.stringify('sub'),
    }),
  ],
};
