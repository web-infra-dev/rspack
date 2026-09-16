import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration[]} */
export default {
  output: {
    filename: '[name].js',
    uniqueName: 'provide-sharing-extra-data',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container-provide-sharing-extra-data',
      shared: {
        react: {
          version: false,
          requiredVersion: false,
          singleton: true,
          strictVersion: false,
          version: '0.1.2',
        },
      },
    }),
  ],
};
