import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  target: 'web',
  entry: './src/index.js',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(import.meta.dirname, 'src'),
    },
  },
});
