import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  mode: 'development',
  output: {
    hashDigestLength: 8,
  },
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'asset/resource',
        generator: {
          filename: () => 'css/style.[contenthash].css',
        },
        use: [
          {
            loader: fileURLToPath(import.meta.resolve('./loader.mjs')),
          },
        ],
      },
    ],
  },
});
