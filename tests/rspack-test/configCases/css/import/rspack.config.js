'use strict';

const path = require('path');
const { rspack } = require('@rspack/core');

// Match webpack's built-in CSS rules: CSS imports are resolved as fully
// specified, so extension-less requests like "./no-extension-in-request" do not
// fall back to "./no-extension-in-request.css".
const cssResolve = {
  fullySpecified: true,
  preferRelative: true,
};

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  mode: 'development',

  resolve: {
    alias: {
      '/style2.css': path.resolve(__dirname, './style2.css'),
      '/alias.css': false,
    },
    byDependency: {
      'css-import': {
        conditionNames: ['custom-name', '...'],
        extensions: ['.mycss', '...'],
      },
    },
  },
  module: {
    rules: [
      {
        test: /\.mycss$/,
        loader: './string-loader',
        type: 'css/global',
      },
      {
        test: /\.less$/,
        use: ['./remove-source-map-url-loader', 'less-loader'],
        type: 'css/global',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
        resolve: cssResolve,
      },
      {
        mimetype: 'text/css',
        type: 'css/auto',
        resolve: cssResolve,
      },
    ],
  },
  externals: {
    path: 'node-commonjs path',
    'external-1.css': 'css-import external-1.css',
    'external-2.css': 'css-import external-2.css',
    'external-3.css': 'css-import external-3.css',
    'external-4.css': 'css-import external-4.css',
    'external-5.css': 'css-import external-5.css',
    'external-6.css': 'css-import external-6.css',
    'external-7.css': 'css-import external-7.css',
    'external-8.css': 'css-import external-8.css',
    'external-9.css': 'css-import external-9.css',
    'external-10.css': 'css-import external-10.css',
    'external-11.css': 'css-import external-11.css',
    'external-12.css': 'css-import external-12.css',
    'external-13.css': 'css-import external-13.css',
    'external-14.css': 'css-import external-14.css',
  },
  plugins: [new rspack.IgnorePlugin({ resourceRegExp: /ignore\.css/ })],
};
