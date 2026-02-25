// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require("@rspack/core").container;

/** @type {import("@rspack/core").Configuration} */

const commonConfig = {
  output: {
    ignoreBrowserWarnings: true
  },
  optimization: {
    minimize: false,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    publicPath: '/',
    chunkFilename: '[id].js',
  },
  target: 'async-node',
};

module.exports = [
  {
    ...commonConfig,
    entry: './index.js',
    plugins: [
      new ModuleFederationPlugin({
        name: 'remote_array_share_scope_host',
        remotes: {
          'remote-alias': {
            external: 'remote_array_share_scope_provider@http://localhost:3001/remoteEntry.js',
            shareScope: ['scope1', 'scope3']
          }
        },
        runtimePlugins: [require.resolve('./runtime-plugin.js')],
        shared: {
          '@scope-sc/ui-lib': {
            requiredVersion: '*',
            shareScope: 'scope1',
          },
          '@scope-sc/ui-lib2': {
            requiredVersion: '*',
            shareScope: 'scope3',
          },
          '@scope-sc/ui-lib3': {
            requiredVersion: '*',
          },
        },
      }),
    ]
  },
  {
    ...commonConfig,
    entry: {
      output:'./Expose.js'
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'remote_array_share_scope_provider',
        manifest: true,
        filename: 'remoteEntry.js',
        shareScope: ['scope1', 'scope2', 'scope3'],
        library: {
          type: 'commonjs-module',
          name: 'remote_array_share_scope_provider',
        },
        exposes: {
          './Expose': './Expose.js',
        },
        runtimePlugins: [require.resolve('./runtime-plugin.js')],
        shared: {
          '@scope-sc/ui-lib': {
            requiredVersion: '*',
            shareScope: 'scope1',
          },
          '@scope-sc/ui-lib2': {
            requiredVersion: '*',
            shareScope: 'scope3',
          },
          '@scope-sc/ui-lib3': {
            requiredVersion: '*',
          },
        },
      }),
    ]
  },
]