const { ModuleFederationPlugin } = require('@rspack/core').container;
const path = require('path');

module.exports = {
  target: 'async-node',
  experiments: { layers: true },
  resolve: { alias: { 'second-alias': path.resolve(__dirname, 'second.js') } },
  module: {
    rules: [
      { test: /second\.js$/, layer: 'server' },
      { test: /third\.js$/, layer: 'client' },
    ],
  },
  optimization: { concatenateModules: false },
  plugins: [
    new ModuleFederationPlugin({
      name: 'manifest_import_usage',
      filename: 'container.js',
      manifest: true,
      exposes: {
        './multi': { import: ['./plain.js', 'second-alias'] },
        './other': './third.js',
        './plain': './plain.js',
      },
      shared: { shared: { requiredVersion: false } },
    }),
  ],
};
