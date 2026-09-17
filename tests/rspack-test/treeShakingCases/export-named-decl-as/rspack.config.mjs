import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: ['./src/index.js'],
    },
  },
  optimization: {
    sideEffects: true,
  },
  plugins: [
    new DefinePlugin({
      'process.env.NODE_ENV': "'development'",
    }),
  ],
};
