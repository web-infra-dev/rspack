import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  plugins: [new rspack.HotModuleReplacementPlugin()],
  mode: 'production',
  stats: {
    assets: true,
    modules: true,
  },
});
