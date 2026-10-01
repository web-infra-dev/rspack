import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig((_env, { testPath }) => ({
  target: 'node14',
  output: {
    module: true,
    chunkLoading: 'import',
  },
  resolve: {
    alias: {
      library: path.resolve(testPath, '../0-create-library/lib.js'),
    },
  },
}));
