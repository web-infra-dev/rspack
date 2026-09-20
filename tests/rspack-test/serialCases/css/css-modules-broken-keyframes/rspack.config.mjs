import { rspack } from '@rspack/core';
import path from 'node:path';

/** @type {function(any, any): import("@rspack/core").Configuration} */
export default (env, { testPath }) => ({
  externals: {
    fs: 'node-commonjs fs',
    './use-style_js.bundle0.js': 'commonjs ./use-style_js.bundle0.js',
  },
  target: 'web',
  mode: 'production',
  output: {
    uniqueName: 'my-app',
  },
  node: {
    __dirname: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  plugins: [
    new rspack.ids.DeterministicModuleIdsPlugin({
      maxLength: 3,
      failOnConflict: true,
      fixedLength: true,
      test: (m) => m.type.startsWith('css'),
    }),
    new rspack.experiments.ids.SyncModuleIdsPlugin({
      test: (m) => m.type.startsWith('css'),
      path: path.resolve(testPath, 'module-ids.json'),
      mode: 'create',
    }),
  ],
});
