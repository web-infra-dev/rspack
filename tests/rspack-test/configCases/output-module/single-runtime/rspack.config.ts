import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
    filename: '[name].mjs',
    chunkFormat: 'module',
    chunkLoading: 'import',
    library: {
      type: 'module',
    },
  },
  optimization: {
    runtimeChunk: true,
  },
  // target: "node14"
});
