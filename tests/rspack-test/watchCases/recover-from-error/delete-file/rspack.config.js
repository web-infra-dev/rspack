/** @type {import("@rspack/core").Configuration} */
module.exports = {
  ignoreWarnings: [/FlagDependencyUsagePlugin/],
  optimization: {
    usedExports: true,
  },
};
