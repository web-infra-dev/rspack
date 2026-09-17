import { sharing } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { ConsumeSharedPlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ConsumeSharedPlugin({
      consumes: {
        shared: {
          import: false,
          strictVersion: true,
        },
        shared2: {
          import: false,
        },
      },
    }),
  ],
};
