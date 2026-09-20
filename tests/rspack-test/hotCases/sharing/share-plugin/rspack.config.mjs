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
        common: {
          eager: true,
          import: './common?1',
          requiredVersion: '1.1.1',
        },
        common2: {
          import: './common?2',
          requiredVersion: '1.1.1',
        },
      },
    }),
  ],
};
