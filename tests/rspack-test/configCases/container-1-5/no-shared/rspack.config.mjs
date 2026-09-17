import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    './container.js': 'commonjs ./container.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      exposes: ['./module'],
    }),
  ],
};
