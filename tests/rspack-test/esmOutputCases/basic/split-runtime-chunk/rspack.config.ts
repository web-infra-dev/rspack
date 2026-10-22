import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    runtimeChunk: false,
    splitChunks: {
      cacheGroups: {
        other: {
          test: /other\.js/,
          name: 'other',
        },
      },
    },
  },
});
