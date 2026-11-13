const { ModuleFederationPlugin } = require('@rspack/core').container;
const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = ['webpack', 'rspack'].flatMap((runtimeMode) =>
  ['async-node', 'node'].map((target) => ({
    target,
    experiments: { runtimeMode },
    entry: {
      main: './index.js',
    },
    output: {
      filename: `[name].${runtimeMode}.${target}.js`,
      chunkFilename: `[id].${runtimeMode}.${target}.js`,
      uniqueName: `mf-ssr-clear-cache-${runtimeMode}-${target}-chunk`,
    },
    plugins: [
      new DefinePlugin({
        __TEST_REMOTE_SERVER_KEY__: JSON.stringify(
          `__mfSsrClearCache_${runtimeMode}_${target}`,
        ),
      }),
      new ModuleFederationPlugin({
        name: `host_${runtimeMode}_${target}_chunk`,
        implementation: process.env.RSPACK_MF_RUNTIME_TOOLS_IMPLEMENTATION,
        runtimePlugins: [
          process.env.RSPACK_MF_NODE_RUNTIME_PLUGIN ||
            require.resolve('@module-federation/node/runtimePlugin'),
        ],
        remotes: {
          remoteA: `promise Promise.resolve(globalThis[${JSON.stringify(`__mfSsrClearCache_${runtimeMode}_${target}`)}].loadRemoteEntry())`,
        },
      }),
    ],
  })),
);
