import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  entry: './index',
  optimization: {
    splitChunks: {
      cacheGroups: {
        x: {
          test: path.resolve(import.meta.dirname, 'x'),
          name: 'x',
          priority: 2,
          enforce: true,
        },
        y: {
          test: path.resolve(import.meta.dirname, 'y'),
          priority: 1,
          name: 'y',
          enforce: true,
          reuseExistingChunk: true,
        },
      },
    },
  },
});
