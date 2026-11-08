import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const CHILD_ID = 'child';
const CHILD_FILENAME = './child';

export default defineConfig({
  devtool: 'source-map',
  node: {
    __dirname: false,
    __filename: false,
  },
  externals: {
    CHILD_FILENAME: `commonjs ${CHILD_FILENAME}`,
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new rspack.DefinePlugin({
      CONTEXT: JSON.stringify(import.meta.dirname),
    }),
    definePlugin((compiler) => {
      compiler.hooks.make.tapAsync('PLUGIN', (compilation, callback) => {
        const outputOptions = {};
        const childCompiler = compilation.createChildCompiler(
          CHILD_ID,
          outputOptions,
          [],
        );
        const SingleEntryPlugin = compiler.rspack.EntryPlugin;
        new SingleEntryPlugin(
          compiler.context,
          path.join(import.meta.dirname, CHILD_FILENAME),
          CHILD_ID,
        ).apply(childCompiler);
        childCompiler.runAsChild((err) => callback(err));
      });
    }),
  ],
});
