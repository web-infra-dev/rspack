const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  target: false,
  output: {
    chunkFormat: 'commonjs',
    chunkLoading: false,
  },
  node: {
    __dirname: false,
  },
  entry: './index.js',
  plugins: [
    new ModuleFederationPlugin({
      name: 'rsc_mf_target_false',
      filename: 'remoteEntry.js',
      library: { type: 'commonjs-module' },
      exposes: {
        './exposed': './exposed.js',
      },
      experiments: {
        rsc: true,
      },
    }),
  ],
};
