/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    concatenateModules: false,
    minimize: false,
  },
};
