import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    sideEffects: true,
    usedExports: true,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
};
