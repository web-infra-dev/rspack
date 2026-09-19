import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.DefinePlugin({
      'process.env.a': 'process.env.a',
      a: 'b',
      b: 'a',
      'typeof process.env.b': 'typeof process.env.b',
      'typeof a': 'typeof b',
      'typeof b': 'typeof a',
    }),
  ],
});
