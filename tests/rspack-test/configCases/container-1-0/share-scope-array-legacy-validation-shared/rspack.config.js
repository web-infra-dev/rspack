const { ModuleFederationPluginV1: ModuleFederationPlugin } =
  require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  entry: './index.js',
  plugins: [
    new ModuleFederationPlugin({
      name: 'legacy_multi_scope_validation_shared',
      shared: {
        react: {
          import: 'react',
          shareScope: ['default', 'ssr'],
        },
      },
    }),
  ],
};
