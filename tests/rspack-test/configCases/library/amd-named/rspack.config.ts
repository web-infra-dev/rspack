import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    library: { type: 'amd', name: 'NamedLibrary' },
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner: 'function define(name, deps, fn) { fn(); }\n',
    }),
  ],
});
