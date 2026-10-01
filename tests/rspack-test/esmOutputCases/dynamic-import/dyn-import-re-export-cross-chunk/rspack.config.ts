import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    splitChunks: {
      cacheGroups: {
        shared: {
          test: /shared\.js$/,
          name: 'shared-chunk',
        },
      },
    },
  },
});
