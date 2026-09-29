import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
    library: { type: 'module' },
    iife: false,
    chunkFormat: 'module',
    filename: 'bundle0.mjs',
  },
  target: 'node',
  optimization: {
    minimize: true,
  },
});
