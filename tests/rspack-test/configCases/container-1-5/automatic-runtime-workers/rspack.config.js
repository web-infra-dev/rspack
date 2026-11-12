const rspack = require('@rspack/core');

module.exports = {
  target: 'async-node',
  mode: 'development',
  devtool: false,
  optimization: { minimize: false },
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: 'worker_host',
      shareStrategy: 'loaded-first',
      remotes: {
        remote:
          "promise Promise.resolve({ get: () => Promise.resolve(() => 'remote value'), init: () => {} })",
      },
      shared: {
        'shared-lib': {
          import: './shared.js',
          version: '1.0.0',
          requiredVersion: false,
        },
      },
    }),
  ],
};
