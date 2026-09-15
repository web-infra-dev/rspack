/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    outputModule: true,
    runtimeMode: 'rspack',
  },
  output: {
    filename: 'main.mjs',
    module: true,
  },
  optimization: {
    concatenateModules: false,
    usedExports: false,
  },
  target: 'es2020',
};
