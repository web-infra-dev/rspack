import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index.js',
  output: {
    filename: '[name]_bundle.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      exposes: {
        './entry': {
          import: './entry',
          name: 'custom-entry',
        },
      },
    }),
  ],
};
