import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.ContextReplacementPlugin(
      /replacement.c$/,
      path.resolve(import.meta.dirname, 'modules'),
      {
        a: './a',
        b: './module-b',
        './c': './module-b',
        d: 'd',
        './d': 'd',
      },
    ),
  ],
};
