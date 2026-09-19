import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  resolve: {
    tsConfig: path.resolve(import.meta.dirname, './tsconfig.json'),
  },
});
