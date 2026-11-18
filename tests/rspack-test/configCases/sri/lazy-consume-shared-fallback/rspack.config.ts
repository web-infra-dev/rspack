import { defineConfig } from '@rspack/cli';
import { SubresourceIntegrityPlugin, sharing } from '@rspack/core';

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
  optimization: {
    minimize: false,
  },
  plugins: [
    new sharing.ConsumeSharedPlugin({
      consumes: {
        'shared-lib': {
          import: './fallback.js',
          requiredVersion: false,
          eager: false,
        },
      },
    }),
    new SubresourceIntegrityPlugin({
      hashFuncNames: ['sha256'],
      hashLoading: 'lazy',
    }),
  ],
});
