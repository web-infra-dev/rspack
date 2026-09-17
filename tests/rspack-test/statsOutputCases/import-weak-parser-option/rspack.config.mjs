/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: {
    entry: './entry',
  },
  module: {
    parser: {
      javascript: {
        dynamicImportMode: 'weak',
      },
    },
  },
  stats: {
    assets: true,
    modules: true,
  },
};
