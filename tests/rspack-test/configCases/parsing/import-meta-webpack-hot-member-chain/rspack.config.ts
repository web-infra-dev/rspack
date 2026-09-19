import { defineConfig } from '@rspack/cli';

import { HotModuleReplacementPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  devtool: false,
  target: 'web',
  plugins: [new HotModuleReplacementPlugin()],
});
