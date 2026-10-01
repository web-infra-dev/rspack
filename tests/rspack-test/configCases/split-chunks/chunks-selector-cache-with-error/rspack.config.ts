import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'async-node',
  // All four leaves have the same three-chunk membership.
  entry: { a: './index.js', b: './index.js', c: './index.js' },
  output: { filename: '[name].js' },
  optimization: {
    concatenateModules: false,
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: {
          test: /shared-\d+\.js$/,
          minChunks: 2,
          enforce: true,
          chunks() {
            throw new Error('CHUNKS_FUNCTION_WITH_ERROR');
          },
        },
      },
    },
  },
});
