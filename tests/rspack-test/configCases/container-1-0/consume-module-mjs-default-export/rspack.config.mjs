import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  plugins: [
    new rspack.container.ModuleFederationPluginV1({
      shared: ['./shared-esm-pkg/index.js'],
    }),
  ],
};
