import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  resolve: {
    tsConfig: {
      configFile: path.resolve(import.meta.dirname, './tsconfig.json'),
    },
  },
  module: {
    rules: [
      {
        test: /index/,
        loader: './loader.mjs',
      },
    ],
  },
});
