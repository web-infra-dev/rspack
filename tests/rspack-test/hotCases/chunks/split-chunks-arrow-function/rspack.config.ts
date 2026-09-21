import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].js',
    environment: {
      arrowFunction: true,
    },
  },
  optimization: {
    chunkIds: 'total-size',
    splitChunks: {
      chunks: 'all',
      minSize: 0,
    },
  },
});
