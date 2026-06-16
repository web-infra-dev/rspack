/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.mjs',
  experiments: {
    runtimeMode: 'rspack',
  },
  output: {
    filename: 'main.js',
  },
  optimization: {
    concatenateModules: false,
  },
};
