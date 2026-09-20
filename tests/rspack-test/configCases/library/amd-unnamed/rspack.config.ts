import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    library: { type: 'amd' },
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner: 'function define(fn) { fn(); }\n',
    }),
  ],
});
