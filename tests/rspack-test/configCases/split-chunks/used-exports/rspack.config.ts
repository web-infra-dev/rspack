import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    foo: './foo',
    bar: './bar',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    usedExports: true,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        shared: {
          test: /shared/,
          minSize: 0,
        },
      },
    },
  },
});
