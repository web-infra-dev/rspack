const rspack = require('@rspack/core');

module.exports = {
  mode: 'production',
  target: 'node',
  optimization: {
    concatenateModules: false,
    sideEffects: true,
    usedExports: true,
    mangleExports: 'deterministic',
    minimize: false,
    runtimeChunk: false,
    avoidEntryIife: true,
  },
  output: {
    module: true,
    chunkFormat: false,
    library: {
      type: 'modern-module',
    },
    chunkLoading: 'import',
  },
  plugins: [
    new rspack.experiments.RslibPlugin({
      interceptApiPlugin: true,
      autoCjsNodeBuiltin: true,
    }),
  ],
};
