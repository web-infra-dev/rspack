const { ModuleFederationPlugin } = require('@rspack/core').container;

const mfImplementation = process.env.RSPACK_MF_RUNTIME_TOOLS_IMPLEMENTATION;
const remoteVersions = ['v1', 'v2', 'v3', 'v4'];

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

function createRemoteConfig(version) {
  const projectName = `remote-project-${version}`;
  return {
    ...commonConfig,
    entry: {
      [projectName]: `./remote-projects/${version}/remoteEntryRuntime.js`,
    },
    output: {
      ...commonConfig.output,
      filename: `${projectName}/[name].js`,
      chunkFilename: `${projectName}/[id].js`,
      uniqueName: `mf-ssr-clear-cache-remote-${version}`,
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'mf_ssr_clear_cache_remote',
        implementation: mfImplementation,
        filename: `${projectName}/remoteEntry.js`,
        library: {
          type: 'commonjs-module',
          name: 'mf_ssr_clear_cache_remote',
        },
        exposes: {
          './A': `./remote-projects/${version}/remoteA.js`,
          './B': `./remote-projects/${version}/remoteB.js`,
        },
        runtimePlugins: [require.resolve('./provider-runtime-plugin.js')],
      }),
    ],
  };
}

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
        implementation: mfImplementation,
        remotes: {
          remoteA: {
            external:
              'mf_ssr_clear_cache_remote@http://localhost:3001/remote-project-v1/remoteEntry.js',
          },
        },
        runtimePlugins: [require.resolve('./runtime-plugin.js')],
      }),
    ],
  },
  ...remoteVersions.map(createRemoteConfig),
];
