import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.mjs',
    enabledLibraryTypes: ['module'],
  },
  optimization: {
    minimize: false,
  },
});
