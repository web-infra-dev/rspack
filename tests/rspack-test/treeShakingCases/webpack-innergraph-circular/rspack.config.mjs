import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    sideEffects: false,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'production'",
    }),
  ],
};
