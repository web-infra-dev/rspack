/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    runtimeMode: 'rspack',
  },
  output: {
    filename: 'main.js',
    chunkFilename: '[name].js',
  },
  optimization: {
    concatenateModules: false,
  },
};
