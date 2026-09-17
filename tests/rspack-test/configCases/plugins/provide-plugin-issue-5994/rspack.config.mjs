import { ProvidePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new ProvidePlugin({
      Mod: ['./esm', 'default'],
      Def: ['./esm', 'default'],
    }),
  ],
};
