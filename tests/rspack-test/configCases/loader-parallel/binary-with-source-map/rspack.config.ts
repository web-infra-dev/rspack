import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'logo.png'),
        use: [{ loader: './empty-loader.mjs', parallel: true, options: {} }],
        type: 'asset/resource',
      },
    ],
  },
});
