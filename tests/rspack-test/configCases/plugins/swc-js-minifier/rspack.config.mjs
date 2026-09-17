import { SwcJsMinimizerRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: ['./index.js'],
    extract: ['./extract.js'],
    'no-extract': ['./no-extract.js'],
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new SwcJsMinimizerRspackPlugin({
      extractComments: true,
      minimizerOptions: {
        format: {
          comments: false,
        },
      },
      include: ['extract.js', 'no-extract.js'],
    }),
  ],
};
