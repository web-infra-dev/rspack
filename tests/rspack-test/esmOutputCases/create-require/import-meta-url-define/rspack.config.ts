import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
export default defineConfig({
  module: {
    parser: {
      javascript: {
        createRequire: true,
      },
    },
    rules: [
      {
        test: /index\.js$/,
        parser: {
          requireResolve: false,
        },
      },
    ],
  },
  plugins: [
    new DefinePlugin({
      'import.meta.url': JSON.stringify(
        pathToFileURL(
          path.join(import.meta.dirname, 'defined-context/index.js'),
        ).href,
      ),
    }),
  ],
});
