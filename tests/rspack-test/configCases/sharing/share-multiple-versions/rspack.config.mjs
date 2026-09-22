import { sharing } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { SharePlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {},
  plugins: [
    new SharePlugin({
      shared: ['shared'],
    }),
  ],
};
