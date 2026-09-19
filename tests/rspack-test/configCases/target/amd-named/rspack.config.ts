import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    library: { type: 'amd', name: 'NamedLibrary' },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner: 'function define(name, deps, fn) { fn(); }\n',
    }),
  ],
});
