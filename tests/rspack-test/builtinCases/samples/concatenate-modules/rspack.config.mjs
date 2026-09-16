import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    concatenateModules: true,
    sideEffects: true,
    providedExports: true,
    usedExports: 'global',
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
};
