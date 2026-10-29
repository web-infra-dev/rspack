'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        shared: {
          test: /shared/,
          name: 'shared',
          chunks: 'async',
          enforce: true,
        },
      },
    },
  },
};
