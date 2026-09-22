import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  optimization: {
    splitChunks: {
      cacheGroups: {
        splitMain: {
          test: /index\.js/,
        },
      },
    },
  },
  plugins: [new rspack.experiments.RslibPlugin()],
});
