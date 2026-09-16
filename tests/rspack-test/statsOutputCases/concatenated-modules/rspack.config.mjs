/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index',
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  stats: {
    assets: true,
    modules: true,
  },
};
