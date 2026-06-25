'use strict';
const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  optimization: {
    minimize: false,
    moduleIds: 'named',
    concatenateModules: true,
    usedExports: true,
  },
  entry: {
    main: './index.js',
    entry: './entry.js',
  },
  output: {
    module: true,
    clean: true,
    filename: '[name].mjs',
    library: {
      type: 'module',
    },
  },
  externalsType: 'module',
  externals: ['externals0', 'externals1'],
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: './loader',
        sideEffects: true,
      },
    ],
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap(
        'testcase',
        (/** @type {import("@rspack/core").Compilation} */ compilation) => {
          compilation.hooks.afterProcessAssets.tap(
            'testcase',
            (
              /** @type {Record<string, import("webpack-sources").Source>} */ assets,
            ) => {
              const source = assets['entry.mjs'].source();
              let snapshotDir;
              if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
                snapshotDir = path.join(
                  __dirname,
                  '__snapshots__',
                  'runtimeModeSnapshot',
                );
              } else {
                snapshotDir = path.join(__dirname, '__snapshots__');
              }
              expect(source).toMatchFileSnapshotSync(
                path.join(snapshotDir, `entry.mjs.txt`),
              );
            },
          );
        },
      );
    },
  ],
};
