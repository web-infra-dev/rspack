import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  cache: { type: 'memory' },
  experiments: {
    newCache: {
      codeGeneration: false,
      loader: true,
      minimize: false,
    },
  },
  optimization: {
    inlineExports: true,
  },
  resolve: {
    extensions: ['.ts', '...'],
  },
  module: {
    rules: [
      {
        test: /enum\.ts$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              collectTypeScriptInfo: {
                exportedEnum: true,
              },
            },
            cache: true,
          },
        ],
      },
      {
        test: /value\.js$/,
        use: [
          {
            loader: path.resolve(import.meta.dirname, 'loader.mjs'),
            options: { name: 'left' },
          },
          {
            loader: path.resolve(import.meta.dirname, 'marked-loader.mjs'),
            options: { name: 'marked' },
            parallel: { maxWorkers: 1 },
            cache: true,
          },
          {
            loader: path.resolve(import.meta.dirname, 'right-loader.mjs'),
            options: { name: 'right' },
            parallel: { maxWorkers: 1 },
            cache: true,
          },
        ],
      },
      {
        test: /bom\.js$/,
        use: [
          {
            loader: path.resolve(import.meta.dirname, 'loader.mjs'),
            options: { name: 'bom-consumer' },
          },
          {
            loader: path.resolve(import.meta.dirname, 'loader.mjs'),
            options: { name: 'bom-producer' },
            cache: true,
          },
        ],
      },
      {
        test: /module-[ab]\.js$/,
        use: [
          {
            loader: path.resolve(import.meta.dirname, 'loader.mjs'),
            options: { name: 'module-id' },
            cache: true,
          },
        ],
      },
      {
        test: /(?:file|build|missing)-dependency\.js$/,
        use: [
          {
            loader: path.resolve(import.meta.dirname, 'loader.mjs'),
            options: { name: 'dependency' },
            cache: true,
          },
        ],
      },
      {
        test: /context-dependency\.js$/,
        use: [
          {
            loader: path.resolve(import.meta.dirname, 'loader.mjs'),
            options: { name: 'context-downstream' },
            cache: true,
          },
          {
            loader: path.resolve(import.meta.dirname, 'loader.mjs'),
            options: { name: 'dependency' },
            cache: true,
          },
        ],
      },
      {
        test: /chain-dependency\.js$/,
        use: [
          {
            loader: path.resolve(import.meta.dirname, 'loader.mjs'),
            options: { name: 'chain-left' },
            cache: true,
          },
          {
            loader: path.resolve(import.meta.dirname, 'chain-right-loader.mjs'),
            cache: true,
          },
        ],
      },
      {
        test: /[/\\]overlap-dependency\.js$/,
        use: [
          {
            loader: 'builtin:test-dependency-loader',
            cache: true,
          },
          {
            loader: path.resolve(
              import.meta.dirname,
              'overlap-owner-loader.mjs',
            ),
          },
        ],
      },
      {
        test: /js-overlap-dependency\.js$/,
        use: [
          {
            loader: path.resolve(
              import.meta.dirname,
              'js-overlap-value-loader.mjs',
            ),
            cache: true,
          },
          {
            loader: path.resolve(
              import.meta.dirname,
              'overlap-owner-loader.mjs',
            ),
          },
        ],
      },
    ],
  },
};
