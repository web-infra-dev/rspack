'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  devtool: false,
  target: 'web',
  mode: 'development',
  output: {
    cssFilename: 'bundle0.css',
  },
  module: {
    generator: {
      'css/auto': {
        localIdentName: '[name]_module_css-[local]',
      },
      'css/module': {
        localIdentName: '[name]_module_css-[local]',
      },
    },
    rules: [
      {
        test: /module-text\.css$/,
        type: 'css/module',
      },
      {
        test: /auto-text\.css$/,
        type: 'css/auto',
      },
      {
        test: /module-text-no-esm\.css$/,
        type: 'css/module',
        generator: {
          esModule: false,
        },
        parser: {
          namedExports: false,
        },
      },
      {
        test: /auto-text-no-esm\.css$/,
        type: 'css/auto',
        generator: {
          esModule: false,
        },
        parser: {
          namedExports: false,
        },
      },
      {
        test: /stylesheet\.css$/,
        type: 'css/auto',
        parser: {
          exportType: 'css-style-sheet',
        },
      },
      {
        test: /module-stylesheet\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'css-style-sheet',
        },
      },
      {
        test: /style-for-inject\.css$/,
        type: 'css/auto',
        parser: {
          exportType: 'style',
        },
      },
    ],
    parser: {
      css: {
        exportType: 'text',
      },
    },
  },
};
