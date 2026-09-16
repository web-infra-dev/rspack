import { container } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ModuleFederationPlugin({
      remoteType: 'commonjs-module',
      remotes: {
        containerB: '../1-container-full/container.js',
      },
      shared: ['@rspack/mocked-react'],
    }),
  ],
};
