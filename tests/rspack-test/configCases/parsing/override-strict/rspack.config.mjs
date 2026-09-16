/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    mode: 'production',
    entry: ['./strict'],
    module: {
      parser: {
        javascript: {
          overrideStrict: 'strict',
        },
      },
    },
  },
  {
    mode: 'production',
    entry: ['./strict'],
    module: {
      parser: {
        javascript: {
          overrideStrict: 'strict',
        },
      },
    },
  },
];
