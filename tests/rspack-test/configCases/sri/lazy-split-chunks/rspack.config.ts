import { defineConfig } from '@rspack/cli';
import { SubresourceIntegrityPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'production',
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
    chunkFilename: '[name].[contenthash:8].js',
    crossOriginLoading: 'anonymous',
  },
  optimization: {
    minimize: false,
    splitChunks: {
      chunks: 'async',
      minSize: 0,
      cacheGroups: {
        lib: { test: /lib\.js$/, name: 'lib', enforce: true },
      },
    },
  },
  plugins: [
    new SubresourceIntegrityPlugin({
      hashFuncNames: ['sha256'],
      hashLoading: 'lazy',
    }),
  ],
});
