import { rspack } from '@rspack/core';

/**
 * @type {import("@rspack/core").Configuration}
 */
export default {
  plugins: [
    new rspack.DefinePlugin({
      'typeof window': JSON.stringify('undefined'),
    }),
  ],
};
