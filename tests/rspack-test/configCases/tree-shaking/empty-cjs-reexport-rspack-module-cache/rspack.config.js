/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  experiments: {
    runtimeMode: 'rspack',
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
