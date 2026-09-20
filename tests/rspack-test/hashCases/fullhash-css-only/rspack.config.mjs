import path from 'node:path';
import { rspack } from '@rspack/core';

function builtinConfig(subpath) {
  return {
    mode: 'production',
    entry: './index.js',
    context: path.resolve(import.meta.dirname, subpath),
    output: {
      path: path.resolve(import.meta.dirname, `dist/builtin-${subpath}`),
      filename: 'main.[fullhash].js',
      cssFilename: 'main.[contenthash].css',
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
  };
}

function extractConfig(subpath) {
  return {
    mode: 'production',
    entry: './index.js',
    context: path.resolve(import.meta.dirname, subpath),
    output: {
      path: path.resolve(import.meta.dirname, `dist/extract-${subpath}`),
      filename: 'main.[fullhash].js',
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
          type: 'javascript/auto',
        },
      ],
    },
    plugins: [
      new rspack.CssExtractRspackPlugin({
        filename: 'main.[contenthash].css',
      }),
    ],
    optimization: {
      realContentHash: true,
    },
  };
}

/** @type {import("@rspack/core").Configuration[]} */
export default [
  builtinConfig('version0'),
  builtinConfig('version1'),
  extractConfig('version0'),
  extractConfig('version1'),
];
