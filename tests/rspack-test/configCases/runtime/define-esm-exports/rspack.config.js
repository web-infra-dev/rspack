/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  output: {
    filename: 'main.js',
  },
  optimization: {
    concatenateModules: false,
    usedExports: false,
    minimize: false,
  },
};
