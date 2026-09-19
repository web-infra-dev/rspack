import { defineConfig } from '@rspack/cli';

import { fileURLToPath } from 'node:url';

export default defineConfig({
  target: 'web',
  output: {
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.txt$/,
        loader: fileURLToPath(import.meta.resolve('./loader.mjs')),
      },
      {
        test: /\.text$/,
        type: 'asset/source',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
