import { CopyRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  plugins: [
    new CopyRspackPlugin({
      patterns: [{ from: 'test.mjs' }],
    }),
  ],
};
