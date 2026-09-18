/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  mode: 'production',
  module: {
    parser: {
      javascript: {
        exportsPresence: 'auto',
      },
    },
  },
  stats: {
    assets: true,
    modules: true,
    warningsSpace: 0,
    warnings: true,
  },
};
