import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  cache: false,
  incremental: 'advance',
  ignoreWarnings: [/is not friendly for incremental/],
  devtool: false,
  optimization: {
    moduleIds: 'named',
    chunkIds: 'deterministic',
    concatenateModules: false,
    minimize: false,
    splitChunks: {
      chunks: 'async',
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: { test: /shared\.js$/, minChunks: 1, enforce: true },
      },
    },
  },
});
