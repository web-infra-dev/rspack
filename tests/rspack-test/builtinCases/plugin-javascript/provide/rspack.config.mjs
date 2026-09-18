import { ProvidePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ProvidePlugin({
      process: ['./process.js'],
      name: ['./name.js'],
    }),
  ],
};
