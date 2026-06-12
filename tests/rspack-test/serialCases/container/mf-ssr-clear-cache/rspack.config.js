const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node14',
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
    uniqueName: 'mf-ssr-clear-cache',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        remoteA:
          'promise Promise.resolve(globalThis.__mfSsrClearCacheHarness.loadRemoteEntry())',
      },
    }),
  ],
};
