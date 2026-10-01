import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    minimize: false,
  },
  output: {
    library: {
      type: 'module',
    },
    filename: '[name].mjs',
    module: true,
    chunkFormat: 'module',
    chunkLoading: 'import',
  },
});
