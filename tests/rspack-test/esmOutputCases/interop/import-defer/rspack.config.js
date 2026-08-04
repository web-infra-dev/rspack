const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './deferred-external': `node-commonjs ${path.resolve(
      __dirname,
      'deferred-external.cjs',
    )}`,
  },
  experiments: {
    deferImport: true,
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        staticAsyncDependency: {
          test: /[/\\]import-defer[/\\]async-dependency\.js$/,
          name: 'static-async-dependency',
          chunks: 'all',
          enforce: true,
        },
        dynamicAsyncDependency: {
          test: /dynamic-async-dependency\.js$/,
          name: 'dynamic-async-dependency',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
};
