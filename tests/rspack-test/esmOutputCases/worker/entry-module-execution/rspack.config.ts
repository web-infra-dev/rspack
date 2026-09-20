import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  optimization: {
    runtimeChunk: false,
  },
  plugins: [new rspack.experiments.RslibPlugin()],
});
