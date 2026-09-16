/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    concatenateModules: false,
    usedExports: true,
    providedExports: true,
    minimize: false,
    mangleExports: false,
    inlineExports: false,
  },
};
