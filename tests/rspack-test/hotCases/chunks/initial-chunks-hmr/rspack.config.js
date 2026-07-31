'use strict';

module.exports = {
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    minimize: false,
    concatenateModules: false,
    splitChunks: {
      minSize: 1000,
      chunks: 'all',
      cacheGroups: {
        lib: {
          test: /lib-js/,
          name: 'lib',
        },
        default: false,
        defaultVendors: false,
      },
    },
    mangleExports: false,
  },
  output: {
    filename: '[name].[fullhash].js',
    chunkFilename: 'async/[name].js',
    chunkLoadingGlobal: '__INITIAL_CHUNK_HMR_CHUNKS__',
  },
};
