const { ModuleFederationPluginV1: ModuleFederationPlugin } =
  require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  entry: './index.js',
  plugins: [
    new ModuleFederationPlugin({
      name: 'legacy_multi_scope_validation',
      filename: 'remoteEntry.js',
      exposes: {
        './exposed': './index.js',
      },
      shareScope: ['default', 'ssr'],
    }),
  ],
};
