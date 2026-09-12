const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    filename: '[name].js',
    uniqueName: 'share-strategy',
  },
  plugins: [
    new ModuleFederationPlugin({
      shareStrategy: 'loaded-first',
      remotes: {
        lazy: `promise (globalThis.__loadedFirstRemoteLoads = (globalThis.__loadedFirstRemoteLoads || 0) + 1, Promise.resolve({ init() {}, get() { return () => 'remote'; } }))`,
      },
      shared: {
        react: {
          version: false,
          requiredVersion: false,
          singleton: true,
          strictVersion: false,
          version: '0.1.2',
        },
      },
    }),
  ],
};
