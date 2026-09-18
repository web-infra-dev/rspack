/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    moduleIds: 'named',
    usedExports: true,
    providedExports: true,
    concatenateModules: true,
  },
};
