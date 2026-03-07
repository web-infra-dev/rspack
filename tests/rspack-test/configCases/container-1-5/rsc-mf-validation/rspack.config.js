const { ModuleFederationPlugin } = require('@rspack/core').container;

const baseMfOptions = {
  name: 'rsc_mf_validation',
  filename: 'remoteEntry.js',
  library: { type: 'commonjs-module' },
  exposes: {
    './exposed': './index.js',
  },
};

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  name: 'missing-rsc-mf-requirements',
  mode: 'development',
  target: 'node',
  entry: './index.js',
  plugins: [
    new ModuleFederationPlugin({
      ...baseMfOptions,
      experiments: {
        rsc: true,
      },
    }),
  ],
};
