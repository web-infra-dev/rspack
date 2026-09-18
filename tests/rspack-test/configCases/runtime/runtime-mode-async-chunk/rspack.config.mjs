/** @type {import("@rspack/core").Configuration} */
export default {
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
