import { sharing } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { SharePlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: false,
  plugins: [
    new SharePlugin({
      shared: {
        '@scope/pkg': {},
        '@scope/pkg/styles': {},
      },
    }),
  ],
};
