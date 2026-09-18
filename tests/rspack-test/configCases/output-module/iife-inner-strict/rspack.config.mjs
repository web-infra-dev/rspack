/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    entry: ['./index.mjs'],
    output: {
      module: false,
    },
  },
  {
    name: 'test-output',
    entry: './test.js',
    output: {
      filename: 'test.js',
    },
  },
];
