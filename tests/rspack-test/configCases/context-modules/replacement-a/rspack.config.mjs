import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.ContextReplacementPlugin(
      /replacement.a$/,
      'new-context',
      true,
      /^replaced$/,
    ),
  ],
};
