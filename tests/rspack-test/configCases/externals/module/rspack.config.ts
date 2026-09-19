import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: ['path'],
  externalsType: 'module',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
  },
});
