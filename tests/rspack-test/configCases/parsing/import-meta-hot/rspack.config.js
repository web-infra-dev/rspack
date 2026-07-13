const { HotModuleReplacementPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    entry: './index.js',
    devtool: false,
    externals: {
      fs: 'node-commonjs fs',
    },
    node: {
      __filename: false,
    },
    plugins: [new HotModuleReplacementPlugin()],
  },
  {
    entry: './production.js',
    mode: 'production',
    target: 'web',
    devServer: {
      hot: true,
    },
  },
  {
    entry: './parser-options.js',
    mode: 'development',
    devtool: false,
    target: 'node',
    experiments: {
      outputModule: true,
    },
    output: {
      module: true,
      chunkFormat: 'module',
    },
    module: {
      parser: {
        javascript: {
          importMeta: {
            hot: false,
          },
        },
      },
    },
    plugins: [new HotModuleReplacementPlugin()],
  },
];
