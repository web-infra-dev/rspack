import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
    uniqueName: 'share-strategy',
  },
  plugins: [
    new ModuleFederationPlugin({
      shareStrategy: 'loaded-first',
      shared: {
        react: {
          version: '0.1.2',
          requiredVersion: false,
          singleton: true,
          strictVersion: false,
        },
      },
    }),
  ],
};
