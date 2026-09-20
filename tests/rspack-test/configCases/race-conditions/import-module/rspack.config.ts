import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  parallelism: 1,
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'javascript/auto',
        use: [fileURLToPath(import.meta.resolve('./loader.mjs')), 'css-loader'],
      },
    ],
  },
});
