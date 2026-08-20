/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  module: {
    parser: {
      javascript: {
        createRequire: true,
      },
    },
  },
  optimization: {
    concatenateModules: false,
    minimize: false,
    moduleIds: 'named',
    providedExports: true,
    usedExports: true,
  },
  stats: {
    modules: true,
    providedExports: true,
  },
};
