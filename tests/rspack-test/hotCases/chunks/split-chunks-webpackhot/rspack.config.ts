import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'total-size',
    splitChunks: {
      chunks: 'all',
      minSize: 0,
    },
  },
});
