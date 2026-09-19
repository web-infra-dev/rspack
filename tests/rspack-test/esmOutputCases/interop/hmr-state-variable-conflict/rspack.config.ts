import { defineConfig } from '@rspack/cli';

import { HotModuleReplacementPlugin } from '@rspack/core';
export default defineConfig({
  optimization: {
    runtimeChunk: false,
  },
  plugins: [new HotModuleReplacementPlugin()],
});
