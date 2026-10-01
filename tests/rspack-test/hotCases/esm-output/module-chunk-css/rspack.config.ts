import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.mjs',
    enabledLibraryTypes: ['module'],
  },
  optimization: {
    minimize: false,
    runtimeChunk: 'single',
  },
});
