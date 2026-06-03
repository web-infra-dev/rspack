// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  output: {
    publicPath: 'PUBLIC_PATH',
    chunkFilename: '[id].js',
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
      },
    }),
  ],
};
