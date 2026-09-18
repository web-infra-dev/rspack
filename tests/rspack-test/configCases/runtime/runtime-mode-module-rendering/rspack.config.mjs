/** @type {import("@rspack/core").Configuration} */
export default {
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
