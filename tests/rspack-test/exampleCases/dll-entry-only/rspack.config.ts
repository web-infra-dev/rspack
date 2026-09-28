import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  // mode: "development" || "production",
  entry: {
    dll: ['./example'],
  },
  output: {
    path: path.join(import.meta.dirname, 'dist'),
    filename: '[name].js',
    library: '[name]_[fullhash]',
  },
  optimization: {
    concatenateModules: true, // this is enabled by default in production mode
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.join(import.meta.dirname, 'dist', '[name]-manifest.json'),
      name: '[name]_[fullhash]',
      entryOnly: true,
    }),
  ],
});
