import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */

export default {
  plugins: [new rspack.experiments.RslibPlugin()],
  experiments: { outputModule: true },
  output: {
    filename: '[name].mjs',
    module: true,
    library: { type: 'module' },
  },
  optimization: {
    runtimeChunk: false,
    minimize: false,
  },
  entry: {
    main: './index.js',
    'worker-source': './worker.js',
  },
  externalsType: 'module',
  externals: [
    ({ request, contextInfo }, callback) => {
      if (contextInfo.issuer && request === './worker.js') {
        callback(undefined, './worker-source.mjs');
        return;
      }
      callback();
    },
  ],
  module: {
    parser: {
      javascript: {
        worker: true,
      },
    },
  },
};
