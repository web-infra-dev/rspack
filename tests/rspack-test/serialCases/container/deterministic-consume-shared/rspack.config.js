const path = require('path');
const { ModuleFederationPlugin } = require('@rspack/core').container;

const sharedOptions = {
  eager: true,
  requiredVersion: false,
  version: false,
};

const aliases = {
  alpha: path.resolve(__dirname, 'alpha.js'),
  beta: path.resolve(__dirname, 'beta.js'),
  delta: path.resolve(__dirname, 'delta.js'),
  gamma: path.resolve(__dirname, 'gamma.js'),
};

function createConfig(name, shared) {
  return {
    entry: './index.js',
    output: {
      filename: `${name}.js`,
      uniqueName: name,
    },
    resolve: {
      alias: aliases,
    },
    optimization: {
      chunkIds: 'named',
      moduleIds: 'named',
    },
    plugins: [
      new ModuleFederationPlugin({
        runtime: false,
        name,
        filename: `${name}-container.js`,
        library: { type: 'commonjs-module' },
        shared,
      }),
    ],
  };
}

module.exports = [
  createConfig('forward', {
    alpha: sharedOptions,
    beta: sharedOptions,
    delta: sharedOptions,
    gamma: sharedOptions,
  }),
  createConfig('reverse', {
    gamma: sharedOptions,
    delta: sharedOptions,
    beta: sharedOptions,
    alpha: sharedOptions,
  }),
];
