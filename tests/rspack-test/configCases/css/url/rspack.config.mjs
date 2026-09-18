import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default [
  {
    target: 'web',
    mode: 'development',
    devtool: false,

    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
    output: {
      assetModuleFilename: '[name].[hash][ext][query][fragment]',
    },
    resolve: {
      alias: {
        'alias-url.png': path.resolve(import.meta.dirname, 'img.png'),
        'alias-url-1.png': false,
      },
    },
    externals: {
      'external-url.png': 'asset ./img.png',
      'external-url-2.png': 'test',
      'schema:test': "asset 'img.png'",
    },
    plugins: [new rspack.IgnorePlugin({ resourceRegExp: /ignore\.png/ })],
  },
  {
    target: 'web',
    mode: 'development',
    devtool: false,

    module: {
      parser: {
        css: {
          url: false,
        },
      },
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
    output: {
      assetModuleFilename: '[name].[hash][ext][query][fragment]',
    },
  },
];
