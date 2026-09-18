/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  module: {
    parser: {
      javascript: {
        createRequire: true,
      },
    },
  },
};
