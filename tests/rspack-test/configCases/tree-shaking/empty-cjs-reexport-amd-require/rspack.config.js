/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  amd: {},
  optimization: {
    concatenateModules: false,
    minimize: false,
    providedExports: true,
    usedExports: true,
  },
  stats: {
    modules: true,
    providedExports: true,
  },
};
