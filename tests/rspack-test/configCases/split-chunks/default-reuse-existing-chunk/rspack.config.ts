import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      minSize: 1,
      chunks: 'all',
      cacheGroups: {
        foo: {
          minSize: 0,
        },
      },
    },
  },
});
