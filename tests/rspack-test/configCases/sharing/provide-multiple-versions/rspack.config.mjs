import { sharing } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { ProvideSharedPlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ProvideSharedPlugin({
      provides: ['shared'],
    }),
  ],
};
