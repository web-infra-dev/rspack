import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    runtimeChunk: false,
    splitChunks: {
      cacheGroups: {
        module: {
          test: /module\.js$/,
        },
      },
    },
  },
});
