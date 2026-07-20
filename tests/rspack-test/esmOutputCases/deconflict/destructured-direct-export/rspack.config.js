const rspack = require('@rspack/core');

module.exports = {
  mode: 'production',
  target: 'node',
  module: {
    parser: {
      javascript: {
        importMeta: false,
        importDynamic: false,
        commonjs: {
          exports: 'skipInEsm',
        },
        requireResolve: false,
        requireDynamic: false,
        requireAsExpression: false,
        worker: false,
      },
    },
  },
  optimization: {
    concatenateModules: false,
    sideEffects: true,
    usedExports: true,
    mangleExports: 'deterministic',
    minimize: false,
    runtimeChunk: false,
    avoidEntryIife: true,
    splitChunks: {
      chunks: 'async',
    },
  },
  output: {
    module: true,
    chunkFormat: false,
    library: {
      type: 'modern-module',
    },
    chunkLoading: 'import',
    workerChunkLoading: 'import',
  },
  plugins: [
    new rspack.experiments.RslibPlugin({
      interceptApiPlugin: true,
      autoCjsNodeBuiltin: true,
    }),
  ],
};
