/** @type {import("@rspack/core").Configuration} */
export default {
  ignoreWarnings: [/FlagDependencyUsagePlugin/],
  optimization: {
    usedExports: true,
  },
};
