/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    removeAvailableModules: true,
    providedExports: true,
    usedExports: 'global',
  },
};
