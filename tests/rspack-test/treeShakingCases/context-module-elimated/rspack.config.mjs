import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    sideEffects: true,
    // Avoid inlineExports causes resolve error and failed to create the context module
    inlineExports: false,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
};
