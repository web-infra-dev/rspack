/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  cache: true,
  output: {
    pathinfo: true,
  },
  optimization: {
    minimize: false,
    sideEffects: true,
    usedExports: true,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
    cache: {
      type: 'memory',
    },
  },
};
