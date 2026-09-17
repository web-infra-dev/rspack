/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  optimization: {
    mangleExports: true,
    usedExports: true,
    providedExports: true,
    concatenateModules: false,
  },
};
