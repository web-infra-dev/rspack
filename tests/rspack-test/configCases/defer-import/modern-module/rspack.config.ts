import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  experiments: { deferImport: true },
  output: {
    filename: 'bundle0.mjs',
    module: true,
    library: { type: 'modern-module' },
  },
  optimization: { minimize: false, runtimeChunk: false },
});
