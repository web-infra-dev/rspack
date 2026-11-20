import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    packageMap: path.resolve(import.meta.dirname, 'package-map.json'),
  },
  module: {
    rules: [
      {
        test: /legacy\.js$/,
        resolve: {
          packageMap: false,
        },
      },
    ],
  },
});
