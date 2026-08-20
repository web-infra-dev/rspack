/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
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
