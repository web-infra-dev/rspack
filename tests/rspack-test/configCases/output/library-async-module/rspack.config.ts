import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  output: {
    module: true,
    chunkFormat: 'module',
    library: {
      type: 'module',
    },
  },
});
