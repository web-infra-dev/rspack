/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    concatenateModules: true,
    usedExports: true,
    providedExports: true,
    minimize: false,
    mangleExports: 'size',
    inlineExports: false,
  },
};
