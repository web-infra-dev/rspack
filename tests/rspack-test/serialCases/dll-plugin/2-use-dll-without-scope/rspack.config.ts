import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack as webpack } from '@rspack/core';
import { readFileSync } from 'node:fs';

export default defineConfig({
  module: {
    rules: [
      {
        oneOf: [
          {
            test: /\.abc\.js$/,
            loader: '../0-create-dll/g-loader.mjs',
            options: {
              test: 1,
            },
          },
        ],
      },
    ],
  },
  optimization: {
    moduleIds: 'named',
  },
  resolve: {
    extensions: ['.js', '.jsx'],
  },
  plugins: [
    new webpack.DllReferencePlugin({
      manifest: JSON.parse(
        readFileSync(
          new URL(
            '../../../js/config/dll-plugin/manifest0.json',
            import.meta.url,
          ),
          'utf-8',
        ),
      ), // eslint-disable-line node/no-missing-require
      name: '../0-create-dll/dll.js',
      context: path.resolve(import.meta.dirname, '../0-create-dll'),
      sourceType: 'commonjs2',
    }),
  ],
});
