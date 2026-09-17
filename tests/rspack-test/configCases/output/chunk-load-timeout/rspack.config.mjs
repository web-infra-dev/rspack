/** @type {import("@rspack/core").Configuration} */
export default [
  {
    entry: './a',
    target: 'web',
    output: {
      filename: 'a.js',
      chunkLoadTimeout: 1234000,
    },
  },
  {
    entry: './index',
  },
];
