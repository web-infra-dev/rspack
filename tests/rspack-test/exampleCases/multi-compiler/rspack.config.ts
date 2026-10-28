import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

export default defineConfig([
  {
    name: 'mobile',
    // mode: "development" || "production",
    entry: './example',
    output: {
      path: path.join(import.meta.dirname, 'dist'),
      filename: 'mobile.js',
    },
    plugins: [
      new webpack.DefinePlugin({
        ENV: JSON.stringify('mobile'),
      }),
    ],
  },

  {
    name: 'desktop',
    // mode: "development" || "production",
    entry: './example',
    output: {
      path: path.join(import.meta.dirname, 'dist'),
      filename: 'desktop.js',
    },
    plugins: [
      new webpack.DefinePlugin({
        ENV: JSON.stringify('desktop'),
      }),
    ],
  },
]);
