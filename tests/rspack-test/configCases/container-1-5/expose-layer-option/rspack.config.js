const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  experiments: {
    layers: true,
  },
  externals: {
    './container-file.js': 'commonjs ./container-file.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'expose_layer_option',
      filename: 'container-file.js',
      library: { type: 'commonjs-module' },
      manifest: false,
      exposes: {
        './server': { import: './module.js', layer: 'server' },
        './client': { import: './module.js', layer: 'client' },
        './empty': { import: './module.js', layer: '' },
        './default': { import: './module.js' },
      },
    }),
  ],
};
