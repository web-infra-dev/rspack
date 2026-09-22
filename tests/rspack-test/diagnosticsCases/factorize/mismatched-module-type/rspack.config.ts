import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const resolve = (filename: string) =>
  path.resolve(import.meta.dirname, filename);

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: resolve('app.jsx'),
        type: 'javascript/auto',
      },
      {
        test: resolve('app.tsx'),
        type: 'ts',
      },
    ],
  },
});
