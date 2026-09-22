import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        // The worker returns immediately after this uncached parallel loader.
        test: /worker-return\.js$/,
        use: [
          {
            loader: path.resolve(import.meta.dirname, 'dependency-loader.mjs'),
            options: { dependency: 'return-dependency.txt' },
            parallel: { maxWorkers: 1 },
            cache: false,
          },
        ],
      },
      {
        // The worker yields after the uncached parallel loader because the next loader runs on the
        // main thread.
        test: /worker-yield\.js$/,
        use: [
          {
            loader: path.resolve(import.meta.dirname, 'passthrough-loader.mjs'),
          },
          {
            loader: path.resolve(import.meta.dirname, 'dependency-loader.mjs'),
            options: { dependency: 'yield-dependency.txt' },
            parallel: { maxWorkers: 1 },
          },
        ],
      },
    ],
  },
});
