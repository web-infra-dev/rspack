/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    concatenateModules: true,
    inlineExports: true,
    mangleExports: false,
    minimize: false,
    providedExports: true,
    usedExports: true,
  },
};
