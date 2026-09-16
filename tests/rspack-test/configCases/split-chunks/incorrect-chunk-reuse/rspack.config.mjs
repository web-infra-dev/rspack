import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index',
  optimization: {
    splitChunks: {
      cacheGroups: {
        x: {
          test: path.resolve(import.meta.dirname, 'x'),
          name: 'x',
          priority: 2,
          enforce: true,
        },
        y: {
          test: path.resolve(import.meta.dirname, 'y'),
          priority: 1,
          name: 'y',
          enforce: true,
          reuseExistingChunk: true,
        },
      },
    },
  },
};
