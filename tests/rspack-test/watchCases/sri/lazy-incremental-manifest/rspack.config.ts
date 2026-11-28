import { defineConfig } from '@rspack/cli';
import { SubresourceIntegrityPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  devtool: false,
  cache: false,
  incremental: {
    // Report the passes the plugin disables, so the test can check for them.
    silent: false,
  },
  target: 'web',
  externals: {
    crypto: "require('crypto')",
    fs: "require('fs')",
    path: "require('path')",
  },
  node: {
    __dirname: false,
  },
  output: {
    chunkFilename: '[name].js',
    crossOriginLoading: 'anonymous',
  },
  optimization: {
    minimize: false,
  },
  plugins: [
    new SubresourceIntegrityPlugin({
      enabled: true,
      hashFuncNames: ['sha256'],
      hashLoading: 'lazy',
    }),
  ],
});
