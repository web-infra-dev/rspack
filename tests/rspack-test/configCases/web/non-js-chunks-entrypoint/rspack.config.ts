import { defineConfig } from '@rspack/cli';

import { sharing } from '@rspack/core';

const { ProvideSharedPlugin } = sharing;

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  target: 'web',
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      chunks: 'all',
      minSize: 1,
      cacheGroups: {
        share: {
          type: 'provide-module',
          name: 'provide-module',
          enforce: true,
        },
      },
    },
  },
  plugins: [
    new ProvideSharedPlugin({
      provides: ['package'],
    }),
  ],
});
