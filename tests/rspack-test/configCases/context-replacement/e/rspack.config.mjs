import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    modules: ['...', path.resolve(import.meta.dirname, 'new-context/modules')],
  },
  plugins: [
    new rspack.ContextReplacementPlugin(
      /context-replacement.e$/,
      'new-context',
      true,
      /^replaced$|^\.\/modules\/rep/,
    ),
  ],
};
