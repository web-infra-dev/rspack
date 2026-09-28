import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: {
    main: {
      import: './index.js',
      filename: 'main',
    },
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        vendor: {
          chunks: 'all',
          reuseExistingChunk: true,
          test: /[\\/]node_modules[\\/]/,
          minSize: 0,
          minChunks: 1,
        },
      },
    },
  },
});
