import path from 'node:path';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

const sharedOptions = {
  eager: true,
  requiredVersion: false,
  version: false,
};

const aliases = {
  alpha: path.resolve(import.meta.dirname, 'alpha.js'),
  beta: path.resolve(import.meta.dirname, 'beta.js'),
  delta: path.resolve(import.meta.dirname, 'delta.js'),
  gamma: path.resolve(import.meta.dirname, 'gamma.js'),
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

export default [
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
