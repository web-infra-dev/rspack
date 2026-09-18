import { container } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      exposes: {
        './Button': './Button',
      },
      shared: {
        react: {
          eager: true,
        },
      },
    }),
  ],
};
