import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    strictModuleExceptionHandling: true,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      library: { type: 'commonjs-module' },
      filename: 'container.js',
      exposes: ['./module'],
      remotes: {
        remote: './container.js',
        invalid: './invalid.js',
      },
    }),
  ],
};
