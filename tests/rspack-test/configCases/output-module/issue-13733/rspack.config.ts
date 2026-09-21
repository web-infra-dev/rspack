import type { ExternalItemFunctionData } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'production',
  target: 'node14',
  entry: './index.js',
  output: {
    filename: '[name].mjs',
    module: true,
    chunkFormat: 'module',
    chunkLoading: 'import',
  },
  optimization: {
    mangleExports: 'size',
    minimize: false,
  },
  plugins: [
    new rspack.optimize.LimitChunkCountPlugin({
      maxChunks: 1,
    }),
  ],
  externals(ctx: ExternalItemFunctionData) {
    if (ctx.request?.startsWith('node:')) {
      return `module ${ctx.request}`;
    }
  },
});
