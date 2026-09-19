import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack } from '@rspack/core';

export default defineConfig({
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: ['./queryloader.mjs?lions=roar'],
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
});
