import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'async-node',
  externals: ['path', 'fs'],
  externalsType: 'module',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
    library: {
      type: 'modern-module',
    },
  },
});
