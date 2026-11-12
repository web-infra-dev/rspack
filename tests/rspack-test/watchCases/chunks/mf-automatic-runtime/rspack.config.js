const rspack = require('@rspack/core');

module.exports = {
  mode: 'development',
  target: 'async-node',
  cache: true,
  incremental: true,
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: 'watch_host',
      excludeBundlerRuntime: true,
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
