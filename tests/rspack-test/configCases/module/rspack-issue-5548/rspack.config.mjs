/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  optimization: {
    minimize: false,
  },
  module: {
    parser: {
      javascript: {
        dynamicImportMode: 'eager',
      },
    },
  },
};
