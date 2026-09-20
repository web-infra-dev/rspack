import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  target: 'node',
  entry: './index',
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.js$/,
        extractSourceMap: true,
        loader: fileURLToPath(import.meta.resolve('./babel-loader.mjs')),
      },
    ],
  },
});
