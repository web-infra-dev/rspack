const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = [
  {
    entry: {
      main: './main.js',
    },
    output: {
      filename: '[name].js',
      chunkFilename: 'async-[name].js',
      module: true,
      library: {
        type: 'modern-module',
      },
      iife: false,
      chunkFormat: 'module',
      chunkLoading: 'import',
      workerChunkLoading: 'import',
    },
    externalsType: 'module-import',
    plugins: [new rspack.experiments.RslibPlugin()],
    optimization: {
      concatenateModules: true,
      avoidEntryIife: true,
      minimize: false,
    },
  },
  {
    entry: {
      index: './index.js',
    },
    output: {
      module: true,
      filename: 'index.mjs',
    },
  },
];
