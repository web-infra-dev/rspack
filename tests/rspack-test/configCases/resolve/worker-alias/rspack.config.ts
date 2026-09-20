import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    splitChunks: false,
  },
  output: {
    chunkFilename: 'worker.js',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/auto',
        resolve: {
          alias: {
            somefakemodule: path.resolve(
              import.meta.dirname,
              './node_modules/corejs',
            ),
          },
        },
      },
    ],
  },
});
