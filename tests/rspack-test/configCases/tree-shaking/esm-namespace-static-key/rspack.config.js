/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  module: {
    parser: {
      javascript: {
        exportsPresence: 'warn',
      },
    },
  },
  optimization: {
    sideEffects: true,
    usedExports: true,
  },
};
