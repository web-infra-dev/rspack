import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: ['./index.js'],
    },
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
  optimization: {
    sideEffects: true,
  },
};
