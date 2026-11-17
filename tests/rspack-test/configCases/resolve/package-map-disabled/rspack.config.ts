import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    packageMap: path.resolve(import.meta.dirname, 'package-map.json'),
    byDependency: {
      commonjs: {
        packageMap: false,
      },
    },
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
