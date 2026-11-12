import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    splitChunks: {
      cacheGroups: {
        ab: {
          test: /[ab]\.js$/,
          name: 'ab-chunk',
          chunks: 'all',
        },
      },
    },
  },
});
