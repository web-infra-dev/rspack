import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  entry: {
    main: './index.js',
    worker: './lib.js',
  },
  output: {
    filename: '[name].mjs',
    chunkFilename: '[name].mjs',
    library: {
      type: 'module',
    },
  },
  optimization: {
    runtimeChunk: 'single',
    mergeDuplicateChunks: false,
    removeAvailableModules: false,
  },
  plugins: [new rspack.experiments.RemoveDuplicateModulesPlugin()],
});
