/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  optimization: {
    concatenateModules: true,
    inlineExports: true,
    minimize: false,
    providedExports: true,
    sideEffects: true,
    usedExports: true,
  },
};
