// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */

module.exports = {
	entry: './index.js',
	output: {
		ignoreBrowserWarnings: true
	},
  optimization: {
    minimize: true,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    publicPath: '/',
    chunkFilename: '[id].js',
  },
  target: 'async-node',
  plugins: [
    new ModuleFederationPlugin({
      name: 'tree_shaking_share_server',
      manifest: true,
      filename: 'remoteEntry.js',
      library: {
        type: 'commonjs-module',
        name: 'tree_shaking_share',
      },
      exposes: {
        './App': './App.js',
      },
      runtimePlugins: [require.resolve('./runtime-plugin.js')],
      shared: {
        'ui-lib': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'server-calc',
          },
        },
        'ui-lib-es': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'server-calc',
          },
        },
        'ui-lib-dynamic-specific-export': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'server-calc',
          },
        },
        'ui-lib-dynamic-default-export': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'server-calc',
          },
        },
        'ui-lib-side-effect': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'server-calc',
          },
        },
      },
    }),
  ],
};
