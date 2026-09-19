import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack } from '@rspack/core';

const outputPath = path.resolve(
  import.meta.dirname,
  '../../../js/config/dll-plugin/concatenated-lib-ident-context',
);

export default defineConfig({
  mode: 'production',
  context: import.meta.dirname,
  entry: './index.mjs',
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  output: {
    path: outputPath,
    filename: 'bundle.js',
    library: {
      type: 'commonjs2',
    },
  },
  plugins: [
    new rspack.DllPlugin({
      context: path.resolve(import.meta.dirname, '../../..'),
      name: 'dll',
      path: path.resolve(outputPath, 'manifest.json'),
    }),
  ],
});
