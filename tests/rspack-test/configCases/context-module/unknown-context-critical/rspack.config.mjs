/** @type {import("@rspack/core").Configuration} */
export default {
  amd: false,
  module: {
    parser: {
      javascript: {
        unknownContextCritical: true,
        requireAlias: true,
        requireAsExpression: true,
      },
    },
  },
};
