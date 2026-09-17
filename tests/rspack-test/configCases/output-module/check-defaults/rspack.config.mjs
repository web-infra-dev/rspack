/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    output: {
      module: true,
    },
    devtool: false,
    target: 'web',
  },
  {
    output: {
      module: true,
    },
    devtool: false,
    target: 'node10',
  },
];
