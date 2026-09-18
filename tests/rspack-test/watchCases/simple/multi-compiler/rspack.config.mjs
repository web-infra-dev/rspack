/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    name: 'changing',
    entry: './index.js',
    output: {
      filename: './bundle.js',
    },
  },
  {
    name: 'static',
    entry: './static-file.js',
    output: {
      filename: './static.js',
    },
  },
];
