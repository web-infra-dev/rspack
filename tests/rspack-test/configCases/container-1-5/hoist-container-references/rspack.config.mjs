import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    splitChunks: {
      chunks: 'all',
    },
    moduleIds: 'named',
  },
  plugins: [new ModuleFederationPlugin({})],
};
