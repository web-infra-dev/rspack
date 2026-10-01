import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    bundle0: './index',
    a: './a',
    b: './b',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    sideEffects: false,
    splitChunks: {
      cacheGroups: {
        default: false,
        defaultVendors: false,
        test: {
          test: /shared/,
          minChunks: 1,
          usedExports: false,
          chunks: 'initial',
          minSize: 100,
          minRemainingSize: 0,
        },
      },
    },
  },
});
