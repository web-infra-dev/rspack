import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    splitChunks: {
      cacheGroups: {
        async: {
          chunks: 'async',
          test: /common/,
          minChunks: 1,
          reuseExistingChunk: false,
          // enforce: true
        },
      },
    },
  },
});
