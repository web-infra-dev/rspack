import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  externals: {
    'node:fs': 'node-commonjs node:fs',
    'node:path': 'node-commonjs node:path',
  },
  entry: {
    bundle0: {
      import: './index.js',
      layer: 'main',
    },
  },
  plugins: [
    new rspack.DefinePlugin({
      __RUNTIME_TYPE__: '__webpack_layer__',
    }),
  ],
});
