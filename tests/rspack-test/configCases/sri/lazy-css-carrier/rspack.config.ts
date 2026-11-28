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
    chunkFilename: '[name].js',
    crossOriginLoading: 'anonymous',
  },
  module: {
    rules: [{ test: /\.css$/, type: 'css/auto' }],
  },
  optimization: {
    minimize: false,
    splitChunks: {
      chunks: 'async',
      minSize: 0,
      cacheGroups: {
        aCode: { test: /[\\/]a\.js$/, name: 'a-code', enforce: true },
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
