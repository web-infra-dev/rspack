import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  externals: [/\.\/externals\/.*/],
  externalsType: 'module',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
    library: {
      type: 'modern-module',
    },
  },
  optimization: {
    avoidEntryIife: true,
  },
  plugins: [
    new rspack.CopyRspackPlugin({
      patterns: ['./externals/**/*'],
    }),
  ],
});
