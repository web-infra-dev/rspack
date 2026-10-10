import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: { filename: '[name].js' },
  optimization: {
    runtimeChunk: 'single',
    chunkIds: 'named',
    moduleIds: 'named',
    concatenateModules: false,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: { test: /shared\.js$/, name: 'shared', enforce: true },
      },
    },
  },
});
