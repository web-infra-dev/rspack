import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
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
        loader: './loader.mjs',
        sideEffects: true,
      },
    ],
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.compilation.tap('testcase', (compilation) => {
        compilation.hooks.afterProcessAssets.tap('testcase', (assets) => {
          const source = assets['entry.mjs'].source();
          let snapshotDir;
          if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
            snapshotDir = path.join(
              import.meta.dirname,
              '__snapshots__',
              'runtimeModeSnapshot',
            );
          } else {
            snapshotDir = path.join(import.meta.dirname, '__snapshots__');
          }
          expect(source).toMatchFileSnapshotSync(
            path.join(snapshotDir, `entry.mjs.txt`),
          );
        });
      });
    }),
  ],
});
