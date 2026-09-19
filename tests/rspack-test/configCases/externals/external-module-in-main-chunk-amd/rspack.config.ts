import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    library: { type: 'amd' },
  },
  externals: {
    external: 'external',
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner: 'function define(deps, fn) { fn(); }\n',
    }),
  ],
});
