const rspack = require('@rspack/core');

module.exports = {
  mode: 'development',
  entry: {
    main: './index.js',
    worker: './lib.js',
  },
  output: {
    filename: '[name].mjs',
    chunkFilename: '[name].mjs',
    library: {
      type: 'module',
    },
  },
  optimization: {
    runtimeChunk: 'single',
    mergeDuplicateChunks: false,
    removeAvailableModules: false,
  },
  plugins: [new rspack.experiments.RemoveDuplicateModulesPlugin()],
};
