const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
const commonConfig = {
  target: 'node14',
  mode: 'development',
  optimization: {
    minimize: false,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    filename: '[name].js',
  },
};

module.exports = [
  {
    ...commonConfig,
    entry: {
      main: './index.js',
    },
    output: {
      ...commonConfig.output,
      uniqueName: 'mf-ssr-clear-cache-host',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'host',
        remotes: {
          remoteA: {
            external:
              'mf_ssr_clear_cache_remote@http://localhost:3001/remoteEntry.js',
          },
        },
        runtimePlugins: [require.resolve('./runtime-plugin.js')],
      }),
    ],
  },
  {
    ...commonConfig,
    entry: {
      remote: './remoteEntryRuntime.js',
    },
    output: {
      ...commonConfig.output,
      filename: 'remote-[name].js',
      chunkFilename: 'remote-[id].js',
      uniqueName: 'mf-ssr-clear-cache-remote',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'mf_ssr_clear_cache_remote',
        filename: 'remoteEntry.js',
        library: {
          type: 'commonjs-module',
          name: 'mf_ssr_clear_cache_remote',
        },
        exposes: {
          './A': './remoteA.js',
          './B': './remoteB.js',
        },
        runtimePlugins: [require.resolve('./provider-runtime-plugin.js')],
      }),
    ],
  },
];
