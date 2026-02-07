const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  target: 'async-node',
  output: {
    chunkLoading: 'async-node',
  },
  entry: './index.js',
  plugins: [
    new ModuleFederationPlugin({
      name: 'rsc_mf_disabled',
      filename: 'remoteEntry.js',
      library: { type: 'commonjs-module' },
      exposes: {
        './exposed': './exposed.js',
      },
      experiments: {
        asyncStartup: true,
        rsc: false,
      },
    }),
  ],
};
