/** @type {import("@rspack/core").Configuration} */
export default {
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
