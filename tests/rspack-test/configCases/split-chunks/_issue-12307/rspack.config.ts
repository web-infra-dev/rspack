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
    sideEffects: false,
    splitChunks: {
      cacheGroups: {
        default: false,
        defaultVendors: false,
        test: {
          test: /shared/,
          minChunks: 1,
          chunks: 'initial',
          minSize: 100,
        },
      },
    },
  },
});
