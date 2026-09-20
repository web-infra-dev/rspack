import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    parser: {
      'css/auto': {
        namedExports: false,
      },
    },
    rules: [
      {
        test: /\.module\.css/,
        type: 'css/auto',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
