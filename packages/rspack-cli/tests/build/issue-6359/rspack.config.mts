import { defineConfig } from '../../../dist/index.js';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

export default defineConfig({
  mode: 'production',
  entry: './entry.js',
  output: {
    clean: true,
    path: path.resolve(import.meta.dirname, 'dist'),
  },
  plugins: [
    {
      apply(compiler: Compiler) {
        new compiler.rspack.DefinePlugin({
          DEFINE_ME: JSON.stringify(
            `WEBPACK_SERVE=${process.env.WEBPACK_SERVE ?? '<EMPTY>'}`,
          ),
        }).apply(compiler);
      },
    },
  ],
  devServer: {
    devMiddleware: {
      writeToDisk: true,
    },
  },
});
