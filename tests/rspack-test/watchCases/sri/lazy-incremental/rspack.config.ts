import { defineConfig } from '@rspack/cli';
import { SubresourceIntegrityPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  devtool: false,
  cache: false,
  // Rebuilds recompute only the chunks a change affects.
  incremental: {
    // Report the passes the plugin disables, so the test can check for them.
    silent: false,
    chunksRuntimeRequirements: true,
    chunksHashes: false,
    chunkAsset: false,
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
