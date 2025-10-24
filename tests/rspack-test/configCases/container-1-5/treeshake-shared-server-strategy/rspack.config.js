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
      name: 'treeshake_share',
      manifest: true,
      filename: 'remoteEntry.js',
      library: {
        type: 'commonjs-module',
        name: 'treeshake_share',
      },
      exposes: {
        './App': './App.js',
      },
      runtimePlugins: [require.resolve('./runtime-plugin.js')],
      shared: {
        'ui-lib': {
          requiredVersion: '*',
          treeshake: {
            strategy: 'server',
          },
        },
        'ui-lib-es': {
          requiredVersion: '*',
          treeshake: {
            strategy: 'server',
          },
        },
        'ui-lib-dynamic-specific-export': {
          requiredVersion: '*',
          treeshake: {
            strategy: 'server',
          },
        },
        'ui-lib-dynamic-default-export': {
          requiredVersion: '*',
          treeshake: {
            strategy: 'server',
          },
        },
        'ui-lib-side-effect': {
          requiredVersion: '*',
          treeshake: {
            strategy: 'server',
          },
        },
      },
    }),
  ],
};
