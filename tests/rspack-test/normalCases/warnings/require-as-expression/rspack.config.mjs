/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    parser: {
      javascript: {
        requireAsExpression: true,
      },
    },
  },
};
