const path = require('path');
const { rspack } = require('@rspack/core');

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
  optimization: {
    inlineExports: true,
  },
  resolve: {
    extensions: ['.ts', '...'],
  },
  plugins: [new rspack.CssExtractRspackPlugin()],
  module: {
    rules: [
      {
        test: /style\.css$/,
        type: 'javascript/auto',
        use: [
          {
            loader: rspack.CssExtractRspackPlugin.loader,
            cache: true,
          },
          'css-loader',
        ],
      },
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
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'left' },
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'source-map-consumer' },
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'source-map-producer' },
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
        test: /module-[ab]\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'module-id' },
            cache: true,
          },
        ],
      },
    ],
  },
};
