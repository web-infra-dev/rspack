'use strict';

/** @typedef {import("@rspack/core").GeneratorOptionsByModuleTypeKnown} GeneratorOptionsByModuleTypeKnown */

/** @type {import("@rspack/core").Configuration} */

/**
 * @param {object} options Configuration options
 * @param {boolean=} options.concatenateModules Whether to concatenate modules
 * @returns {import("@rspack/core").Configuration} Webpack configuration
 */
const getConfig = (
  { concatenateModules } = {
    concatenateModules: false,
  },
) => ({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  devtool: false,
  target: 'web',
  mode: 'development',
  optimization: {
    chunkIds: 'named',
    concatenateModules,
  },
  module: {
    rules: [
      {
        test: /module-text\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'text',
        },
      },
      {
        test: /auto-text\.css$/,
        type: 'css/auto',
        parser: {
          exportType: 'text',
        },
      },
      {
        test: /module-text-no-esm\.css$/,
        type: 'css/module',
        generator: {
          esModule: false,
        },
        parser: {
          exportType: 'text',
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
          exportType: 'text',
          namedExports: false,
        },
      },
      {
        test: /foo\.css$/,
        type: 'css/auto',
      },
      {
        test: /module-with-imports\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'text',
        },
      },
      {
        test: /imported-(base|layer)\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'text',
        },
      },
      {
        test: /parent-module-with-imports\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'text',
        },
      },
      {
        test: /text-with-stylesheet-import\.css$/,
        type: 'css/auto',
        parser: {
          exportType: 'text',
        },
      },
      {
        test: /stylesheet(?:-with-url)?\.css$/,
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
        test: /icss-export\.modules\.css$/,
        type: 'css/module',
      },
      {
        test: /icss-text\.modules\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'text',
        },
      },
      {
        test: /icss-stylesheet\.modules\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'css-style-sheet',
        },
      },
    ],
    parser: {
      css: {
        import: true,
      },
    },
  },
  experiments: {
    css: true,
  },
});
module.exports = [
  getConfig(),
  getConfig({
    concatenateModules: true,
  }),
];
