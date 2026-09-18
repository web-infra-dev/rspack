import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new rspack.DefinePlugin({
      PROPERTY: JSON.stringify('foo'),
    }),
  ],
};
