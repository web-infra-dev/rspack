const { ModuleFederationPlugin } = require('@rspack/core').container;

module.exports = {
  target: 'async-node',
  experiments: { layers: true },
  optimization: { concatenateModules: false },
  plugins: [
    new ModuleFederationPlugin({
      implementation:
        require.resolve('@module-federation/runtime-tools-layers'),
      name: 'transitive_requirements',
      filename: 'container.js',
      manifest: true,
      exposes: {
        './first': { import: './exposed.js', layer: 'server' },
        './second': { import: './exposed.js', layer: 'client' },
        './boundary': 'boundary-only',
        './unrelated': './unrelated.js',
      },
      shared: {
        'boundary-only': {
          import: './boundary.js',
          shareScope: 'tenant',
          layer: 'shared',
          requiredVersion: false,
        },
        'remote-only': {
          import: false,
          shareScope: 'tenant',
          layer: 'shared',
          requiredVersion: false,
        },
      },
    }),
  ],
};
