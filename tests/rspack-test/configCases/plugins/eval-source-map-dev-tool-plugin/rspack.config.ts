import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack } from '@rspack/core';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new rspack.EvalSourceMapDevToolPlugin({
      sourceRoot: path.join(import.meta.dirname, 'folder') + '/',
    }),
    new rspack.DefinePlugin({
      CONTEXT: JSON.stringify(import.meta.dirname),
    }),
  ],
});
