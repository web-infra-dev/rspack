import { defineConfig } from '@rspack/cli';
import { CircularCheckRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    aa: './require-circular/d.js',
    bb: './import-circular/index.js',
    cc: './no-cycle/index.js',
    dd: './ignore-circular/a.js',
    ee: './multiple-circular/a.js',
    ff: {
      import: './multiple-circular/a.js',
      layer: 'f',
    },
    gg: './dynamic-circular/index.js',
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
  plugins: [
    new CircularCheckRspackPlugin({
      failOnError: false,
      exclude: /(ignore-circular|loader)/,
    }),
  ],
});
