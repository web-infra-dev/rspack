/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    concatenateModules: false,
    minimize: false,
    providedExports: true,
  },
  stats: {
    modules: true,
    providedExports: true,
  },
};
