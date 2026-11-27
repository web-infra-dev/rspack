const path = require('path');
const { ModuleFederationPlugin } = require('@rspack/core').container;

const nodeAlias = {
  '@module-federation/node/runtimePlugin': path.resolve(
    __dirname,
    'node-runtime-plugin.js',
  ),
};

/** @type {import("@rspack/core").Configuration} */
const baseConfig = {
  target: 'node',
  output: {
    filename: '[name].js',
    uniqueName: 'mf-clear-cache-metadata',
  },
  optimization: {
    concatenateModules: false,
    chunkIds: 'named',
    moduleIds: 'named',
  },
};

module.exports = [
  {
    ...baseConfig,
    entry: {
      bundle: './index.js',
      'no-remote': './no-remote.js',
    },
    resolve: { alias: nodeAlias },
    plugins: [
      new ModuleFederationPlugin({
        name: 'host',
        remotes: {
          remoteA: 'promise Promise.resolve({ init() {}, get() {} })',
        },
        runtimePlugins: ['@module-federation/node/runtimePlugin'],
      }),
    ],
  },
  {
    ...baseConfig,
    entry: { 'plain-node': './plain-node.js' },
    output: { ...baseConfig.output, uniqueName: 'mf-clear-cache-plain-node' },
    plugins: [
      new ModuleFederationPlugin({
        name: 'host_plain',
        remotes: {
          remoteA: 'promise Promise.resolve({ init() {}, get() {} })',
        },
      }),
    ],
  },
  {
    ...baseConfig,
    target: 'web',
    entry: { browser: './browser.js' },
    performance: false,
    resolve: { alias: nodeAlias },
    output: { ...baseConfig.output, uniqueName: 'mf-clear-cache-web' },
    plugins: [
      new ModuleFederationPlugin({
        name: 'host_web',
        remotes: {
          remoteA: 'promise Promise.resolve({ init() {}, get() {} })',
        },
        runtimePlugins: ['@module-federation/node/runtimePlugin'],
      }),
    ],
  },
];
