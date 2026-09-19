import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack as webpack } from '@rspack/core';
export default defineConfig({
  // mode: "development" || "production",
  resolve: {
    extensions: ['.js', '.jsx'],
  },
  entry: {
    alpha: ['./alpha', './a', 'module'],
    beta: ['./beta', './b', './c'],
  },
  output: {
    path: path.join(import.meta.dirname, 'dist'),
    filename: 'MyDll.[name].js',
    library: '[name]_[fullhash]',
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.join(import.meta.dirname, 'dist', '[name]-manifest.json'),
      name: '[name]_[fullhash]',
    }),
  ],
});
