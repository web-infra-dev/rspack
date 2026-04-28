'use strict';

const path = require('path');

/** @typedef {import("@rspack/core").PathData} PathData */

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /\.less$/,
        use: ['./remove-source-map-url-loader', 'less-loader'],
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-1$/,
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashDigest: 'base64url',
          localIdentHashDigestLength: 5,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-2$/,
        generator: {
          localIdentName: '-1[local]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-3$/,
        generator: {
          localIdentName: '--[local]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-4$/,
        generator: {
          localIdentName: '__[local]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-5$/,
        generator: {
          localIdentName: '[local]--[fullhash]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-6$/,
        generator: {
          localIdentName: '😀- -[local]',
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-7$/,
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashFunction: 'sha256',
          localIdentHashDigestLength: 10,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-8$/,
        generator: {
          localIdentName: '[name]--[local]--[fullhash]',
          localIdentHashFunction: 'xxhash64',
          localIdentHashDigestLength: 6,
        },
      },
      {
        test: /.css$/,
        resourceQuery: /\?local-ident-name-9$/,
        generator: {
          /**
           * @param {PathData} pathData path data
           * @returns {string} local ident name
           */
          localIdentName: (pathData) =>
            `prefix-${pathData.filename}---${pathData.local}---${pathData.hash}-postfix`,
        },
      },
    ],
  },
  resolve: {
    alias: {
      '~test': path.resolve(__dirname, 'node_modules/test'),
    },
  },
};
