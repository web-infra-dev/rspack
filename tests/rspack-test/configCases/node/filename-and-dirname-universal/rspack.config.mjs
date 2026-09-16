/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    name: 'web',
    target: ['node', 'web'],
    output: {
      module: true,
    },
  },
  {
    name: 'node',
    target: ['node', 'web'],
    output: {
      module: true,
    },
  },
  {
    name: 'web',
    devtool: 'eval',
    target: ['node', 'web'],
    output: {
      module: true,
    },
  },
  {
    name: 'node',
    devtool: 'eval',
    target: ['node', 'web'],
    output: {
      module: true,
    },
  },
];
