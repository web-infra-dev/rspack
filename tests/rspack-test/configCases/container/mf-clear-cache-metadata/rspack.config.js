const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  entry: './index.js',
  output: {
    filename: 'bundle.js',
    uniqueName: 'mf-clear-cache-metadata',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        remoteA: 'promise Promise.resolve({ init() {}, get() {} })',
      },
    }),
  ],
};
