/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    runtimeMode: 'rspack',
  },
  output: {
    filename: 'main.js',
  },
  optimization: {
    concatenateModules: false,
    usedExports: false,
  },
};
