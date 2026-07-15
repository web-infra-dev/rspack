/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  devtool: false,
  output: {
    filename: 'main.js',
  },
  optimization: {
    concatenateModules: false,
    moduleIds: 'named',
    usedExports: false,
    minimize: false,
  },
};
