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
  optimization: {
    minimize: false,
    splitChunks: {
      chunks: 'async',
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        // Every module of the fork group moves into these, so the group keeps no chunk
        // of its own. The priorities put util, which boot also loads, last in it.
        shared: {
          test: /[\\/](fork|a|b)\.js$/,
          name: 'shared',
          minChunks: 2,
          priority: 10,
          enforce: true,
        },
        util: {
          test: /[\\/]util\.js$/,
          name: 'util',
          minChunks: 2,
          priority: 5,
          enforce: true,
        },
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
