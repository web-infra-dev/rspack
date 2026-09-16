/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  target: 'node14',
  externalsPresets: {
    node: true,
  },
  optimization: {
    concatenateModules: true,
    usedExports: true,
    providedExports: true,
    mangleExports: true,
  },
};
