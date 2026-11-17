import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolveLoader: {
    packageMap: path.resolve(import.meta.dirname, 'package-map.json'),
  },
  module: {
    rules: [
      {
        test: /value\.js$/,
        loader: 'value-loader',
      },
    ],
  },
});
