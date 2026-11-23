const rspack = require('@rspack/core');

module.exports = {
  mode: 'development',
  target: 'async-node',
  cache: true,
  incremental: true,
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: 'watch_host',
      removeUnusedRuntime: true,
      remotes: {
        remote:
          "promise Promise.resolve({ get: () => Promise.resolve(() => 'remote'), init: () => {} })",
      },
    }),
  ],
};
