import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [new rspack.experiments.RslibPlugin()],
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
      if (contextInfo?.issuer && request === './worker.js') {
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
});
