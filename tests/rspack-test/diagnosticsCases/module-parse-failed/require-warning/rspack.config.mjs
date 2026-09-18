/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    parser: {
      javascript: {
        requireAlias: true,
        requireAsExpression: true,
      },
    },
  },
};
