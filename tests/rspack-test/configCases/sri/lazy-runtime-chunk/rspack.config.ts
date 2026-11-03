import { defineConfig } from '@rspack/cli';
import { SubresourceIntegrityPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'production',
  target: 'web',
  entry: {
    main: './index.js',
  },
  externals: {
    crypto: "require('crypto')",
    fs: "require('fs')",
    path: "require('path')",
  },
  node: {
    __dirname: false,
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
    crossOriginLoading: 'anonymous',
  },
  optimization: {
    minimize: false,
    runtimeChunk: 'single',
  },
  plugins: [
    new SubresourceIntegrityPlugin({
      hashFuncNames: ['sha256'],
      hashLoading: 'lazy',
    }),
  ],
});
