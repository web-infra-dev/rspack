/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'none',
  entry: './main.js',
  optimization: {
    usedExports: true,
    sideEffects: true,
    concatenateModules: true,
  },
  stats: {
    assets: true,
    modules: true,
    orphanModules: true,
    nestedModules: true,
    usedExports: true,
    reasons: true,
  },
};
