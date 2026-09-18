/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  target: 'node',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    minimize: false,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
  },
};
