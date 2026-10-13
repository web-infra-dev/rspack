import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'node',
  output: {
    filename: '[name].mjs',
    chunkFilename: '[id].mjs',
    library: { type: 'modern-module' },
  },
  optimization: { minimize: false, runtimeChunk: false },
});
