import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

const config = defineConfig({
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [
          {
            loader: fileURLToPath(import.meta.resolve('./loader.mjs')),
          },
        ],
      },
    ],
  },
});

export default config;
