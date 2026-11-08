import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  target: 'node',
  optimization: {
    chunkIds: 'named',
    usedExports: true,
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        defaultVendors: {
          test: /[\\/]node_modules[\\/]/,
          enforce: true,
        },
      },
    },
  },
  entry: {
    a: './1',
    b: './2',
    c: './3',
  },
});
