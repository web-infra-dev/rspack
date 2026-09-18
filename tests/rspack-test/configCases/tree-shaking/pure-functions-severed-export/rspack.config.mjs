/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  target: 'node',
  optimization: {
    innerGraph: true,
    minimize: false,
    sideEffects: true,
    usedExports: true,
  },
  experiments: {
    pureFunctions: true,
  },
};
