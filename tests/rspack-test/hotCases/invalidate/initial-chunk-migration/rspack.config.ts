import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: { main: './index.js', other: './other.js' },
  output: { filename: '[name].js' },
  optimization: {
    runtimeChunk: 'single',
    chunkIds: 'named',
    concatenateModules: false,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: { test: /shared\.js$/, name: 'shared', minChunks: 2 },
      },
    },
  },
});
