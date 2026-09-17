/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    parser: {
      javascript: {
        strictThisContextOnImports: true,
      },
    },
  },
};
