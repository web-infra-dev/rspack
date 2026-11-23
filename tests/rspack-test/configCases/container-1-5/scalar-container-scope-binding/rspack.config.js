const { ModuleFederationPlugin } = require('@rspack/core').container;

module.exports = {
  target: 'async-node',
  externals: { './remoteEntry.js': 'commonjs ./remoteEntry.js' },
  plugins: [
    new ModuleFederationPlugin({
      name: 'scalar-container-scope-binding',
      filename: 'remoteEntry.js',
      library: { type: 'commonjs-module' },
      shareScope: 'container-custom',
      exposes: { './widget': './widget.js' },
      shared: { lib: { import: false, requiredVersion: false } },
    }),
  ],
};
