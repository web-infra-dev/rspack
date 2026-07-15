/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  output: {
    filename: 'main.js',
    chunkFilename: '[name].js',
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
    mangleExports: false,
  },
};
