import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
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
});
