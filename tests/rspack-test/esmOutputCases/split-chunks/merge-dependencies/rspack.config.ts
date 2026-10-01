import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    splitChunks: {
      cacheGroups: {
        a: {
          test: /a\.js/,
        },
        b: {
          test: /b\.js/,
        },
      },
    },
  },
});
