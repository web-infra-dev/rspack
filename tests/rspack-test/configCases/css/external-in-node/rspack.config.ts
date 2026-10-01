import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  context: path.join(import.meta.dirname, '../external'),
  entry: '../external-in-node/index.js',
  target: 'node',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
