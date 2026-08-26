const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  cache: { type: 'memory' },
  experiments: {
    newCache: {
      codeGeneration: false,
      loader: true,
      minimize: false,
    },
  },
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'left' },
          },
          {
            loader: path.resolve(__dirname, 'marked-loader.js'),
            options: { name: 'marked' },
            parallel: { maxWorkers: 1 },
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'right-loader.js'),
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
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'bom-consumer' },
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'bom-producer' },
            cache: true,
          },
        ],
      },
      {
        test: /module-[ab]\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'module-id' },
            cache: true,
          },
        ],
      },
      {
        test: /(?:file|build|missing)-dependency\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'dependency' },
            cache: true,
          },
        ],
      },
      {
        test: /context-dependency\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'context-downstream' },
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'dependency' },
            cache: true,
          },
        ],
      },
      {
        test: /chain-dependency\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'chain-left' },
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'chain-right-loader.js'),
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
            loader: path.resolve(__dirname, 'overlap-owner-loader.js'),
          },
        ],
      },
      {
        test: /js-overlap-dependency\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'js-overlap-value-loader.js'),
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'overlap-owner-loader.js'),
          },
        ],
      },
    ],
  },
};
