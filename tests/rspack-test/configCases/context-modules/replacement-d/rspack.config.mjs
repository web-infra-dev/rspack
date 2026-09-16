import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: ['./queryloader?lions=roar'],
      },
    ],
  },
  plugins: [
    new rspack.ContextReplacementPlugin(
      /replacement.d$/,
      path.resolve(import.meta.dirname, 'modules?cats=meow'),
      {
        a: path.resolve(import.meta.dirname, './modules/a'),
      },
    ),
  ],
};
