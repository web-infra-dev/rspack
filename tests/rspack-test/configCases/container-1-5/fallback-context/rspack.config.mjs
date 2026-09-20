import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './src/index.js',
  plugins: [
    new ModuleFederationPlugin({
      shared: ['./src/shared.js'],
    }),
  ],
};
