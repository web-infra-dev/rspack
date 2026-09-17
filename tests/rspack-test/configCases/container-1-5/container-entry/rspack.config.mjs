import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    './container-file.js': 'commonjs ./container-file.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container-file.js',
      library: {
        type: 'commonjs-module',
      },
      exposes: {
        './test': './test',
        './test2': ['./init-module', './test2'],
        '.': './main',
      },
    }),
  ],
};
