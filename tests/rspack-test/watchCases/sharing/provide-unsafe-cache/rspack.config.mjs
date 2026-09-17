import { sharing } from '@rspack/core';

const { ProvideSharedPlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ProvideSharedPlugin({
      provides: ['package'],
    }),
  ],
};
