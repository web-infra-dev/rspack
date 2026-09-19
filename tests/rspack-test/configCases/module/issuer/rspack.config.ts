import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        exclude: [/index\.js/],
        use: './loader0.mjs',
      },
      {
        exclude: [/index\.js/],
        use: './loader1.mjs',
        issuer: {
          not: [/index\.js/],
        },
      },
      {
        exclude: [/index\.js/],
        use: './loader2.mjs',
        issuer: {
          and: [/1\.js/, path.resolve(import.meta.dirname, 'lib')],
        },
      },
      {
        exclude: [/index\.js/],
        use: './loader3.mjs',
        issuer: {
          or: [/1\.js/, /2\.js/],
        },
      },
    ],
  },
});
