import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig([
  {
    // no hmr
  },
  {
    // with hmr
    plugins: [new rspack.HotModuleReplacementPlugin()],
  },
]);
