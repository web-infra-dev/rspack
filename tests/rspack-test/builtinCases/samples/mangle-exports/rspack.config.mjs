import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    mangleExports: 'deterministic',
    providedExports: true,
    usedExports: 'global',
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
};
