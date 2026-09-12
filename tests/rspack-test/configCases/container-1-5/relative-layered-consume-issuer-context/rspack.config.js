const { ModuleFederationPlugin } = require('@rspack/core').container;
const { ConsumeSharedPlugin } = require('@rspack/core').sharing;

module.exports = {
  experiments: {
    layers: true,
  },
  module: {
    rules: [
      {
        test: /nested[\\/]consumer\.js$/,
        layer: 'server',
      },
    ],
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'relative_layered_host',
      manifest: true,
    }),
    new ConsumeSharedPlugin({
      enhanced: true,
      consumes: {
        './shared': {
          import: './shared',
          request: './shared',
          shareKey: 'relative-shared',
          requiredVersion: false,
          issuerLayer: 'server',
          layer: 'server',
        },
      },
    }),
  ],
};
