import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: false,
  target: 'web',
  plugins: [
    new DefinePlugin({
      _VALUE_: {
        _DEFINED_: 1,
        _PROP_: {
          _DEFINED_: 2,
        },
      },
    }),
  ],
};
