import { sharing } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { SharePlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    uniqueName: 'b',
  },
  plugins: [
    new SharePlugin({
      shared: ['package'],
    }),
  ],
};
