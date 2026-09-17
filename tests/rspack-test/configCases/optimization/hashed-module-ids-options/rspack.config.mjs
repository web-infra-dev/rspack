import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    optimization: {
      moduleIds: false,
    },
    plugins: [
      new webpack.ids.HashedModuleIdsPlugin({
        hashFunction: 'md4',
        hashDigest: 'hex',
        hashDigestLength: 8,
      }),
    ],
  },
];
