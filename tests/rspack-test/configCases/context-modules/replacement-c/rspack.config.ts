import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack } from '@rspack/core';

export default defineConfig({
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
});
