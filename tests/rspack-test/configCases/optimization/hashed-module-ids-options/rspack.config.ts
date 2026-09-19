import { defineConfig } from '@rspack/cli';

import { rspack as webpack } from '@rspack/core';

export default defineConfig([
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
]);
