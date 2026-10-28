import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'production',
  target: 'node',
  entry: { main: './main.js', other: './other.js' },
  output: {
    filename: '[name].mjs',
    chunkFilename: '[id].mjs',
    library: { type: 'modern-module' },
  },
  optimization: { runtimeChunk: false, minimize: false },
  plugins: [
    new rspack.DefinePlugin({ MAIN_BUNDLE: JSON.stringify('./main.mjs') }),
  ],
});
