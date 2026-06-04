// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require('@rspack/core').container;
const { resolve } = require('node:path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  output: {
    publicPath: 'PUBLIC_PATH',
    chunkFilename: '[id].js',
  },
  resolve: {
    alias: {
      'local-provided': resolve(__dirname, 'local-provided/index.js'),
    },
  },
  target: 'async-node',
  plugins: [
    new ModuleFederationPlugin({
      name: 'tree_shaking_shared_provide_only',
      filename: 'remoteEntry.js',
      library: {
        type: 'commonjs-module',
        name: 'tree_shaking_shared_provide_only',
      },
      runtimePlugins: [require.resolve('./runtime-plugin.js')],
      shared: {
        'provided-only': {
          import: './node_modules/provided-only/index.js',
          requiredVersion: '*',
          version: '1.0.0',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'local-provided': {
          import: './local-provided/index.js',
          requiredVersion: '*',
          version: '2.3.4',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
      },
    }),
  ],
};
