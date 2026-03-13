const { ModuleFederationPluginV1: ModuleFederationPlugin } =
  require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  entry: './index.js',
  plugins: [
    new ModuleFederationPlugin({
      name: 'legacy_multi_scope_validation_remotes',
      remotes: {
        remote: {
          external: 'remote@http://localhost:3000/remoteEntry.js',
          shareScope: ['default', 'ssr'],
        },
      },
    }),
  ],
};
