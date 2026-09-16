import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: ['./index.js'],
  },
  plugins: [
    new rspack.ProvidePlugin({
      str: './str.js',
    }),
  ],
};
