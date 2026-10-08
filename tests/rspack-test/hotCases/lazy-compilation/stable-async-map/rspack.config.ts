import { defineConfig } from '@rspack/cli';

export default defineConfig({
  lazyCompilation: { entries: false, test: /feature/ },
  optimization: {
    splitChunks: {
      chunks: 'async',
      minSize: 0,
      cacheGroups: { shared: { test: /shared/, name: 'shared', minChunks: 2 } },
    },
  },
});
