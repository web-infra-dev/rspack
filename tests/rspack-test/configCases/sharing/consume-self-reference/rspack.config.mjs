import { sharing } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { SharePlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new SharePlugin({
      shared: {
        'my-middleware': {
          singleton: true,
          // import: false
        },
        'my-module/a': {
          singleton: true,
          version: '1.2.3',
          // import: false
        },
        'my-module/b': {
          singleton: true,
          version: '1.2.3',
          // import: false
        },
      },
    }),
  ],
};
