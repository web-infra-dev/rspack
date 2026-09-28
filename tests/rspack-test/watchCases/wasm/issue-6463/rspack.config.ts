import { defineConfig } from '@rspack/cli';
import { HotModuleReplacementPlugin } from '@rspack/core';

export default defineConfig({
  experiments: {
    asyncWebAssembly: true,
  },
  plugins: [new HotModuleReplacementPlugin()],
});
