/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  cache: {
    type: 'persistent',
  },
  optimization: {
    concatenateModules: false,
    minimize: false,
    moduleIds: 'named',
    providedExports: true,
    usedExports: true,
  },
};
