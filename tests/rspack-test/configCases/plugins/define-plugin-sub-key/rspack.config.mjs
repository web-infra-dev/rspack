import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new DefinePlugin({
      'foo.bar.baz': '"test"',
    }),
  ],
};
