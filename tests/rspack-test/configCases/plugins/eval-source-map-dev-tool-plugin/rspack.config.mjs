import path from 'node:path';
import { rspack } from '@rspack/core';

/**
 * @type {import("@rspack/core").Configuration}
 */
export default {
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
};
