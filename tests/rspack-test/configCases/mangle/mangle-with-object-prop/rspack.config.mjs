import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default [
  {
    output: {
      filename: 'deterministic.js',
    },
    optimization: {
      mangleExports: true,
      usedExports: true,
      providedExports: true,
      inlineExports: false,
    },
    plugins: [
      new DefinePlugin({
        OPTIMIZATION: JSON.stringify('deterministic'),
      }),
    ],
  },
  {
    output: {
      filename: 'size.js',
    },
    optimization: {
      mangleExports: 'size',
      usedExports: true,
      providedExports: true,
      inlineExports: false,
    },
    plugins: [
      new DefinePlugin({
        OPTIMIZATION: JSON.stringify('size'),
      }),
    ],
  },
];
