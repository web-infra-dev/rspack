import { rspack as webpack } from '@rspack/core';
import fs from 'node:fs';
import { join } from 'node:path';

function read(path) {
  return JSON.stringify(
    fs
      .readFileSync(join(import.meta.dirname, path), 'utf8')
      .replace(/\r\n?/g, '\n'),
  );
}

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    mode: 'production',
    entry: './index',
    output: {
      filename: '123.js',
    },
    plugins: [
      new webpack.DefinePlugin({
        VALUE: '123',
      }),
    ],
  },

  {
    mode: 'production',
    entry: './index',
    output: {
      filename: '321.js',
    },
    plugins: [
      new webpack.DefinePlugin({
        VALUE: '321',
      }),
    ],
  },

  {
    mode: 'production',
    entry: './index',
    output: {
      filename: 'both.js',
    },
    plugins: [
      new webpack.DefinePlugin({
        VALUE: webpack.DefinePlugin.runtimeValue(
          () => read('123.txt'),
          [join(import.meta.dirname, './123.txt')],
        ),
      }),
      new webpack.DefinePlugin({
        VALUE: webpack.DefinePlugin.runtimeValue(
          () => read('321.txt'),
          [join(import.meta.dirname, './321.txt')],
        ),
      }),
    ],
  },
];
