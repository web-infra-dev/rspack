import { sharing } from '@rspack/core';

const { ProvideSharedPlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
