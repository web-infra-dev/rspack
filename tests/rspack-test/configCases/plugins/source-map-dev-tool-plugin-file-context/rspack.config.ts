import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new rspack.SourceMapDevToolPlugin({
      filename: 'sourcemaps/[file].map',
      fileContext: 'assets',
      publicPath: 'http://localhost:50505/',
    }),
  ],
});
