import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new webpack.ContextReplacementPlugin(
      /context-replacement.a$/,
      'new-context',
      true,
      /^replaced$/,
    ),
  ],
};
