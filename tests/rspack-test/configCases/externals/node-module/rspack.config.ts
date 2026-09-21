import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
  },
});
