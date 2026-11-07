import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  module: {
    rules: [
      {
        test: /dep\.js$/,
        // Fixed to this checkout: it does not move with the project.
        include: path.join(import.meta.dirname, 'src'),
        use: ['./loader.cjs'],
      },
    ],
  },
});
