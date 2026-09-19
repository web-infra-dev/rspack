import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    library: { type: 'amd' },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner: 'function define(fn) { fn(); }\n',
    }),
  ],
});
