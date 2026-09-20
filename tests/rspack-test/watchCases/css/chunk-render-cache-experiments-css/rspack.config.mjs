import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    ab: './ab.js',
    ba: './ba.js',
  },
  output: {
    filename: '[name].js',
  },
  target: 'web',
  optimization: {
    splitChunks: {
      cacheGroups: {
        styles: {
          name: 'styles',
          chunks: 'all',
          test: /\.css$/,
          enforce: true,
        },
      },
    },
  },
};
