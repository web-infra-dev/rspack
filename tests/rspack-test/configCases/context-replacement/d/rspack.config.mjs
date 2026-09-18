import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: ['./queryloader.mjs?lions=roar'],
      },
    ],
  },
  plugins: [
    new webpack.ContextReplacementPlugin(
      /context-replacement.d$/,
      path.resolve(import.meta.dirname, 'modules?cats=meow'),
      {
        a: path.resolve(import.meta.dirname, './modules/a'),
      },
    ),
  ],
};
