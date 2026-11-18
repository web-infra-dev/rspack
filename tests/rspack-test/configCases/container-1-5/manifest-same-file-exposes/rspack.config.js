const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  experiments: { layers: true },
  externals: {
    '@rspack/core': 'node-commonjs @rspack/core',
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
    './container-file.js': 'commonjs ./container-file.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'same_file_exposes',
      filename: 'container-file.js',
      library: { type: 'commonjs-module' },
      manifest: true,
      exposes: {
        './server': { import: './module.js', layer: 'server', name: 'server' },
        './client': { import: './module.js', layer: 'client', name: 'client' },
      },
    }),
  ],
};
