const { ModuleFederationPluginV1: ModuleFederationPlugin } =
  require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  entry: './index.js',
  plugins: [
    new ModuleFederationPlugin({
      name: 'legacy_singleton_scope',
      filename: 'remoteEntry.js',
      exposes: {
        './exposed': './index.js',
      },
      shareScope: ['default'],
    }),
  ],
};
