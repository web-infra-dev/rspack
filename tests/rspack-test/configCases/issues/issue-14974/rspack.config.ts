import { defineConfig } from '@rspack/cli';

import { HotModuleReplacementPlugin } from '@rspack/core';

export default defineConfig({
  devtool: false,
  optimization: { usedExports: false, sideEffects: false },
  plugins: [new HotModuleReplacementPlugin()],
});
